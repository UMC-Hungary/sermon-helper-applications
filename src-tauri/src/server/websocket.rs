use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        FromRequestParts, Query, Request, State,
    },
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::future::poll_fn;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_postgres::AsyncMessage;
use chrono::Utc;
use uuid::Uuid;

use sqlx::PgPool;

use crate::connectors::{facebook, youtube, ConnectorStatus};
use crate::models::event::{fetch_event, Event};
use crate::models::recording::Recording;
use crate::server::AppState;

#[derive(Deserialize, Serialize)]
struct PgNotify<T> {
    operation: String,
    record: T,
}

#[derive(Deserialize)]
pub struct WsQuery {
    token: String,
}

pub async fn ws_handler(State(state): State<AppState>, req: Request) -> Response {
    // Reject plain HTTP requests immediately with 426 rather than Axum's
    // generic 400 "Connection header did not include 'upgrade'".  This also
    // avoids having CorsLayer (applied on /api routes only) ever touching the
    // WebSocket upgrade path.
    let is_upgrade = req
        .headers()
        .get(header::CONNECTION)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.to_ascii_lowercase().contains("upgrade"));

    if !is_upgrade {
        return (
            StatusCode::UPGRADE_REQUIRED,
            [
                (header::UPGRADE, "websocket"),
                (header::CONTENT_TYPE, "application/json"),
            ],
            r#"{"error":"upgrade_required","description":"This endpoint requires a WebSocket connection, not a plain HTTP request.","connect":"ws://<host>/ws?token=<your-token>","auth":"Pass the bearer token as the 'token' query parameter."}"#,
        )
            .into_response();
    }

    let (mut parts, body) = req.into_parts();

    let query = match Query::<WsQuery>::from_request_parts(&mut parts, &state).await {
        Ok(q) => q.0,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    let current_token = state.auth_token.read().await.clone();
    if query.token != current_token {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let ws = match WebSocketUpgrade::from_request_parts(&mut parts, &state).await {
        Ok(ws) => ws,
        Err(e) => return e.into_response(),
    };

    drop(body); // WebSocket requests have no body; release it explicitly.
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

    // Push current connector statuses so the client doesn't have to poll.
    let obs_status = state.obs_connector.get_status().await;
    let vmix_status = state.vmix_connector.get_status();
    let yt_status = state.youtube_connector.get_status().await;
    let fb_status = state.facebook_connector.get_status().await;
    for (connector, status) in [
        ("obs", obs_status),
        ("vmix", vmix_status),
        ("youtube", yt_status),
        ("facebook", fb_status),
    ] {
        let msg = json!({
            "type": "connector.status",
            "connector": connector,
            "status": status,
        })
        .to_string();
        let _ = tx.send(Message::Text(msg.into()));
    }

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
    app_state: AppState,
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

        let msg_text = if channel == "event_changes" {
            match serde_json::from_str::<PgNotify<Event>>(payload) {
                Ok(n) => {
                    // Spawn non-blocking scheduling tasks for INSERT/UPDATE
                    if n.operation == "INSERT" || n.operation == "UPDATE" {
                        spawn_scheduling_tasks(n.record.clone(), app_state.clone());
                    }
                    json!({ "type": "event.changed", "data": n }).to_string()
                }
                Err(e) => {
                    tracing::warn!("Failed to parse event notification: {e}");
                    continue;
                }
            }
        } else {
            match serde_json::from_str::<PgNotify<Recording>>(payload) {
                Ok(n) => json!({ "type": "recording.changed", "data": n }).to_string(),
                Err(e) => {
                    tracing::warn!("Failed to parse recording notification: {e}");
                    continue;
                }
            }
        };

        let clients = ws_clients.read().await;
        for tx in clients.values() {
            let _ = tx.send(Message::Text(msg_text.clone().into()));
        }
    }
}

/// Broadcast an `event.changed` message to all connected WebSocket clients.
pub async fn broadcast_event_changed(state: &AppState, operation: &str, event: &Event) {
    let msg = json!({
        "type": "event.changed",
        "data": { "operation": operation, "record": event }
    })
    .to_string();
    let clients = state.ws_clients.read().await;
    for tx in clients.values() {
        let _ = tx.send(Message::Text(msg.clone().into()));
    }
}

/// Spawns a detached task that schedules the event on connected social platforms.
/// Uses SET LOCAL app.skip_sync_notify='true' to prevent re-triggering the NOTIFY loop.
pub fn spawn_scheduling_tasks(event: Event, state: AppState) {
    tokio::spawn(async move {
        // ── YouTube ──────────────────────────────────────────────────────────
        let yt_status = state.youtube_connector.get_status().await;
        if matches!(yt_status, ConnectorStatus::Connected) {
            if let Some(token) = youtube::load_tokens(&state.pool).await {
                let yt_conn = event.connection("youtube");
                let yt_schedule_status = yt_conn
                    .map(|c| c.schedule_status.as_str())
                    .unwrap_or("not_scheduled");
                let existing = yt_conn.and_then(|c| c.external_id.as_deref());
                let privacy_status = yt_conn
                    .and_then(|c| c.privacy_status.as_deref())
                    .unwrap_or("private");
                if yt_schedule_status != "scheduled" && event.date_time > Utc::now() {
                    match youtube::schedule_event(
                        &event.id.to_string(),
                        &event.title,
                        &event.date_time,
                        &token.access_token,
                        existing,
                        privacy_status,
                    )
                    .await
                    {
                        Ok(result) => {
                            match write_youtube_result(&state, event.id, &result).await {
                                Ok(updated) => broadcast_event_changed(&state, "UPDATE", &updated).await,
                                Err(e) => tracing::error!("YouTube DB write failed: {e}"),
                            }
                        }
                        Err(e) => {
                            tracing::error!("YouTube auto-schedule failed for {}: {e}", event.id);
                            let _ = write_youtube_status(&state.pool, event.id, "failed").await;
                        }
                    }
                }
            }
        }

        // ── Facebook ─────────────────────────────────────────────────────────
        let fb_status = state.facebook_connector.get_status().await;
        if matches!(fb_status, ConnectorStatus::Connected) {
            if let Some(token) = facebook::load_tokens(&state.pool).await {
                let config = state.facebook_config.read().await.clone();
                let fb_conn = event.connection("facebook");
                let fb_schedule_status = fb_conn
                    .map(|c| c.schedule_status.as_str())
                    .unwrap_or("not_scheduled");
                let fb_privacy_status = fb_conn
                    .and_then(|c| c.privacy_status.as_deref())
                    .unwrap_or("EVERYONE");
                if !config.page_id.is_empty() && fb_schedule_status != "scheduled" {
                    match facebook::schedule_event(
                        &event.title,
                        &event.date_time,
                        &token.access_token,
                        &config.page_id,
                        fb_privacy_status,
                    )
                    .await
                    {
                        Ok(result) => {
                            match write_facebook_result(&state, event.id, &result).await {
                                Ok(updated) => {
                                    broadcast_event_changed(&state, "UPDATE", &updated).await;
                                }
                                Err(e) => tracing::error!("Facebook DB write failed: {e}"),
                            }
                        }
                        Err(e) => {
                            tracing::error!(
                                "Facebook auto-schedule failed for {}: {e}",
                                event.id
                            );
                            let _ = write_facebook_status(&state.pool, event.id, "failed").await;
                        }
                    }
                }
            }
        }
    });
}

pub async fn write_youtube_result(
    state: &AppState,
    event_id: Uuid,
    result: &youtube::BroadcastResult,
) -> anyhow::Result<Event> {
    let mut tx = state.pool.begin().await?;
    sqlx::query("SET LOCAL app.skip_sync_notify = 'true'")
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        r#"INSERT INTO event_connections (event_id, platform, external_id, stream_url, schedule_status)
           VALUES ($1, 'youtube', $2, $3, 'scheduled')
           ON CONFLICT (event_id, platform) DO UPDATE SET
               external_id     = EXCLUDED.external_id,
               stream_url      = EXCLUDED.stream_url,
               schedule_status = 'scheduled',
               updated_at      = NOW()"#,
    )
    .bind(event_id)
    .bind(&result.broadcast_id)
    .bind(&result.watch_url)
    .execute(&mut *tx)
    .await?;
    sqlx::query("UPDATE events SET updated_at = NOW() WHERE id = $1")
        .bind(event_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    fetch_event(event_id, &state.pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("event {event_id} not found after YouTube write"))
}

pub async fn write_youtube_status(
    pool: &PgPool,
    event_id: Uuid,
    status: &str,
) -> anyhow::Result<()> {
    let mut tx = pool.begin().await?;
    sqlx::query("SET LOCAL app.skip_sync_notify = 'true'")
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        r#"INSERT INTO event_connections (event_id, platform, schedule_status)
           VALUES ($1, 'youtube', $2)
           ON CONFLICT (event_id, platform) DO UPDATE SET
               schedule_status = EXCLUDED.schedule_status,
               updated_at      = NOW()"#,
    )
    .bind(event_id)
    .bind(status)
    .execute(&mut *tx)
    .await?;
    sqlx::query("UPDATE events SET updated_at = NOW() WHERE id = $1")
        .bind(event_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn write_facebook_result(
    state: &AppState,
    event_id: Uuid,
    result: &facebook::FacebookScheduleResult,
) -> anyhow::Result<Event> {
    let extra = serde_json::json!({"stream_id": result.stream_id});
    let mut tx = state.pool.begin().await?;
    sqlx::query("SET LOCAL app.skip_sync_notify = 'true'")
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        r#"INSERT INTO event_connections (event_id, platform, external_id, event_url, schedule_status, extra)
           VALUES ($1, 'facebook', $2, $3, 'scheduled', $4)
           ON CONFLICT (event_id, platform) DO UPDATE SET
               external_id     = EXCLUDED.external_id,
               event_url       = EXCLUDED.event_url,
               schedule_status = 'scheduled',
               extra           = EXCLUDED.extra,
               updated_at      = NOW()"#,
    )
    .bind(event_id)
    .bind(&result.event_id)
    .bind(&result.event_url)
    .bind(extra)
    .execute(&mut *tx)
    .await?;
    sqlx::query("UPDATE events SET updated_at = NOW() WHERE id = $1")
        .bind(event_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    fetch_event(event_id, &state.pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("event {event_id} not found after Facebook write"))
}

pub async fn write_facebook_status(
    pool: &PgPool,
    event_id: Uuid,
    status: &str,
) -> anyhow::Result<()> {
    let mut tx = pool.begin().await?;
    sqlx::query("SET LOCAL app.skip_sync_notify = 'true'")
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        r#"INSERT INTO event_connections (event_id, platform, schedule_status)
           VALUES ($1, 'facebook', $2)
           ON CONFLICT (event_id, platform) DO UPDATE SET
               schedule_status = EXCLUDED.schedule_status,
               updated_at      = NOW()"#,
    )
    .bind(event_id)
    .bind(status)
    .execute(&mut *tx)
    .await?;
    sqlx::query("UPDATE events SET updated_at = NOW() WHERE id = $1")
        .bind(event_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}
