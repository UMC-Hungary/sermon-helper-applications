pub mod auth;
pub mod openapi;
pub mod routes;
pub mod websocket;

use axum::{
    middleware,
    routing::{get, post, put},
    Router,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, RwLock};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use uuid::Uuid;

use axum::extract::ws::Message;
use serde_json::json;
use sqlx::PgPool;

use crate::connectors::{
    facebook::FacebookConnector, obs::ObsConnector, vmix::VmixConnector,
    youtube::YouTubeConnector, FacebookConfig, YouTubeConfig,
};
use crate::scheduler::CronScheduler;

/// Fixed port for OAuth callbacks — must match Google/Facebook Cloud Console configuration.
pub(crate) const OAUTH_CALLBACK_PORT: u16 = 8766;
/// Exact redirect URI registered in both Google and Facebook Cloud Consoles.
pub(crate) const OAUTH_REDIRECT_URI: &str = "http://127.0.0.1:8766/callback";

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub auth_token: Arc<RwLock<String>>,
    pub ws_clients: Arc<RwLock<HashMap<Uuid, mpsc::UnboundedSender<Message>>>>,
    pub server_id: String,
    pub obs_connector: Arc<ObsConnector>,
    pub vmix_connector: Arc<VmixConnector>,
    pub youtube_connector: Arc<YouTubeConnector>,
    pub facebook_connector: Arc<FacebookConnector>,
    pub youtube_config: Arc<RwLock<YouTubeConfig>>,
    pub facebook_config: Arc<RwLock<FacebookConfig>>,
    /// Pending OAuth state tokens: state_string → (connector_name, created_at)
    pub oauth_states: Arc<RwLock<HashMap<String, (String, Instant)>>>,
    pub app_handle: tauri::AppHandle,
    pub cron_scheduler: Arc<CronScheduler>,
}

pub async fn build_and_serve(
    pool: PgPool,
    auth_token: Arc<RwLock<String>>,
    connection_url: String,
    port: u16,
    static_dir: Option<String>,
    obs_connector: Arc<ObsConnector>,
    vmix_connector: Arc<VmixConnector>,
    youtube_connector: Arc<YouTubeConnector>,
    facebook_connector: Arc<FacebookConnector>,
    youtube_config: Arc<RwLock<YouTubeConfig>>,
    facebook_config: Arc<RwLock<FacebookConfig>>,
    oauth_states: Arc<RwLock<std::collections::HashMap<String, (String, std::time::Instant)>>>,
    app_handle: tauri::AppHandle,
    cron_scheduler: Arc<CronScheduler>,
) -> anyhow::Result<()> {
    let ws_clients: Arc<RwLock<HashMap<Uuid, mpsc::UnboundedSender<Message>>>> =
        Arc::new(RwLock::new(HashMap::new()));
    let server_id = Uuid::new_v4().to_string();

    // Initial scheduler load — runs with the real ws_clients so broadcasts reach clients.
    {
        let pool_c = pool.clone();
        let clients_c = ws_clients.clone();
        let yt_c = youtube_connector.clone();
        let sched_c = cron_scheduler.clone();
        tokio::spawn(async move {
            sched_c.reload(pool_c, clients_c, yt_c).await;
        });
    }

    let state = AppState {
        pool,
        auth_token,
        ws_clients: ws_clients.clone(),
        server_id,
        obs_connector: obs_connector.clone(),
        vmix_connector,
        youtube_connector: youtube_connector.clone(),
        facebook_connector: facebook_connector.clone(),
        youtube_config,
        facebook_config,
        oauth_states,
        app_handle,
        cron_scheduler,
    };

    {
        let clients = ws_clients.clone();
        let url = connection_url.clone();
        let st = state.clone();
        tokio::spawn(async move {
            websocket::start_notify_listener(url, clients, st).await;
        });
    }

    // Forward OBS status broadcasts to all connected WS clients.
    {
        let clients = ws_clients.clone();
        let mut obs_rx = obs_connector.status_tx.subscribe();
        tokio::spawn(async move {
            while let Ok(status) = obs_rx.recv().await {
                let msg = json!({
                    "type": "connector.status",
                    "connector": "obs",
                    "status": status,
                })
                .to_string();
                let guard = clients.read().await;
                for tx in guard.values() {
                    let _ = tx.send(Message::Text(msg.clone().into()));
                }
            }
        });
    }

    // Forward OBS streaming/recording state changes to all connected WS clients.
    {
        let clients = ws_clients.clone();
        let mut obs_state_rx = obs_connector.output_state_tx.subscribe();
        tokio::spawn(async move {
            while let Ok(state) = obs_state_rx.recv().await {
                let msg = json!({
                    "type": "connector.state",
                    "connector": "obs",
                    "isStreaming": state.is_streaming,
                    "isRecording": state.is_recording,
                })
                .to_string();
                let guard = clients.read().await;
                for tx in guard.values() {
                    let _ = tx.send(Message::Text(msg.clone().into()));
                }
            }
        });
    }

    // Forward YouTube status broadcasts to all connected WS clients.
    {
        let clients = ws_clients.clone();
        let mut yt_rx = youtube_connector.status_tx.subscribe();
        tokio::spawn(async move {
            while let Ok(status) = yt_rx.recv().await {
                let msg = json!({
                    "type": "connector.status",
                    "connector": "youtube",
                    "status": status,
                })
                .to_string();
                let guard = clients.read().await;
                for tx in guard.values() {
                    let _ = tx.send(Message::Text(msg.clone().into()));
                }
            }
        });
    }

    // Forward Facebook status broadcasts to all connected WS clients.
    {
        let clients = ws_clients.clone();
        let mut fb_rx = facebook_connector.status_tx.subscribe();
        tokio::spawn(async move {
            while let Ok(status) = fb_rx.recv().await {
                let msg = json!({
                    "type": "connector.status",
                    "connector": "facebook",
                    "status": status,
                })
                .to_string();
                let guard = clients.read().await;
                for tx in guard.values() {
                    let _ = tx.send(Message::Text(msg.clone().into()));
                }
            }
        });
    }

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // OAuth URL + logout routes — no auth required
    let oauth_routes = Router::new()
        .route("/auth/youtube/url", get(routes::youtube_auth_url))
        .route("/auth/youtube/logout", post(routes::youtube_logout))
        .route("/auth/facebook/url", get(routes::facebook_auth_url))
        .route("/auth/facebook/logout", post(routes::facebook_logout));

    let api_routes = Router::new()
        .route(
            "/events",
            get(routes::list_events).post(routes::create_event),
        )
        .route("/events/{id}", get(routes::get_event).put(routes::update_event))
        .route(
            "/events/{id}/recordings",
            get(routes::list_recordings).post(routes::create_recording),
        )
        .route("/connectors/status", get(routes::get_connector_statuses))
        .route("/stream/stats", get(routes::get_stream_stats))
        .route("/connectors/youtube/content", get(routes::get_youtube_content))
        .route("/connectors/youtube/stream-key", get(routes::get_youtube_stream_key))
        .route("/connectors/facebook/stream-key", get(routes::get_facebook_stream_key))
        .route(
            "/connectors/youtube/schedule/{event_id}",
            post(routes::trigger_youtube_schedule),
        )
        .route(
            "/connectors/facebook/schedule/{event_id}",
            post(routes::trigger_facebook_schedule),
        )
        .route(
            "/cron-jobs",
            get(routes::list_cron_jobs).post(routes::create_cron_job),
        )
        .route(
            "/cron-jobs/{id}",
            put(routes::update_cron_job).delete(routes::delete_cron_job),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::auth_middleware,
        ))
        .merge(oauth_routes);

    // CorsLayer must be the outermost layer so it intercepts OPTIONS preflight
    // requests before they reach the auth middleware. tower-http's CorsLayer
    // only adds response headers and never modifies request headers, so it is
    // safe to apply to all routes including /ws.
    let mut app = Router::new()
        .route("/openapi.json", get(openapi::serve_spec))
        .route("/docs", get(openapi::serve_docs))
        .route("/ws", get(websocket::ws_handler))
        .nest("/api", api_routes)
        .with_state(state.clone());

    if let Some(dir) = static_dir {
        let fallback = ServeFile::new(format!("{dir}/index.html"));
        app = app.fallback_service(ServeDir::new(&dir).not_found_service(fallback));
    }

    let app = app.layer(cors);

    // Dedicated OAuth callback listener on the fixed port 8766.
    // This keeps the redirect URI stable (matching the Cloud Console config)
    // even if the main API port is changed.
    let callback_addr = SocketAddr::from(([127, 0, 0, 1], OAUTH_CALLBACK_PORT));
    match TcpListener::bind(callback_addr).await {
        Ok(cb_listener) => {
            let cb_app = Router::new()
                .route("/callback", get(routes::oauth_callback))
                .with_state(state.clone());
            tracing::info!("OAuth callback server listening on {callback_addr}");
            tokio::spawn(async move {
                let _ = axum::serve(cb_listener, cb_app).await;
            });
        }
        Err(e) => {
            tracing::warn!("Could not bind OAuth callback port {OAUTH_CALLBACK_PORT}: {e}");
        }
    }

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("Axum server listening on {addr}");
    axum::serve(listener, app).await?;

    Ok(())
}
