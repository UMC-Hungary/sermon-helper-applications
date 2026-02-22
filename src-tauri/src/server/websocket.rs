use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::StatusCode,
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::future::poll_fn;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_postgres::AsyncMessage;
use uuid::Uuid;

use crate::server::AppState;

#[derive(Deserialize)]
pub struct WsQuery {
    token: String,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsQuery>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let current_token = state.auth_token.read().await.clone();
    if query.token != current_token {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let server_id = state.server_id.clone();
    ws.on_upgrade(move |socket| handle_socket(socket, state, server_id))
}

async fn handle_socket(socket: WebSocket, state: AppState, server_id: String) {
    let client_id = Uuid::new_v4();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    {
        let mut clients = state.ws_clients.write().await;
        clients.insert(client_id, tx.clone());
    }

    let connected_msg = json!({ "type": "connected", "serverId": server_id }).to_string();
    let _ = tx.send(Message::Text(connected_msg.into()));

    let (mut ws_sink, mut ws_stream) = socket.split();

    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sink.send(msg).await.is_err() {
                break;
            }
        }
    });

    let recv_task = tokio::spawn(async move {
        while let Some(Ok(_msg)) = ws_stream.next().await {}
    });

    tokio::select! {
        _ = send_task => {}
        _ = recv_task => {}
    }

    let mut clients = state.ws_clients.write().await;
    clients.remove(&client_id);
}

pub async fn start_notify_listener(
    connection_url: String,
    ws_clients: Arc<RwLock<HashMap<Uuid, mpsc::UnboundedSender<Message>>>>,
) {
    let (client, mut connection) =
        match tokio_postgres::connect(&connection_url, tokio_postgres::NoTls).await {
            Ok(pair) => pair,
            Err(e) => {
                tracing::error!("NOTIFY listener connect failed: {e}");
                return;
            }
        };

    let (notify_tx, mut notify_rx) = mpsc::unbounded_channel::<tokio_postgres::Notification>();

    tokio::spawn(async move {
        loop {
            match poll_fn(|cx| connection.poll_message(cx)).await {
                Some(Ok(AsyncMessage::Notification(n))) => {
                    let _ = notify_tx.send(n);
                }
                Some(Ok(_)) => {}
                None | Some(Err(_)) => break,
            }
        }
    });

    if let Err(e) = client.execute("LISTEN event_changes", &[]).await {
        tracing::error!("LISTEN event_changes failed: {e}");
        return;
    }
    if let Err(e) = client.execute("LISTEN recording_changes", &[]).await {
        tracing::error!("LISTEN recording_changes failed: {e}");
        return;
    }

    while let Some(notification) = notify_rx.recv().await {
        let channel = notification.channel();
        let payload = notification.payload();

        let ws_type = if channel == "event_changes" {
            "event.changed"
        } else {
            "recording.changed"
        };

        let msg_text = match serde_json::from_str::<serde_json::Value>(payload) {
            Ok(data) => json!({ "type": ws_type, "data": data }).to_string(),
            Err(_) => json!({ "type": ws_type, "raw": payload }).to_string(),
        };

        let clients = ws_clients.read().await;
        for tx in clients.values() {
            let _ = tx.send(Message::Text(msg_text.clone().into()));
        }
    }
}
