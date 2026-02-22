pub mod auth;
pub mod routes;
pub mod websocket;

use axum::{
    middleware,
    routing::get,
    Router,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, RwLock};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use uuid::Uuid;

use axum::extract::ws::Message;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub auth_token: Arc<RwLock<String>>,
    pub ws_clients: Arc<RwLock<HashMap<Uuid, mpsc::UnboundedSender<Message>>>>,
    pub server_id: String,
}

pub async fn build_and_serve(
    pool: PgPool,
    auth_token: Arc<RwLock<String>>,
    connection_url: String,
    port: u16,
    static_dir: Option<String>,
) -> anyhow::Result<()> {
    let ws_clients: Arc<RwLock<HashMap<Uuid, mpsc::UnboundedSender<Message>>>> =
        Arc::new(RwLock::new(HashMap::new()));
    let server_id = Uuid::new_v4().to_string();

    let state = AppState {
        pool,
        auth_token,
        ws_clients: ws_clients.clone(),
        server_id,
    };

    {
        let clients = ws_clients.clone();
        let url = connection_url.clone();
        tokio::spawn(async move {
            websocket::start_notify_listener(url, clients).await;
        });
    }

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api_routes = Router::new()
        .route(
            "/events",
            get(routes::list_events).post(routes::create_event),
        )
        .route("/events/{id}", get(routes::get_event))
        .route(
            "/events/{id}/recordings",
            get(routes::list_recordings).post(routes::create_recording),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::auth_middleware,
        ));

    let mut app = Router::new()
        .route("/ws", get(websocket::ws_handler))
        .nest("/api", api_routes)
        .layer(cors)
        .with_state(state);

    if let Some(dir) = static_dir {
        let fallback = ServeFile::new(format!("{dir}/index.html"));
        app = app.fallback_service(ServeDir::new(&dir).not_found_service(fallback));
    }

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("Axum server listening on {addr}");
    axum::serve(listener, app).await?;

    Ok(())
}
