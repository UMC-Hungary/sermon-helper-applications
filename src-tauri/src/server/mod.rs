pub mod auth;
pub mod caption;
pub mod openapi;
pub mod ppt;
pub mod presenter;
pub mod routes;
pub mod websocket;

use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::AtomicBool;
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
    broadlink::BroadlinkConnector, facebook::FacebookConnector, obs::ObsConnector,
    vmix::VmixConnector, youtube::YouTubeConnector, FacebookConfig, YouTubeConfig,
};
use crate::obs_devices::ObsAvailableDevices;
#[cfg(target_os = "macos")]
use crate::connectors::keynote::KeynoteConnector;
use crate::models::event::find_current_event;
use crate::scheduler::CronScheduler;
use crate::uploader::UploadService;

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
    pub broadlink_connector: Arc<BroadlinkConnector>,
    pub broadlink_learn_active: Arc<AtomicBool>,
    pub youtube_config: Arc<RwLock<YouTubeConfig>>,
    pub facebook_config: Arc<RwLock<FacebookConfig>>,
    /// Pending OAuth state tokens: state_string → (connector_name, created_at)
    pub oauth_states: Arc<RwLock<HashMap<String, (String, Instant)>>>,
    pub app_handle: Option<tauri::AppHandle>,
    pub cron_scheduler: Arc<CronScheduler>,
    pub upload_service: Arc<UploadService>,
    /// Cached result of the last OBS device scan; `None` until first scan completes.
    pub obs_available_devices: Arc<tokio::sync::RwLock<Option<ObsAvailableDevices>>>,
    /// In-memory state for the active web-presenter session.
    pub presenter_state: Arc<tokio::sync::RwLock<presenter::PresenterState>>,
    /// Metadata for every currently-connected WebSocket client.
    pub ws_client_info: Arc<tokio::sync::RwLock<HashMap<Uuid, websocket::WsClientInfo>>>,
    #[cfg(target_os = "macos")]
    pub keynote_connector: Arc<KeynoteConnector>,
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
    broadlink_connector: Arc<BroadlinkConnector>,
    youtube_config: Arc<RwLock<YouTubeConfig>>,
    facebook_config: Arc<RwLock<FacebookConfig>>,
    oauth_states: Arc<RwLock<std::collections::HashMap<String, (String, std::time::Instant)>>>,
    app_handle: Option<tauri::AppHandle>,
    cron_scheduler: Arc<CronScheduler>,
    #[cfg(target_os = "macos")] keynote_connector: Arc<KeynoteConnector>,
) -> anyhow::Result<()> {
    let ws_clients: Arc<RwLock<HashMap<Uuid, mpsc::UnboundedSender<Message>>>> =
        Arc::new(RwLock::new(HashMap::new()));
    let server_id = Uuid::new_v4().to_string();

    // Create the upload service here so it shares the ws_clients Arc.
    let upload_service = Arc::new(UploadService::new(
        pool.clone(),
        Arc::clone(&youtube_connector),
        Arc::clone(&facebook_connector),
        Arc::clone(&obs_connector),
        Arc::clone(&facebook_config),
        Arc::clone(&ws_clients),
    ));

    // Initial scheduler load — runs with the real ws_clients so broadcasts reach clients.
    {
        let pool_c = pool.clone();
        let clients_c = ws_clients.clone();
        let yt_c = youtube_connector.clone();
        let sched_c = cron_scheduler.clone();
        let us_c = upload_service.clone();
        tokio::spawn(async move {
            sched_c.reload(pool_c, clients_c, yt_c, us_c).await;
        });
    }

    let obs_available_devices: Arc<tokio::sync::RwLock<Option<ObsAvailableDevices>>> =
        Arc::new(tokio::sync::RwLock::new(None));

    let presenter_state: Arc<tokio::sync::RwLock<presenter::PresenterState>> =
        Arc::new(tokio::sync::RwLock::new(presenter::PresenterState::empty()));

    let ws_client_info: Arc<tokio::sync::RwLock<HashMap<Uuid, websocket::WsClientInfo>>> =
        Arc::new(tokio::sync::RwLock::new(HashMap::new()));

    let state = AppState {
        pool,
        auth_token,
        ws_clients: ws_clients.clone(),
        server_id,
        obs_connector: obs_connector.clone(),
        vmix_connector,
        youtube_connector: youtube_connector.clone(),
        facebook_connector: facebook_connector.clone(),
        broadlink_connector: broadlink_connector.clone(),
        broadlink_learn_active: Arc::new(AtomicBool::new(false)),
        youtube_config,
        facebook_config,
        oauth_states,
        app_handle,
        cron_scheduler,
        upload_service: upload_service.clone(),
        obs_available_devices: obs_available_devices.clone(),
        presenter_state: presenter_state.clone(),
        ws_client_info: ws_client_info.clone(),
        #[cfg(target_os = "macos")]
        keynote_connector: keynote_connector.clone(),
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

    // Forward Broadlink status broadcasts to all connected WS clients.
    {
        let clients = ws_clients.clone();
        let mut bl_rx = broadlink_connector.status_tx.subscribe();
        tokio::spawn(async move {
            while let Ok(status) = bl_rx.recv().await {
                let msg = json!({
                    "type": "connector.status",
                    "connector": "broadlink",
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

    // Forward Broadlink learn results to all connected WS clients.
    {
        let clients = ws_clients.clone();
        let mut learn_rx = broadlink_connector.learn_tx.subscribe();
        tokio::spawn(async move {
            while let Ok(ev) = learn_rx.recv().await {
                let msg = json!({
                    "type": "broadlink.learn.result",
                    "code": ev.code,
                    "error": ev.error,
                })
                .to_string();
                let guard = clients.read().await;
                for tx in guard.values() {
                    let _ = tx.send(Message::Text(msg.clone().into()));
                }
            }
        });
    }

    // Forward Keynote status broadcasts to all connected WS clients (macOS only).
    #[cfg(target_os = "macos")]
    {
        let clients = ws_clients.clone();
        let mut kn_rx = keynote_connector.status_tx.subscribe();
        tokio::spawn(async move {
            while let Ok(status) = kn_rx.recv().await {
                let msg = json!({
                    "type": "keynote.status",
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

    // Start Keynote adaptive polling loop (macOS only).
    #[cfg(target_os = "macos")]
    Arc::clone(&keynote_connector).start_polling();

    // Forward OBS streaming/recording state to all connected WS clients.
    {
        let clients = ws_clients.clone();
        let mut obs_state_rx = obs_connector.state_tx.subscribe();
        tokio::spawn(async move {
            while let Ok(ev) = obs_state_rx.recv().await {
                let msg = json!({
                    "type": "obs.state",
                    "isStreaming": ev.is_streaming,
                    "isRecording": ev.is_recording,
                })
                .to_string();
                let guard = clients.read().await;
                for tx in guard.values() {
                    let _ = tx.send(Message::Text(msg.clone().into()));
                }
            }
        });
    }

    // Detect OBS recordings and auto-assign to the current event.
    {
        let pool_c = state.pool.clone();
        let clients_c = ws_clients.clone();
        let mut recording_rx = obs_connector.recording_tx.subscribe();
        tokio::spawn(async move {
            handle_obs_recording_events(pool_c, clients_c, &mut recording_rx).await;
        });
    }

    // Rescan OBS devices whenever devices_tx fires (input added/removed/changed or initial connect).
    {
        let pool_c = state.pool.clone();
        let obs_c = obs_connector.clone();
        let devices_cache = obs_available_devices.clone();
        let clients_c = ws_clients.clone();
        let mut devices_rx = obs_connector.devices_tx.subscribe();
        tokio::spawn(async move {
            loop {
                match devices_rx.recv().await {
                    Ok(()) => {}
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
                let client_opt = obs_c.client.lock().await.clone();
                let Some(client) = client_opt else { continue };
                let scanned = crate::obs_devices::scan_obs_devices(&client).await;
                let listeners: Vec<crate::models::device_listener::DeviceListener> =
                    sqlx::query_as("SELECT * FROM device_listeners ORDER BY created_at")
                        .fetch_all(&pool_c)
                        .await
                        .unwrap_or_default();
                let statuses = crate::obs_devices::compute_listener_statuses(&scanned, &listeners);
                *devices_cache.write().await = Some(scanned.clone());
                let msg = serde_json::json!({
                    "type": "obs.devices.available",
                    "devices": scanned,
                    "listenerStatuses": statuses,
                })
                .to_string();
                let guard = clients_c.read().await;
                for tx in guard.values() {
                    let _ = tx.send(axum::extract::ws::Message::Text(msg.clone().into()));
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

    // PPT folder and file search routes (all platforms).
    let ppt_routes = Router::new()
        .route(
            "/ppt/folders",
            get(ppt::list_folders).post(ppt::add_folder),
        )
        .route("/ppt/folders/{id}", delete(ppt::remove_folder))
        .route("/ppt/files", get(ppt::search_files));

    // Keynote control routes (macOS only; 501 stub on other platforms).
    #[cfg(target_os = "macos")]
    let keynote_routes = Router::new()
        .route("/keynote/status", get(ppt::keynote_status))
        .route("/keynote/open", post(ppt::keynote_open))
        .route("/keynote/next", post(ppt::keynote_next))
        .route("/keynote/prev", post(ppt::keynote_prev))
        .route("/keynote/first", post(ppt::keynote_first))
        .route("/keynote/last", post(ppt::keynote_last))
        .route("/keynote/goto", post(ppt::keynote_goto))
        .route("/keynote/start", post(ppt::keynote_start))
        .route("/keynote/stop", post(ppt::keynote_stop))
        .route("/keynote/close_all", post(ppt::keynote_close_all));

    #[cfg(not(target_os = "macos"))]
    let keynote_routes = Router::new()
        .route("/keynote/status", get(ppt::keynote_not_implemented))
        .route("/keynote/open", post(ppt::keynote_not_implemented))
        .route("/keynote/next", post(ppt::keynote_not_implemented))
        .route("/keynote/prev", post(ppt::keynote_not_implemented))
        .route("/keynote/first", post(ppt::keynote_not_implemented))
        .route("/keynote/last", post(ppt::keynote_not_implemented))
        .route("/keynote/goto", post(ppt::keynote_not_implemented))
        .route("/keynote/start", post(ppt::keynote_not_implemented))
        .route("/keynote/stop", post(ppt::keynote_not_implemented))
        .route("/keynote/close_all", post(ppt::keynote_not_implemented));

    let api_routes = Router::new()
        .route(
            "/events",
            get(routes::list_events).post(routes::create_event),
        )
        .route(
            "/events/{id}",
            get(routes::get_event).put(routes::update_event).delete(routes::delete_event),
        )
        .route(
            "/events/{id}/recordings",
            get(routes::list_recordings).post(routes::create_recording),
        )
        .route(
            "/events/{id}/recordings/{recording_id}",
            delete(routes::delete_recording),
        )
        .route(
            "/events/{id}/activities",
            get(routes::list_event_activities).post(routes::create_event_activity),
        )
        .route(
            "/events/{id}/activities/{activity_id}",
            delete(routes::delete_event_activity),
        )
        .route("/recordings", get(routes::list_all_recordings))
        .route(
            "/recordings/untracked",
            get(routes::list_untracked_recordings),
        )
        .route(
            "/recordings/untracked/{id}/assign",
            post(routes::assign_untracked_recording),
        )
        .route(
            "/recordings/untracked/{id}",
            delete(routes::delete_untracked_recording),
        )
        .route("/connectors/broadlink/status", get(routes::broadlink_get_status))
        .route(
            "/connectors/broadlink/devices",
            get(routes::broadlink_list_devices).post(routes::broadlink_add_device),
        )
        .route(
            "/connectors/broadlink/devices/{id}",
            delete(routes::broadlink_remove_device),
        )
        .route(
            "/connectors/broadlink/discover",
            post(routes::broadlink_discover),
        )
        .route(
            "/connectors/broadlink/commands",
            get(routes::broadlink_list_commands).post(routes::broadlink_add_command),
        )
        .route(
            "/connectors/broadlink/commands/{id}",
            put(routes::broadlink_update_command).delete(routes::broadlink_remove_command),
        )
        .route(
            "/connectors/broadlink/devices/{id}/learn",
            post(routes::broadlink_start_learn),
        )
        .route(
            "/connectors/broadlink/learn/cancel",
            post(routes::broadlink_cancel_learn),
        )
        .route(
            "/connectors/broadlink/commands/{id}/send",
            post(routes::broadlink_send_command),
        )
        .route("/connectors/state", get(routes::get_connector_state))
        .route("/connectors/status", get(routes::get_connector_statuses))
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
        .route(
            "/events/{id}/recordings/flag-upload",
            post(routes::flag_upload),
        )
        .route("/uploads/trigger", post(routes::trigger_upload_cycle))
        .merge(ppt_routes)
        .merge(keynote_routes)
        .route("/presenter/parse", post(presenter::parse_presentation))
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
        .route("/health", get(|| async { axum::http::StatusCode::OK }))
        .route("/caption", get(caption::caption_handler))
        .route("/caption/logo", get(caption::caption_logo_handler))
        .route("/openapi.json", get(openapi::serve_spec))
        .route("/docs", get(openapi::serve_docs))
        .route("/ws", get(websocket::ws_handler))
        .nest("/api", api_routes)
        .with_state(state.clone());

    if let Some(dir) = static_dir {
        let fallback = ServeFile::new(format!("{dir}/index.html"));
        app = app.fallback_service(ServeDir::new(&dir).fallback(fallback));
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

/// Probe the video file duration via `ffprobe`. Returns 0.0 if unavailable.
async fn probe_duration(path: &std::path::Path) -> f64 {
    let out = tokio::process::Command::new("ffprobe")
        .args(["-v", "quiet", "-of", "json", "-show_entries", "format=duration", "-i"])
        .arg(path)
        .output()
        .await;
    match out {
        Ok(o) if o.status.success() => {
            let text = String::from_utf8_lossy(&o.stdout);
            serde_json::from_str::<serde_json::Value>(&text)
                .ok()
                .and_then(|v| v["format"]["duration"].as_str().and_then(|s| s.parse::<f64>().ok()))
                .unwrap_or(0.0)
        }
        _ => 0.0,
    }
}

/// Receives OBS recording-stopped events and inserts the file into `recordings`
/// (if a current event exists) or `untracked_recordings`.
async fn handle_obs_recording_events(
    pool: PgPool,
    clients: Arc<RwLock<HashMap<Uuid, mpsc::UnboundedSender<Message>>>>,
    recording_rx: &mut tokio::sync::broadcast::Receiver<crate::connectors::obs::ObsRecordingEvent>,
) {
    loop {
        let event = match recording_rx.recv().await {
            Ok(e) => e,
            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
            Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
        };

        let file_path = event.output_path.to_string_lossy().to_string();
        let file_name = event
            .output_path
            .file_name()
            .map(|n: &std::ffi::OsStr| n.to_string_lossy().to_string())
            .unwrap_or_else(|| file_path.clone());

        let file_size: i64 = tokio::fs::metadata(&event.output_path)
            .await
            .map(|m| m.len() as i64)
            .unwrap_or(0);

        let duration_seconds = probe_duration(&event.output_path).await;

        match find_current_event(&pool).await {
            Ok(Some(ev)) => {
                let insert_result = sqlx::query(
                    "INSERT INTO recordings (event_id, file_path, file_name, file_size, duration_seconds, detected_at) \
                     VALUES ($1, $2, $3, $4, $5, NOW())",
                )
                .bind(ev.id)
                .bind(&file_path)
                .bind(&file_name)
                .bind(file_size)
                .bind(duration_seconds)
                .execute(&pool)
                .await;

                if let Err(e) = insert_result {
                    tracing::error!("handle_obs_recording_events: insert recording: {e}");
                    continue;
                }

                websocket::broadcast_recording_detected(&clients, &file_name, Some(&ev.title))
                    .await;
            }
            Ok(None) => {
                let insert_result = sqlx::query(
                    "INSERT INTO untracked_recordings (file_path, file_name, file_size, duration_seconds, detected_at) \
                     VALUES ($1, $2, $3, $4, NOW())",
                )
                .bind(&file_path)
                .bind(&file_name)
                .bind(file_size)
                .bind(duration_seconds)
                .execute(&pool)
                .await;

                if let Err(e) = insert_result {
                    tracing::error!("handle_obs_recording_events: insert untracked: {e}");
                    continue;
                }

                websocket::broadcast_recording_detected(&clients, &file_name, None).await;
            }
            Err(e) => {
                tracing::error!("handle_obs_recording_events: find_current_event: {e}");
            }
        }
    }
}
