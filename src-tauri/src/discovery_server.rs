//! Discovery server for mobile companion app integration.
//!
//! This module provides:
//! - mDNS/DNS-SD service registration for network discovery
//! - HTTP REST API for system status and control
//! - WebSocket for real-time status updates

use crate::mdns_service::{MdnsService, SERVICE_TYPE};
use axum::{
    extract::{
        rejection::JsonRejection,
        ws::{Message, WebSocket, WebSocketUpgrade},
        FromRequest, Request, State,
    },
    http::{header, HeaderMap, Method, StatusCode},
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{broadcast, Mutex, RwLock};
use tower_http::cors::{Any, CorsLayer};
use chrono::Utc;

/// Default port for the discovery server
pub const DEFAULT_PORT: u16 = 8765;

/// Server status information returned to frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveryServerInfo {
    pub running: bool,
    pub port: u16,
    pub addresses: Vec<String>,
    pub service_name: String,
    pub auth_required: bool,
    /// URL to API documentation (Swagger UI)
    pub docs_url: String,
}

/// Full server status including connection info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveryServerStatus {
    pub running: bool,
    pub port: Option<u16>,
    pub addresses: Vec<String>,
    pub connected_clients: u32,
    pub mdns_registered: bool,
    /// URL to API documentation (Swagger UI)
    pub docs_url: Option<String>,
}

/// System status for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemStatus {
    pub obs_connected: bool,
    pub obs_streaming: bool,
    pub obs_recording: bool,
    pub rode_interface: bool,
    pub main_display: bool,
    pub secondary_display: bool,
    pub youtube_logged_in: bool,
}

impl Default for SystemStatus {
    fn default() -> Self {
        Self {
            obs_connected: false,
            obs_streaming: false,
            obs_recording: false,
            rode_interface: false,
            main_display: false,
            secondary_display: false,
            youtube_logged_in: false,
        }
    }
}

/// OBS-specific status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObsStatus {
    pub connected: bool,
    pub streaming: bool,
    pub recording: bool,
    pub stream_timecode: Option<String>,
    pub record_timecode: Option<String>,
}

impl Default for ObsStatus {
    fn default() -> Self {
        Self {
            connected: false,
            streaming: false,
            recording: false,
            stream_timecode: None,
            record_timecode: None,
        }
    }
}

/// RF/IR command for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RfIrCommandInfo {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub category: String,
    #[serde(rename = "type")]
    pub signal_type: String,
}

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum WsMessage {
    StatusUpdate(SystemStatus),
    ObsStatusChanged(ObsStatus),
    StreamStateChanged { streaming: bool },
    RecordStateChanged { recording: bool },
    // RF/IR events
    RfIrCommandExecuted { slug: String, success: bool },
    RfIrCommandList { commands: Vec<RfIrCommandInfo> },
    // PPT events
    PptFoldersChanged { folders: Vec<PptFolder> },
    PptFileOpened { file_name: String, file_path: String, success: bool, presenter_started: bool },
    Ping,
    Pong,
    Error { message: String },
}

/// API response wrapper
#[derive(Debug, Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    fn error(message: impl Into<String>) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            error: Some(message.into()),
        }
    }
}

/// Custom JSON extractor that returns JSON-formatted errors instead of plain text
struct AppJson<T>(T);

impl<S, T> FromRequest<S> for AppJson<T>
where
    axum::Json<T>: FromRequest<S, Rejection = JsonRejection>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, axum::Json<ApiResponse<()>>);

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match axum::Json::<T>::from_request(req, state).await {
            Ok(value) => Ok(Self(value.0)),
            Err(rejection) => {
                let message = rejection.body_text();
                Err((
                    rejection.status(),
                    axum::Json(ApiResponse::<()>::error(message)),
                ))
            }
        }
    }
}

/// Stored RF/IR command data (subset of full command for API)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredRfIrCommand {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub device_host: String,
    pub device_mac: String,
    pub device_type: String,
    pub code: String,
    pub signal_type: String,
    pub category: String,
}

// ============================================================================
// PPT Folder/File Types
// ============================================================================

/// PPT folder configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PptFolder {
    pub id: String,
    pub path: String,
    pub name: String,
}

/// PPT file info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PptFile {
    pub id: String,
    pub name: String,
    pub path: String,
    pub folder_id: String,
}

/// PPT files response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PptFilesResponse {
    pub files: Vec<PptFile>,
    pub total: usize,
    pub filter: Option<String>,
}

/// Request to add a PPT folder
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddPptFolderRequest {
    pub path: String,
    pub name: String,
}

/// Request to open a PPT file
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenPptRequest {
    pub file_path: String,
    #[serde(default = "default_start_presenter")]
    pub start_presenter: bool,
}

fn default_start_presenter() -> bool {
    true
}

// ============================================================================
// Settings Export/Import Types
// ============================================================================

/// Query parameters for settings export endpoint
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsExportQuery {
    /// Include sensitive data like YouTube tokens (default: false)
    #[serde(default)]
    pub include_sensitive: bool,
}

/// Exported settings structure (matches TypeScript ExportedSettings)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportedSettings {
    pub schema_version: u32,
    pub exported_at: String,
    pub settings: serde_json::Value,
}

/// Request body for settings import
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportSettingsRequest {
    pub schema_version: u32,
    pub exported_at: String,
    pub settings: serde_json::Value,
}

/// Shared state for the discovery server
pub struct DiscoveryServerState {
    /// Current system status (updated by frontend)
    pub system_status: RwLock<SystemStatus>,
    /// Current OBS status
    pub obs_status: RwLock<ObsStatus>,
    /// Broadcast channel for WebSocket updates
    pub ws_broadcast: broadcast::Sender<WsMessage>,
    /// Optional auth token
    pub auth_token: Option<String>,
    /// Connected WebSocket client count
    pub connected_clients: RwLock<u32>,
    /// RF/IR commands (synced from frontend)
    pub rfir_commands: RwLock<Vec<StoredRfIrCommand>>,
    /// PPT folders (synced from frontend) - kept for WebSocket broadcasts
    pub ppt_folders: RwLock<Vec<PptFolder>>,
    /// App data directory for reading settings file directly
    pub app_data_dir: Option<std::path::PathBuf>,
}

impl DiscoveryServerState {
    pub fn new(auth_token: Option<String>, app_data_dir: Option<std::path::PathBuf>) -> Self {
        let (ws_broadcast, _) = broadcast::channel(100);
        Self {
            system_status: RwLock::new(SystemStatus::default()),
            obs_status: RwLock::new(ObsStatus::default()),
            ws_broadcast,
            auth_token,
            connected_clients: RwLock::new(0),
            rfir_commands: RwLock::new(Vec::new()),
            ppt_folders: RwLock::new(Vec::new()),
            app_data_dir,
        }
    }

    /// Read the entire settings file as JSON Value
    pub fn read_all_settings(&self) -> Option<serde_json::Value> {
        let data_dir = self.app_data_dir.as_ref()?;
        let settings_path = data_dir.join("app-settings.json");

        if !settings_path.exists() {
            return None;
        }

        match std::fs::read_to_string(&settings_path) {
            Ok(content) => serde_json::from_str(&content).ok(),
            Err(e) => {
                log::warn!("Failed to read settings file: {}", e);
                None
            }
        }
    }

    /// Write settings to the app settings file
    pub fn write_settings(&self, settings: &serde_json::Value) -> Result<(), String> {
        let data_dir = self.app_data_dir.as_ref()
            .ok_or_else(|| "App data directory not available".to_string())?;

        let settings_path = data_dir.join("app-settings.json");

        let content = serde_json::to_string_pretty(settings)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        std::fs::write(&settings_path, content)
            .map_err(|e| format!("Failed to write settings file: {}", e))?;

        Ok(())
    }

    /// Read PPT folders directly from the app settings file
    pub fn read_ppt_folders_from_settings(&self) -> Vec<PptFolder> {
        let Some(ref data_dir) = self.app_data_dir else {
            return Vec::new();
        };

        let settings_path = data_dir.join("app-settings.json");
        if !settings_path.exists() {
            return Vec::new();
        }

        match std::fs::read_to_string(&settings_path) {
            Ok(content) => {
                // Parse the JSON and extract pptSettings.folders
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(folders) = json.get("pptSettings").and_then(|s| s.get("folders")) {
                        if let Ok(folders) = serde_json::from_value::<Vec<PptFolder>>(folders.clone()) {
                            return folders;
                        }
                    }
                }
                Vec::new()
            }
            Err(e) => {
                log::warn!("Failed to read settings file: {}", e);
                Vec::new()
            }
        }
    }

    /// Read RF/IR commands directly from the app settings file
    pub fn read_rfir_commands_from_settings(&self) -> Vec<StoredRfIrCommand> {
        let Some(ref data_dir) = self.app_data_dir else {
            return Vec::new();
        };

        let settings_path = data_dir.join("app-settings.json");
        if !settings_path.exists() {
            return Vec::new();
        }

        match std::fs::read_to_string(&settings_path) {
            Ok(content) => {
                // Parse the JSON and extract rfIrSettings
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(rf_ir_settings) = json.get("rfIrSettings") {
                        let devices = rf_ir_settings
                            .get("devices")
                            .and_then(|d| d.as_array())
                            .cloned()
                            .unwrap_or_default();

                        let commands = rf_ir_settings
                            .get("commands")
                            .and_then(|c| c.as_array())
                            .cloned()
                            .unwrap_or_default();

                        // Build a map of device ID -> device info
                        let device_map: std::collections::HashMap<String, &serde_json::Value> = devices
                            .iter()
                            .filter_map(|d| {
                                d.get("id").and_then(|id| id.as_str()).map(|id| (id.to_string(), d))
                            })
                            .collect();

                        // Convert commands to StoredRfIrCommand format
                        return commands
                            .iter()
                            .filter_map(|cmd| {
                                let id = cmd.get("id")?.as_str()?;
                                let name = cmd.get("name")?.as_str()?;
                                let slug = cmd.get("slug")?.as_str()?;
                                let device_id = cmd.get("deviceId")?.as_str()?;
                                let code = cmd.get("code")?.as_str()?;
                                let signal_type = cmd.get("type")?.as_str()?;
                                let category = cmd.get("category")?.as_str().unwrap_or("other");

                                // Get device info
                                let device = device_map.get(device_id);
                                let device_host = device
                                    .and_then(|d| d.get("host"))
                                    .and_then(|h| h.as_str())
                                    .unwrap_or("");
                                let device_mac = device
                                    .and_then(|d| d.get("mac"))
                                    .and_then(|m| m.as_str())
                                    .unwrap_or("");
                                let device_type = device
                                    .and_then(|d| d.get("type"))
                                    .and_then(|t| t.as_str())
                                    .unwrap_or("");

                                Some(StoredRfIrCommand {
                                    id: id.to_string(),
                                    name: name.to_string(),
                                    slug: slug.to_string(),
                                    device_host: device_host.to_string(),
                                    device_mac: device_mac.to_string(),
                                    device_type: device_type.to_string(),
                                    code: code.to_string(),
                                    signal_type: signal_type.to_string(),
                                    category: category.to_string(),
                                })
                            })
                            .collect();
                    }
                }
                Vec::new()
            }
            Err(e) => {
                log::warn!("Failed to read settings file: {}", e);
                Vec::new()
            }
        }
    }

    /// Broadcast a message to all connected WebSocket clients
    pub fn broadcast(&self, message: WsMessage) {
        // Ignore send errors (no receivers)
        let _ = self.ws_broadcast.send(message);
    }
}

pub type SharedServerState = Arc<DiscoveryServerState>;

/// Discovery server handle
pub struct DiscoveryServer {
    pub port: u16,
    pub state: SharedServerState,
    mdns_service: Option<MdnsService>,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl DiscoveryServer {
    /// Start the discovery server
    pub async fn start(
        port: u16,
        auth_token: Option<String>,
        instance_name: &str,
        app_data_dir: Option<std::path::PathBuf>,
    ) -> Result<Self, String> {
        // Try the specified port first, then fallback to a random port
        let listener = match TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], port))).await {
            Ok(l) => l,
            Err(_) => {
                log::warn!("Port {} in use, trying random port", port);
                TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], 0)))
                    .await
                    .map_err(|e| format!("Failed to bind to port: {}", e))?
            }
        };

        let actual_port = listener
            .local_addr()
            .map_err(|e| format!("Failed to get local address: {}", e))?
            .port();

        // Create shared state with app data directory for reading settings
        let state = Arc::new(DiscoveryServerState::new(auth_token.clone(), app_data_dir));

        // Build CORS layer
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

        // Build the router
        let app = build_router(state.clone()).layer(cors);

        // Create shutdown channel
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

        // Spawn the server
        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = shutdown_rx.await;
                })
                .await
                .expect("Discovery server error");
        });

        log::info!("Discovery server started on port {}", actual_port);

        // Register mDNS service
        let mut properties = HashMap::new();
        properties.insert("version".to_string(), "1".to_string());
        properties.insert(
            "auth".to_string(),
            if auth_token.is_some() {
                "required"
            } else {
                "none"
            }
            .to_string(),
        );

        let mdns_service = match MdnsService::register(instance_name, actual_port, properties) {
            Ok(service) => {
                log::info!("mDNS service registered successfully");
                Some(service)
            }
            Err(e) => {
                log::warn!("Failed to register mDNS service: {}. Server will still work but won't be discoverable.", e);
                None
            }
        };

        Ok(Self {
            port: actual_port,
            state,
            mdns_service,
            shutdown_tx: Some(shutdown_tx),
        })
    }

    /// Stop the discovery server
    pub fn stop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
            log::info!("Discovery server stopped");
        }
    }

    /// Get server info for frontend
    pub fn get_info(&self) -> DiscoveryServerInfo {
        let addresses = get_local_addresses();
        // Use the first LAN address if available, otherwise localhost
        let host = get_categorized_addresses()
            .lan
            .first()
            .map(|n| n.address.clone())
            .unwrap_or_else(|| "localhost".to_string());
        DiscoveryServerInfo {
            running: true,
            port: self.port,
            addresses,
            service_name: self
                .mdns_service
                .as_ref()
                .map(|s| s.fullname().to_string())
                .unwrap_or_else(|| SERVICE_TYPE.to_string()),
            auth_required: self.state.auth_token.is_some(),
            docs_url: format!("http://{}:{}/api/docs", host, self.port),
        }
    }

    /// Get detailed server status
    pub async fn get_status(&self) -> DiscoveryServerStatus {
        let connected_clients = *self.state.connected_clients.read().await;
        let host = get_categorized_addresses()
            .lan
            .first()
            .map(|n| n.address.clone())
            .unwrap_or_else(|| "localhost".to_string());
        DiscoveryServerStatus {
            running: true,
            port: Some(self.port),
            addresses: get_local_addresses(),
            connected_clients,
            mdns_registered: self.mdns_service.is_some(),
            docs_url: Some(format!("http://{}:{}/api/docs", host, self.port)),
        }
    }

    /// Update system status and broadcast to WebSocket clients
    pub async fn update_system_status(&self, status: SystemStatus) {
        *self.state.system_status.write().await = status.clone();
        self.state.broadcast(WsMessage::StatusUpdate(status));
    }

    /// Update OBS status and broadcast to WebSocket clients
    pub async fn update_obs_status(&self, status: ObsStatus) {
        *self.state.obs_status.write().await = status.clone();
        self.state.broadcast(WsMessage::ObsStatusChanged(status));
    }
}

impl Drop for DiscoveryServer {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Build the Axum router with all endpoints
fn build_router(state: SharedServerState) -> Router {
    Router::new()
        // Health check
        .route("/api/v1/health", get(health_handler))
        // System status
        .route("/api/v1/status", get(status_handler))
        // OBS endpoints
        .route("/api/v1/obs/status", get(obs_status_handler))
        .route("/api/v1/obs/stream/start", post(obs_stream_start_handler))
        .route("/api/v1/obs/stream/stop", post(obs_stream_stop_handler))
        .route("/api/v1/obs/record/start", post(obs_record_start_handler))
        .route("/api/v1/obs/record/stop", post(obs_record_stop_handler))
        // RF/IR endpoints
        .route("/api/v1/rfir/commands", get(rfir_commands_handler))
        .route("/api/v1/rfir/commands/{slug}", get(rfir_command_by_slug_handler))
        .route("/api/v1/rfir/commands/{slug}/execute", post(rfir_execute_handler))
        // PPT endpoints
        .route("/api/v1/ppt/folders", get(ppt_folders_handler).post(ppt_add_folder_handler))
        .route("/api/v1/ppt/folders/{id}", axum::routing::delete(ppt_delete_folder_handler))
        .route("/api/v1/ppt/files", get(ppt_files_handler))
        .route("/api/v1/ppt/open", post(ppt_open_handler))
        // Settings export/import endpoints
        .route("/api/v1/settings/export", get(settings_export_handler))
        .route("/api/v1/settings/import", post(settings_import_handler))
        // OBS Caption endpoint (embeddable HTML for OBS browser source)
        .route("/caption", get(caption_handler))
        // OpenAPI documentation
        .route("/api/v1/openapi.json", get(openapi_handler))
        .route("/api/docs", get(swagger_ui_handler))
        // WebSocket
        .route("/ws", get(ws_handler))
        .with_state(state)
}

// ============================================================================
// HTTP Handlers
// ============================================================================

/// Health check endpoint
async fn health_handler() -> impl IntoResponse {
    Json(ApiResponse::success(serde_json::json!({
        "status": "ok",
        "service": "sermon-helper",
        "version": env!("CARGO_PKG_VERSION")
    })))
}

/// Get full system status
async fn status_handler(
    headers: HeaderMap,
    State(state): State<SharedServerState>,
) -> impl IntoResponse {
    if !check_auth(&headers, &state) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::<()>::error("Unauthorized")),
        )
            .into_response();
    }

    let status = state.system_status.read().await.clone();
    Json(ApiResponse::success(status)).into_response()
}

/// Get OBS status
async fn obs_status_handler(
    headers: HeaderMap,
    State(state): State<SharedServerState>,
) -> impl IntoResponse {
    if !check_auth(&headers, &state) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::<()>::error("Unauthorized")),
        )
            .into_response();
    }

    let status = state.obs_status.read().await.clone();
    Json(ApiResponse::success(status)).into_response()
}

/// Start OBS streaming
/// Note: Actual OBS control is done by the frontend via Tauri events
async fn obs_stream_start_handler(
    headers: HeaderMap,
    State(state): State<SharedServerState>,
) -> impl IntoResponse {
    if !check_auth(&headers, &state) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::<()>::error("Unauthorized")),
        )
            .into_response();
    }

    // This endpoint will be connected to OBS control via Tauri events
    // For now, return a placeholder response
    Json(ApiResponse::success(serde_json::json!({
        "action": "stream_start",
        "message": "Stream start command sent"
    })))
    .into_response()
}

/// Stop OBS streaming
async fn obs_stream_stop_handler(
    headers: HeaderMap,
    State(state): State<SharedServerState>,
) -> impl IntoResponse {
    if !check_auth(&headers, &state) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::<()>::error("Unauthorized")),
        )
            .into_response();
    }

    Json(ApiResponse::success(serde_json::json!({
        "action": "stream_stop",
        "message": "Stream stop command sent"
    })))
    .into_response()
}

/// Start OBS recording
async fn obs_record_start_handler(
    headers: HeaderMap,
    State(state): State<SharedServerState>,
) -> impl IntoResponse {
    if !check_auth(&headers, &state) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::<()>::error("Unauthorized")),
        )
            .into_response();
    }

    Json(ApiResponse::success(serde_json::json!({
        "action": "record_start",
        "message": "Record start command sent"
    })))
    .into_response()
}

/// Stop OBS recording
async fn obs_record_stop_handler(
    headers: HeaderMap,
    State(state): State<SharedServerState>,
) -> impl IntoResponse {
    if !check_auth(&headers, &state) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::<()>::error("Unauthorized")),
        )
            .into_response();
    }

    Json(ApiResponse::success(serde_json::json!({
        "action": "record_stop",
        "message": "Record stop command sent"
    })))
    .into_response()
}

// ============================================================================
// OBS Caption Handler
// ============================================================================

/// Query parameters for caption endpoint
#[derive(Debug, Deserialize)]
struct CaptionQuery {
    #[serde(rename = "type", default = "default_caption_type")]
    caption_type: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    bold: String,
    #[serde(default)]
    light: String,
    #[serde(default = "default_color")]
    color: String,
    #[serde(rename = "showLogo", default = "default_show_logo")]
    show_logo: String,
    #[serde(default)]
    logo: String,
    #[serde(default = "default_resolution")]
    resolution: String,
    /// Explicit width override (pixels)
    #[serde(default)]
    width: Option<u32>,
    /// Explicit height override (pixels)
    #[serde(default)]
    height: Option<u32>,
}

fn default_caption_type() -> String {
    "caption".to_string()
}

fn default_color() -> String {
    "black".to_string()
}

fn default_show_logo() -> String {
    "visible".to_string()
}

fn default_resolution() -> String {
    "1080p".to_string()
}

/// Generate embeddable caption HTML for OBS browser source
async fn caption_handler(
    axum::extract::Query(params): axum::extract::Query<CaptionQuery>,
) -> impl IntoResponse {
    // Resolution-based dimensions
    let (base_width, base_height) = match params.resolution.as_str() {
        "4k" => (3840u32, 2160u32),
        _ => (1920u32, 1080u32), // 1080p default
    };

    // Calculate dimensions
    let (width, height) = if let (Some(w), Some(h)) = (params.width, params.height) {
        (w, h)
    } else if params.caption_type == "full" {
        (base_width, base_height)
    } else {
        // Caption bar: ~14% of screen height
        let caption_height = (base_height as f32 * 0.14) as u32;
        (base_width, caption_height)
    };

    // Background and text colors
    let (bg_color, text_color, accent_color) = match params.color.as_str() {
        "white" => ("#ffffff", "#000000", "#dc2626"),
        "red" => ("#8B0000", "#ffffff", "#ffffff"),
        "blue" => ("#1a365d", "#ffffff", "#ffffff"),
        "green" => ("#1a4d1a", "#ffffff", "#ffffff"),
        _ => ("#000000", "#ffffff", "#dc2626"), // black default
    };

    let show_logo = params.show_logo == "visible";

    // Decode the SVG logo if provided (URL-encoded)
    let logo_svg = if show_logo && !params.logo.is_empty() {
        urlencoding::decode(&params.logo)
            .map(|s| s.into_owned())
            .unwrap_or_default()
    } else {
        String::new()
    };

    // Generate HTML based on caption type
    let html = if params.caption_type == "full" {
        // Full-screen service announcement style (v0 template design)
        let title_html = if !params.title.is_empty() {
            format!(r#"<h1 class="name-title">{}</h1>"#, html_escape(&params.title))
        } else {
            String::new()
        };

        // Service info with dot separator
        let service_info = if !params.bold.is_empty() || !params.light.is_empty() {
            let mut parts = Vec::new();
            if !params.bold.is_empty() {
                parts.push(format!("<span>{}</span>", html_escape(&params.bold).to_uppercase()));
            }
            if !params.bold.is_empty() && !params.light.is_empty() {
                parts.push(r#"<span class="dot"></span>"#.to_string());
            }
            if !params.light.is_empty() {
                parts.push(format!("<span>{}</span>", html_escape(&params.light).to_uppercase()));
            }
            format!(r#"<div class="service-info">{}</div>"#, parts.join(""))
        } else {
            String::new()
        };

        let logo_html = if show_logo && !logo_svg.is_empty() {
            format!(r#"<div class="logo-container">{}</div>"#, logo_svg)
        } else {
            String::new()
        };

        format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>OBS Caption</title>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}

        body {{
            width: {width}px;
            height: {height}px;
            overflow: hidden;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
            background: {bg_color};
            color: {text_color};
        }}

        .aspect-container {{
            width: 100%;
            height: 100%;
            display: flex;
            flex-direction: column;
            justify-content: space-between;
            padding: 5% 10%;
        }}

        .content {{
            display: flex;
            flex-direction: column;
        }}

        .name-title {{
            font-size: clamp(48px, 10vw, 180px);
            font-weight: 700;
            line-height: 1;
            margin: 0;
            padding: 0;
        }}

        .service-info {{
            color: {accent_color};
            font-size: clamp(24px, 3vw, 56px);
            font-weight: 700;
            margin-top: clamp(16px, 2vw, 40px);
            display: flex;
            align-items: center;
            gap: 0.5em;
        }}

        .dot {{
            display: inline-block;
            width: 0.5em;
            height: 0.5em;
            border-radius: 50%;
            background-color: {accent_color};
        }}

        .logo-container {{
            width: 100%;
            max-width: 400px;
            margin-top: auto;
        }}

        .logo-container svg {{
            width: 100%;
            height: auto;
        }}
    </style>
</head>
<body>
    <div class="aspect-container">
        <div class="content">
            {title_html}
            {service_info}
        </div>
        {logo_html}
    </div>
</body>
</html>"#)
    } else {
        // Caption bar style (original)
        let scale = if params.resolution == "4k" { 2.0 } else { 1.0 };
        let padding = (40.0 * scale) as u32;
        let gap = (30.0 * scale) as u32;
        let logo_height = (80.0 * scale) as u32;
        let logo_max_width = (120.0 * scale) as u32;
        let title_size = (36.0 * scale) as u32;
        let text_size = (28.0 * scale) as u32;
        let content_gap = (8.0 * scale) as u32;

        let logo_html = if show_logo && !logo_svg.is_empty() {
            format!(r#"<div class="logo">{}</div>"#, logo_svg)
        } else {
            String::new()
        };

        let title_html = if !params.title.is_empty() {
            format!(r#"<div class="title">{}</div>"#, html_escape(&params.title))
        } else {
            String::new()
        };

        let bold_html = if !params.bold.is_empty() {
            format!(r#"<span class="bold">{}</span>"#, html_escape(&params.bold))
        } else {
            String::new()
        };

        let light_html = if !params.light.is_empty() {
            format!(r#"<span class="light">{}</span>"#, html_escape(&params.light))
        } else {
            String::new()
        };

        let text_line = if !bold_html.is_empty() || !light_html.is_empty() {
            format!(r#"<div class="text-line">{}{}{}</div>"#,
                bold_html,
                if !bold_html.is_empty() && !light_html.is_empty() { " " } else { "" },
                light_html
            )
        } else {
            String::new()
        };

        format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>OBS Caption</title>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}

        body {{
            width: {width}px;
            height: {height}px;
            overflow: hidden;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
        }}

        .caption-container {{
            width: 100%;
            height: 100%;
            background-color: {bg_color};
            display: flex;
            align-items: center;
            padding: 0 {padding}px;
            gap: {gap}px;
        }}

        .logo {{
            flex-shrink: 0;
            display: flex;
            align-items: center;
            justify-content: center;
        }}

        .logo svg {{
            height: {logo_height}px;
            width: auto;
            max-width: {logo_max_width}px;
        }}

        .content {{
            flex: 1;
            display: flex;
            flex-direction: column;
            justify-content: center;
            gap: {content_gap}px;
            color: {text_color};
        }}

        .title {{
            font-size: {title_size}px;
            font-weight: 700;
            line-height: 1.2;
            text-transform: uppercase;
            letter-spacing: 1px;
        }}

        .text-line {{
            font-size: {text_size}px;
            line-height: 1.3;
        }}

        .bold {{
            font-weight: 600;
        }}

        .light {{
            font-weight: 300;
            opacity: 0.9;
        }}
    </style>
</head>
<body>
    <div class="caption-container">
        {logo_html}
        <div class="content">
            {title_html}
            {text_line}
        </div>
    </div>
</body>
</html>"#)
    };

    axum::response::Html(html)
}

/// Escape HTML special characters
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

// ============================================================================
// WebSocket Handler
// ============================================================================

/// WebSocket upgrade handler
async fn ws_handler(
    State(state): State<SharedServerState>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    // Check auth for WebSocket connections
    if !check_auth(&headers, &state) {
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }

    ws.on_upgrade(|socket| handle_websocket(socket, state))
        .into_response()
}

/// Handle WebSocket connection
async fn handle_websocket(mut socket: WebSocket, state: SharedServerState) {
    // Increment connected clients count
    {
        let mut count = state.connected_clients.write().await;
        *count += 1;
        log::info!("WebSocket client connected. Total: {}", *count);
    }

    // Subscribe to broadcast channel
    let mut rx = state.ws_broadcast.subscribe();

    // Send initial status
    let initial_status = state.system_status.read().await.clone();
    if let Ok(msg) = serde_json::to_string(&WsMessage::StatusUpdate(initial_status)) {
        let _ = socket.send(Message::Text(msg.into())).await;
    }

    // Create ping interval - send ping every 20 seconds to keep connection alive
    let mut ping_interval = tokio::time::interval(tokio::time::Duration::from_secs(20));
    ping_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            // Send periodic ping to keep connection alive
            _ = ping_interval.tick() => {
                if let Ok(ping_msg) = serde_json::to_string(&WsMessage::Ping) {
                    if socket.send(Message::Text(ping_msg.into())).await.is_err() {
                        break;
                    }
                }
            }
            // Handle broadcast messages
            Ok(broadcast_msg) = rx.recv() => {
                if let Ok(text) = serde_json::to_string(&broadcast_msg) {
                    if socket.send(Message::Text(text.into())).await.is_err() {
                        break;
                    }
                }
            }
            // Handle incoming messages from client
            Some(result) = socket.recv() => {
                match result {
                    Ok(Message::Text(text)) => {
                        // Try to parse as WsMessage
                        if let Ok(msg) = serde_json::from_str::<WsMessage>(&text) {
                            match msg {
                                WsMessage::Ping => {
                                    // Respond with pong directly
                                    if let Ok(pong) = serde_json::to_string(&WsMessage::Pong) {
                                        let _ = socket.send(Message::Text(pong.into())).await;
                                    }
                                }
                                WsMessage::Pong => {
                                    // Client responded to our ping, connection is alive
                                }
                                _ => {
                                    // Handle other messages as needed
                                }
                            }
                        }
                    }
                    Ok(Message::Close(_)) => break,
                    Err(_) => break,
                    _ => {}
                }
            }
            else => break,
        }
    }

    // Cleanup
    {
        let mut count = state.connected_clients.write().await;
        *count = count.saturating_sub(1);
        log::info!("WebSocket client disconnected. Total: {}", *count);
    }
}

// ============================================================================
// Auth Helper
// ============================================================================

/// Check authorization header
fn check_auth(headers: &HeaderMap, state: &SharedServerState) -> bool {
    match &state.auth_token {
        None => true, // No auth required
        Some(expected_token) => {
            headers
                .get(header::AUTHORIZATION)
                .and_then(|v| v.to_str().ok())
                .map(|auth| {
                    auth.strip_prefix("Bearer ")
                        .map(|token| token == expected_token)
                        .unwrap_or(false)
                })
                .unwrap_or(false)
        }
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Categorized network addresses for discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkAddresses {
    /// Localhost addresses (127.0.0.1) - only accessible from this computer
    pub localhost: Vec<String>,
    /// LAN addresses (192.168.x.x, 10.x.x.x, 172.16-31.x.x) - accessible from same network
    pub lan: Vec<NetworkInterface>,
    /// All addresses as flat list (for backward compatibility)
    pub all: Vec<String>,
}

/// Network interface with name and address
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInterface {
    pub name: String,
    pub address: String,
    pub is_primary: bool,
}

/// Get all local IP addresses, categorized by accessibility
pub fn get_local_addresses() -> Vec<String> {
    get_categorized_addresses().all
}

/// Get categorized network addresses
pub fn get_categorized_addresses() -> NetworkAddresses {
    let mut localhost = Vec::new();
    let mut lan = Vec::new();
    let mut all = Vec::new();

    // Always add localhost
    localhost.push("127.0.0.1".to_string());
    all.push("127.0.0.1".to_string());

    // Get primary local IP
    let primary_ip = local_ip_address::local_ip().ok().map(|ip| ip.to_string());

    // Get all network interfaces
    if let Ok(list) = local_ip_address::list_afinet_netifas() {
        for (name, ip) in list {
            let ip_str = ip.to_string();

            // Skip if already in the list
            if all.contains(&ip_str) {
                continue;
            }

            // Categorize the address
            if ip_str.starts_with("127.") {
                // Loopback addresses
                if !localhost.contains(&ip_str) {
                    localhost.push(ip_str.clone());
                }
            } else if is_lan_address(&ip_str) {
                // LAN addresses
                let is_primary = primary_ip.as_ref() == Some(&ip_str);
                lan.push(NetworkInterface {
                    name: name.clone(),
                    address: ip_str.clone(),
                    is_primary,
                });
            }

            // Add to all list (excluding loopback)
            if !ip_str.starts_with("127.") {
                all.push(ip_str);
            }
        }
    }

    // Sort LAN addresses: primary first, then by name
    lan.sort_by(|a, b| {
        match (a.is_primary, b.is_primary) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        }
    });

    NetworkAddresses { localhost, lan, all }
}

/// Check if an IP address is a LAN (private) address
fn is_lan_address(ip: &str) -> bool {
    // Parse IPv4 address
    let parts: Vec<u8> = ip.split('.')
        .filter_map(|s| s.parse().ok())
        .collect();

    if parts.len() != 4 {
        // Could be IPv6, check for link-local
        return ip.starts_with("fe80:") || ip.starts_with("fd");
    }

    // Check private IPv4 ranges (RFC 1918)
    match parts[0] {
        10 => true,                                    // 10.0.0.0/8
        172 => (16..=31).contains(&parts[1]),          // 172.16.0.0/12
        192 => parts[1] == 168,                        // 192.168.0.0/16
        169 => parts[1] == 254,                        // 169.254.0.0/16 (link-local)
        _ => false,
    }
}

/// Generate a random auth token
pub fn generate_auth_token() -> String {
    uuid::Uuid::new_v4().to_string()
}

// ============================================================================
// Shared Server Instance
// ============================================================================

/// Global discovery server instance
pub type SharedDiscoveryServer = Arc<Mutex<Option<DiscoveryServer>>>;

/// Create a new shared discovery server instance
pub fn create_shared_discovery_server() -> SharedDiscoveryServer {
    Arc::new(Mutex::new(None))
}

// ============================================================================
// RF/IR Handlers
// ============================================================================

/// Request body for updating RF/IR commands
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRfIrCommandsRequest {
    pub commands: Vec<StoredRfIrCommand>,
}

/// List all RF/IR commands
async fn rfir_commands_handler(
    headers: HeaderMap,
    State(state): State<SharedServerState>,
) -> impl IntoResponse {
    if !check_auth(&headers, &state) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::<()>::error("Unauthorized")),
        )
            .into_response();
    }

    // Read directly from settings file - no sync needed
    let commands = state.read_rfir_commands_from_settings();
    let command_infos: Vec<RfIrCommandInfo> = commands
        .iter()
        .map(|c| RfIrCommandInfo {
            id: c.id.clone(),
            name: c.name.clone(),
            slug: c.slug.clone(),
            category: c.category.clone(),
            signal_type: c.signal_type.clone(),
        })
        .collect();

    Json(ApiResponse::success(command_infos)).into_response()
}

/// Get a specific RF/IR command by slug
async fn rfir_command_by_slug_handler(
    headers: HeaderMap,
    State(state): State<SharedServerState>,
    axum::extract::Path(slug): axum::extract::Path<String>,
) -> impl IntoResponse {
    if !check_auth(&headers, &state) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::<()>::error("Unauthorized")),
        )
            .into_response();
    }

    // Read directly from settings file
    let commands = state.read_rfir_commands_from_settings();
    let command = commands.iter().find(|c| c.slug == slug);

    match command {
        Some(cmd) => Json(ApiResponse::success(RfIrCommandInfo {
            id: cmd.id.clone(),
            name: cmd.name.clone(),
            slug: cmd.slug.clone(),
            category: cmd.category.clone(),
            signal_type: cmd.signal_type.clone(),
        }))
        .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error(format!("Command not found: {}", slug))),
        )
            .into_response(),
    }
}

/// Execute an RF/IR command by slug
async fn rfir_execute_handler(
    headers: HeaderMap,
    State(state): State<SharedServerState>,
    axum::extract::Path(slug): axum::extract::Path<String>,
) -> impl IntoResponse {
    if !check_auth(&headers, &state) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::<()>::error("Unauthorized")),
        )
            .into_response();
    }

    // Read directly from settings file
    let commands = state.read_rfir_commands_from_settings();
    let command = commands.into_iter().find(|c| c.slug == slug);

    match command {
        Some(cmd) => {
            // Execute the command using the broadlink module
            match crate::broadlink::send_code(
                &cmd.device_host,
                &cmd.device_mac,
                &cmd.device_type,
                &cmd.code,
            )
            .await
            {
                Ok(result) => {
                    if result.success {
                        // Broadcast success to WebSocket clients
                        state.broadcast(WsMessage::RfIrCommandExecuted {
                            slug: cmd.slug.clone(),
                            success: true,
                        });

                        Json(ApiResponse::success(serde_json::json!({
                            "executed": true,
                            "command": cmd.name,
                            "slug": cmd.slug
                        })))
                        .into_response()
                    } else {
                        // Broadcast failure
                        state.broadcast(WsMessage::RfIrCommandExecuted {
                            slug: cmd.slug.clone(),
                            success: false,
                        });

                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ApiResponse::<()>::error(
                                result.error.unwrap_or_else(|| "Send failed".to_string()),
                            )),
                        )
                            .into_response()
                    }
                }
                Err(e) => {
                    state.broadcast(WsMessage::RfIrCommandExecuted {
                        slug: cmd.slug.clone(),
                        success: false,
                    });

                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::<()>::error(e)),
                    )
                        .into_response()
                }
            }
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error(format!("Command not found: {}", slug))),
        )
            .into_response(),
    }
}

// ============================================================================
// PPT Handlers
// ============================================================================

/// List all PPT folders
async fn ppt_folders_handler(
    headers: HeaderMap,
    State(state): State<SharedServerState>,
) -> impl IntoResponse {
    if !check_auth(&headers, &state) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::<()>::error("Unauthorized")),
        )
            .into_response();
    }

    // Read directly from settings file - no sync needed
    let folders = state.read_ppt_folders_from_settings();
    Json(ApiResponse::success(serde_json::json!({ "folders": folders }))).into_response()
}

/// Add a new PPT folder
async fn ppt_add_folder_handler(
    headers: HeaderMap,
    State(state): State<SharedServerState>,
    Json(request): Json<AddPptFolderRequest>,
) -> impl IntoResponse {
    if !check_auth(&headers, &state) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::<()>::error("Unauthorized")),
        )
            .into_response();
    }

    // Validate the path exists and is a directory
    let path = std::path::Path::new(&request.path);
    if !path.exists() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error("Path does not exist")),
        )
            .into_response();
    }
    if !path.is_dir() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error("Path is not a directory")),
        )
            .into_response();
    }

    let folder = PptFolder {
        id: uuid::Uuid::new_v4().to_string(),
        path: request.path,
        name: request.name,
    };

    let mut folders = state.ppt_folders.write().await;
    folders.push(folder.clone());
    let folders_clone = folders.clone();
    drop(folders);

    // Broadcast the change
    state.broadcast(WsMessage::PptFoldersChanged { folders: folders_clone });

    Json(ApiResponse::success(folder)).into_response()
}

/// Delete a PPT folder
async fn ppt_delete_folder_handler(
    headers: HeaderMap,
    State(state): State<SharedServerState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if !check_auth(&headers, &state) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::<()>::error("Unauthorized")),
        )
            .into_response();
    }

    let mut folders = state.ppt_folders.write().await;
    let original_len = folders.len();
    folders.retain(|f| f.id != id);

    if folders.len() == original_len {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Folder not found")),
        )
            .into_response();
    }

    let folders_clone = folders.clone();
    drop(folders);

    // Broadcast the change
    state.broadcast(WsMessage::PptFoldersChanged { folders: folders_clone });

    Json(ApiResponse::success(serde_json::json!({ "deleted": true }))).into_response()
}

/// Query parameters for PPT files endpoint
#[derive(Debug, Deserialize)]
struct PptFilesQuery {
    folder_id: String,
    #[serde(default)]
    filter: Option<String>,
}

/// List PPT files in a folder with optional numeric filter
async fn ppt_files_handler(
    headers: HeaderMap,
    State(state): State<SharedServerState>,
    axum::extract::Query(query): axum::extract::Query<PptFilesQuery>,
) -> impl IntoResponse {
    if !check_auth(&headers, &state) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::<()>::error("Unauthorized")),
        )
            .into_response();
    }

    // Read folders directly from settings file - no sync needed
    let folders = state.read_ppt_folders_from_settings();
    let folder = folders.into_iter().find(|f| f.id == query.folder_id);

    let folder = match folder {
        Some(f) => f,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error("Folder not found")),
            )
                .into_response();
        }
    };

    // Scan the folder for PPT files
    let files = match scan_ppt_folder(&folder.path, &folder.id) {
        Ok(f) => f,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(format!("Failed to scan folder: {}", e))),
            )
                .into_response();
        }
    };

    // Apply filter if provided (searches anywhere in filename)
    let filtered_files: Vec<PptFile> = if let Some(ref filter) = query.filter {
        files
            .into_iter()
            .filter(|f| f.name.contains(filter))
            .take(5) // Only return first 5 matches
            .collect()
    } else {
        files.into_iter().take(5).collect()
    };

    let total = filtered_files.len();

    Json(ApiResponse::success(PptFilesResponse {
        files: filtered_files,
        total,
        filter: query.filter,
    }))
    .into_response()
}

/// Scan a folder for PPT/PPTX/ODP files
fn scan_ppt_folder(folder_path: &str, folder_id: &str) -> Result<Vec<PptFile>, String> {
    let path = std::path::Path::new(folder_path);

    if !path.exists() {
        return Err("Folder does not exist".to_string());
    }

    let entries = std::fs::read_dir(path)
        .map_err(|e| format!("Failed to read directory: {}", e))?;

    let mut files: Vec<PptFile> = entries
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            if !path.is_file() {
                return None;
            }

            let extension = path.extension()?.to_str()?.to_lowercase();
            if !["ppt", "pptx", "odp"].contains(&extension.as_str()) {
                return None;
            }

            let name = path.file_name()?.to_str()?.to_string();
            let full_path = path.to_str()?.to_string();

            Some(PptFile {
                id: uuid::Uuid::new_v4().to_string(),
                name,
                path: full_path,
                folder_id: folder_id.to_string(),
            })
        })
        .collect();

    // Sort alphabetically by name
    files.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(files)
}

/// Open a PPT file and optionally start presenter mode
async fn ppt_open_handler(
    headers: HeaderMap,
    State(state): State<SharedServerState>,
    AppJson(request): AppJson<OpenPptRequest>,
) -> impl IntoResponse {
    if !check_auth(&headers, &state) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::<()>::error("Unauthorized")),
        )
            .into_response();
    }

    let path = std::path::Path::new(&request.file_path);
    if !path.exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("File not found")),
        )
            .into_response();
    }

    let file_name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    // Open the file with the system default application
    match open::that(&request.file_path) {
        Ok(_) => {
            let mut presenter_started = false;

            // If requested, start presenter mode after a delay
            if request.start_presenter {
                // Wait for the application to open
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

                // Try to start presenter mode
                presenter_started = start_presenter_mode().await;
            }

            // Broadcast the event
            state.broadcast(WsMessage::PptFileOpened {
                file_name: file_name.clone(),
                file_path: request.file_path.clone(),
                success: true,
                presenter_started,
            });

            Json(ApiResponse::success(serde_json::json!({
                "success": true,
                "file_name": file_name,
                "presenter_started": presenter_started
            })))
            .into_response()
        }
        Err(e) => {
            state.broadcast(WsMessage::PptFileOpened {
                file_name: file_name.clone(),
                file_path: request.file_path.clone(),
                success: false,
                presenter_started: false,
            });

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(format!("Failed to open file: {}", e))),
            )
                .into_response()
        }
    }
}

/// Start presenter mode by sending F5 keypress (Windows)
#[cfg(target_os = "windows")]
async fn start_presenter_mode() -> bool {
    use std::process::Command;

    // Use PowerShell to send F5 key to the active window
    let script = r#"
        Add-Type -AssemblyName System.Windows.Forms
        [System.Windows.Forms.SendKeys]::SendWait("{F5}")
    "#;

    match Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
        .output()
    {
        Ok(output) => output.status.success(),
        Err(e) => {
            log::error!("Failed to send F5 key: {}", e);
            false
        }
    }
}

/// Fallback for non-Windows platforms
#[cfg(not(target_os = "windows"))]
async fn start_presenter_mode() -> bool {
    log::warn!("Presenter mode automation not supported on this platform");
    false
}

// ============================================================================
// Settings Export/Import Handlers
// ============================================================================

/// Sensitive settings keys to exclude from export by default
const SENSITIVE_KEYS: &[&str] = &["youtubeTokens", "youtubeOAuthConfig"];

/// Export all settings as JSON
async fn settings_export_handler(
    headers: HeaderMap,
    State(state): State<SharedServerState>,
    axum::extract::Query(query): axum::extract::Query<SettingsExportQuery>,
) -> impl IntoResponse {
    if !check_auth(&headers, &state) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::<()>::error("Unauthorized")),
        )
            .into_response();
    }

    // Read all settings from the file
    let settings = match state.read_all_settings() {
        Some(s) => s,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Failed to read settings file")),
            )
                .into_response();
        }
    };

    // Optionally strip sensitive data
    let mut exported_settings = settings;
    if !query.include_sensitive {
        if let Some(obj) = exported_settings.as_object_mut() {
            for key in SENSITIVE_KEYS {
                obj.remove(*key);
            }
        }
    }

    let export_data = ExportedSettings {
        schema_version: 1,
        exported_at: Utc::now().to_rfc3339(),
        settings: exported_settings,
    };

    Json(ApiResponse::success(export_data)).into_response()
}

/// Import settings from JSON
async fn settings_import_handler(
    headers: HeaderMap,
    State(state): State<SharedServerState>,
    AppJson(request): AppJson<ImportSettingsRequest>,
) -> impl IntoResponse {
    if !check_auth(&headers, &state) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::<()>::error("Unauthorized")),
        )
            .into_response();
    }

    // Validate schema version
    if request.schema_version < 1 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error("Invalid schema version")),
        )
            .into_response();
    }

    // Validate that settings is an object
    if !request.settings.is_object() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error("Settings must be a JSON object")),
        )
            .into_response();
    }

    // Read existing settings to merge with imported ones
    let mut existing_settings = state.read_all_settings().unwrap_or_else(|| serde_json::json!({}));

    // Merge imported settings into existing (imported values take precedence)
    if let (Some(existing), Some(imported)) = (existing_settings.as_object_mut(), request.settings.as_object()) {
        for (key, value) in imported {
            existing.insert(key.clone(), value.clone());
        }
    }

    // Write merged settings back to file
    if let Err(e) = state.write_settings(&existing_settings) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Failed to save settings: {}", e))),
        )
            .into_response();
    }

    Json(ApiResponse::success(serde_json::json!({
        "imported": true,
        "message": "Settings imported successfully. Restart the app to apply all changes."
    })))
    .into_response()
}

// ============================================================================
// OpenAPI / Swagger Documentation
// ============================================================================

/// OpenAPI specification
async fn openapi_handler() -> impl IntoResponse {
    let spec = serde_json::json!({
        "openapi": "3.0.3",
        "info": {
            "title": "Sermon Helper API",
            "description": "REST API for controlling Sermon Helper from remote devices",
            "version": env!("CARGO_PKG_VERSION")
        },
        "servers": [
            {
                "url": "/api/v1",
                "description": "API v1"
            }
        ],
        "security": [
            {
                "bearerAuth": []
            }
        ],
        "paths": {
            "/health": {
                "get": {
                    "summary": "Health check",
                    "description": "Check if the server is running",
                    "tags": ["Health"],
                    "security": [],
                    "responses": {
                        "200": {
                            "description": "Server is healthy"
                        }
                    }
                }
            },
            "/status": {
                "get": {
                    "summary": "Get system status",
                    "description": "Get the current system status",
                    "tags": ["Status"],
                    "responses": {
                        "200": {
                            "description": "System status"
                        },
                        "401": {
                            "description": "Unauthorized"
                        }
                    }
                }
            },
            "/obs/status": {
                "get": {
                    "summary": "Get OBS status",
                    "description": "Get the current OBS connection and streaming status",
                    "tags": ["OBS"],
                    "responses": {
                        "200": {
                            "description": "OBS status"
                        }
                    }
                }
            },
            "/obs/stream/start": {
                "post": {
                    "summary": "Start streaming",
                    "description": "Start OBS streaming",
                    "tags": ["OBS"],
                    "responses": {
                        "200": {
                            "description": "Stream started"
                        }
                    }
                }
            },
            "/obs/stream/stop": {
                "post": {
                    "summary": "Stop streaming",
                    "description": "Stop OBS streaming",
                    "tags": ["OBS"],
                    "responses": {
                        "200": {
                            "description": "Stream stopped"
                        }
                    }
                }
            },
            "/obs/record/start": {
                "post": {
                    "summary": "Start recording",
                    "description": "Start OBS recording",
                    "tags": ["OBS"],
                    "responses": {
                        "200": {
                            "description": "Recording started"
                        }
                    }
                }
            },
            "/obs/record/stop": {
                "post": {
                    "summary": "Stop recording",
                    "description": "Stop OBS recording",
                    "tags": ["OBS"],
                    "responses": {
                        "200": {
                            "description": "Recording stopped"
                        }
                    }
                }
            },
            "/rfir/commands": {
                "get": {
                    "summary": "List RF/IR commands",
                    "description": "Get all configured RF/IR remote control commands",
                    "tags": ["RF/IR"],
                    "responses": {
                        "200": {
                            "description": "List of commands",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "array",
                                        "items": {
                                            "$ref": "#/components/schemas/RfIrCommand"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/rfir/commands/{slug}": {
                "get": {
                    "summary": "Get RF/IR command by slug",
                    "description": "Get a specific RF/IR command by its URL-safe slug",
                    "tags": ["RF/IR"],
                    "parameters": [
                        {
                            "name": "slug",
                            "in": "path",
                            "required": true,
                            "schema": {
                                "type": "string"
                            },
                            "description": "Command slug (e.g., 'projector-power-on')"
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "Command details"
                        },
                        "404": {
                            "description": "Command not found"
                        }
                    }
                }
            },
            "/rfir/commands/{slug}/execute": {
                "post": {
                    "summary": "Execute RF/IR command",
                    "description": "Send the IR/RF signal for the specified command",
                    "tags": ["RF/IR"],
                    "parameters": [
                        {
                            "name": "slug",
                            "in": "path",
                            "required": true,
                            "schema": {
                                "type": "string"
                            },
                            "description": "Command slug to execute"
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "Command executed successfully"
                        },
                        "404": {
                            "description": "Command not found"
                        },
                        "500": {
                            "description": "Failed to execute command"
                        }
                    }
                }
            },
            "/ppt/folders": {
                "get": {
                    "summary": "List PPT folders",
                    "description": "Get all configured PPT folders.\n\n**Example:**\n```bash\ncurl http://localhost:8765/api/v1/ppt/folders\n```",
                    "tags": ["PPT"],
                    "responses": {
                        "200": {
                            "description": "List of folders",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "success": { "type": "boolean" },
                                            "data": {
                                                "type": "object",
                                                "properties": {
                                                    "folders": {
                                                        "type": "array",
                                                        "items": { "$ref": "#/components/schemas/PptFolder" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                "post": {
                    "summary": "Add PPT folder",
                    "description": "Add a new folder to scan for PPT files.\n\n**Example:**\n```bash\ncurl -X POST http://localhost:8765/api/v1/ppt/folders \\\n  -H 'Content-Type: application/json' \\\n  -d '{\"path\": \"C:/Presentations\", \"name\": \"Main Folder\"}'\n```",
                    "tags": ["PPT"],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "required": ["path", "name"],
                                    "properties": {
                                        "path": { "type": "string", "description": "Folder path" },
                                        "name": { "type": "string", "description": "Display name" }
                                    }
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": { "description": "Folder added" },
                        "400": { "description": "Invalid path" }
                    }
                }
            },
            "/ppt/folders/{id}": {
                "delete": {
                    "summary": "Delete PPT folder",
                    "description": "Remove a folder from the list.\n\n**Example:**\n```bash\ncurl -X DELETE http://localhost:8765/api/v1/ppt/folders/FOLDER_ID\n```",
                    "tags": ["PPT"],
                    "parameters": [
                        {
                            "name": "id",
                            "in": "path",
                            "required": true,
                            "schema": { "type": "string" },
                            "description": "Folder ID"
                        }
                    ],
                    "responses": {
                        "200": { "description": "Folder deleted" },
                        "404": { "description": "Folder not found" }
                    }
                }
            },
            "/ppt/files": {
                "get": {
                    "summary": "List PPT files",
                    "description": "List PowerPoint files in a folder with optional filter. Filter searches anywhere in filename.\n\n**Example:**\n```bash\ncurl 'http://localhost:8765/api/v1/ppt/files?folder_id=FOLDER_ID&filter=01'\n```\nThis would match files like D-001.pptx, D-010.pptx, sermon-01.pptx, etc.",
                    "tags": ["PPT"],
                    "parameters": [
                        {
                            "name": "folder_id",
                            "in": "query",
                            "required": true,
                            "schema": { "type": "string" },
                            "description": "The folder ID to search in"
                        },
                        {
                            "name": "filter",
                            "in": "query",
                            "required": false,
                            "schema": { "type": "string" },
                            "description": "Filter string to match anywhere in filename"
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "List of matching PPT files (max 5)",
                            "content": {
                                "application/json": {
                                    "schema": { "$ref": "#/components/schemas/PptFilesResponse" }
                                }
                            }
                        },
                        "404": { "description": "Folder not found" }
                    }
                }
            },
            "/ppt/open": {
                "post": {
                    "summary": "Open PPT file",
                    "description": "Open a PowerPoint file and optionally start presenter mode.\n\n**Example:**\n```bash\ncurl -X POST http://localhost:8765/api/v1/ppt/open \\\n  -H 'Content-Type: application/json' \\\n  -d '{\"filePath\": \"C:/Presentations/D-001.pptx\", \"startPresenter\": true}'\n```",
                    "tags": ["PPT"],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "required": ["filePath"],
                                    "properties": {
                                        "filePath": { "type": "string", "description": "Full path to PPT file" },
                                        "startPresenter": { "type": "boolean", "default": true, "description": "Auto-start presenter mode (F5)" }
                                    }
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "File opened",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "success": { "type": "boolean" },
                                            "file_name": { "type": "string" },
                                            "presenter_started": { "type": "boolean" }
                                        }
                                    }
                                }
                            }
                        },
                        "404": { "description": "File not found" }
                    }
                }
            },
            "/settings/export": {
                "get": {
                    "summary": "Export settings",
                    "description": "Export all app settings as JSON. Sensitive data (YouTube tokens) is excluded by default.\n\n**Example:**\n```bash\n# Export settings (excluding sensitive data)\ncurl http://localhost:8765/api/v1/settings/export -o settings.json\n\n# Export with sensitive data included\ncurl 'http://localhost:8765/api/v1/settings/export?includeSensitive=true' -o settings.json\n```",
                    "tags": ["Settings"],
                    "parameters": [
                        {
                            "name": "includeSensitive",
                            "in": "query",
                            "required": false,
                            "schema": { "type": "boolean", "default": false },
                            "description": "Include sensitive data like YouTube OAuth tokens"
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "Exported settings",
                            "content": {
                                "application/json": {
                                    "schema": { "$ref": "#/components/schemas/ExportedSettings" }
                                }
                            }
                        },
                        "401": { "description": "Unauthorized" },
                        "500": { "description": "Failed to read settings" }
                    }
                }
            },
            "/settings/import": {
                "post": {
                    "summary": "Import settings",
                    "description": "Import settings from a previously exported JSON file. Settings are merged with existing values (imported values take precedence).\n\n**Example:**\n```bash\ncurl -X POST http://localhost:8765/api/v1/settings/import \\\n  -H 'Content-Type: application/json' \\\n  -d @settings.json\n```\n\n**Note:** After importing, restart the app to apply all changes.",
                    "tags": ["Settings"],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": { "$ref": "#/components/schemas/ExportedSettings" }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Settings imported successfully",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "success": { "type": "boolean" },
                                            "data": {
                                                "type": "object",
                                                "properties": {
                                                    "imported": { "type": "boolean" },
                                                    "message": { "type": "string" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        "400": { "description": "Invalid settings format" },
                        "401": { "description": "Unauthorized" },
                        "500": { "description": "Failed to save settings" }
                    }
                }
            }
        },
        "x-websocket": {
            "/ws": {
                "description": "WebSocket endpoint for real-time updates. Connect with `ws://HOST:PORT/ws`.\n\n**Connection Example (JavaScript):**\n```javascript\nconst ws = new WebSocket('ws://localhost:8765/ws');\nws.onmessage = (event) => console.log(JSON.parse(event.data));\n```\n\n**Message Types (server → client):**\n- `status_update` - System status changed\n- `obs_status_changed` - OBS connection/streaming status\n- `stream_state_changed` - Streaming started/stopped\n- `record_state_changed` - Recording started/stopped\n- `rfir_command_executed` - RF/IR command result\n- `rfir_command_list` - Updated command list\n- `ppt_folders_changed` - PPT folders updated\n- `ppt_file_opened` - PPT file opened result\n- `ping` / `pong` - Keep-alive\n\n**Example Message:**\n```json\n{\"type\": \"status_update\", \"data\": {\"obsConnected\": true, \"obsStreaming\": false}}\n```"
            }
        },
        "components": {
            "securitySchemes": {
                "bearerAuth": {
                    "type": "http",
                    "scheme": "bearer",
                    "description": "Bearer token authentication"
                }
            },
            "schemas": {
                "RfIrCommand": {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Unique identifier"
                        },
                        "name": {
                            "type": "string",
                            "description": "Display name"
                        },
                        "slug": {
                            "type": "string",
                            "description": "URL-safe identifier for API access"
                        },
                        "category": {
                            "type": "string",
                            "description": "Command category (projector, screen, hvac, etc.)"
                        },
                        "type": {
                            "type": "string",
                            "enum": ["ir", "rf"],
                            "description": "Signal type"
                        }
                    }
                },
                "SystemStatus": {
                    "type": "object",
                    "properties": {
                        "obsConnected": { "type": "boolean" },
                        "obsStreaming": { "type": "boolean" },
                        "obsRecording": { "type": "boolean" },
                        "rodeInterface": { "type": "boolean" },
                        "mainDisplay": { "type": "boolean" },
                        "secondaryDisplay": { "type": "boolean" },
                        "youtubeLoggedIn": { "type": "boolean" }
                    }
                },
                "ObsStatus": {
                    "type": "object",
                    "properties": {
                        "connected": { "type": "boolean" },
                        "streaming": { "type": "boolean" },
                        "recording": { "type": "boolean" },
                        "streamTimecode": { "type": "string", "nullable": true },
                        "recordTimecode": { "type": "string", "nullable": true }
                    }
                },
                "PptFolder": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "string", "description": "Unique folder identifier" },
                        "path": { "type": "string", "description": "Folder path on disk" },
                        "name": { "type": "string", "description": "Display name" }
                    }
                },
                "PptFile": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "string", "description": "Unique file identifier" },
                        "name": { "type": "string", "description": "Filename (e.g., 'D-001.pptx')" },
                        "path": { "type": "string", "description": "Full file path" },
                        "folderId": { "type": "string", "description": "Parent folder ID" }
                    }
                },
                "PptFilesResponse": {
                    "type": "object",
                    "properties": {
                        "files": {
                            "type": "array",
                            "items": { "$ref": "#/components/schemas/PptFile" },
                            "description": "Matching files (max 5)"
                        },
                        "total": { "type": "integer", "description": "Number of files returned" },
                        "filter": { "type": "string", "nullable": true, "description": "The filter that was applied" }
                    }
                },
                "ExportedSettings": {
                    "type": "object",
                    "required": ["schemaVersion", "exportedAt", "settings"],
                    "properties": {
                        "schemaVersion": { "type": "integer", "description": "Schema version for migration support", "example": 1 },
                        "exportedAt": { "type": "string", "format": "date-time", "description": "ISO 8601 timestamp of export" },
                        "settings": {
                            "type": "object",
                            "description": "App settings object containing all configuration",
                            "properties": {
                                "bibleTranslation": { "type": "string" },
                                "eventList": { "type": "array", "items": { "type": "object" } },
                                "obsDevicesSettings": { "type": "object" },
                                "discoverySettings": { "type": "object" },
                                "rfIrSettings": { "type": "object" },
                                "pptSettings": { "type": "object" },
                                "uploadSettings": { "type": "object" }
                            }
                        }
                    }
                }
            }
        }
    });

    Json(spec)
}

/// Swagger UI HTML page
async fn swagger_ui_handler() -> impl IntoResponse {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Sermon Helper API Documentation</title>
    <link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css">
    <style>
        body { margin: 0; }
        .swagger-ui .topbar { display: none; }
    </style>
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
    <script>
        window.onload = function() {
            SwaggerUIBundle({
                url: "/api/v1/openapi.json",
                dom_id: '#swagger-ui',
                presets: [
                    SwaggerUIBundle.presets.apis,
                    SwaggerUIBundle.SwaggerUIStandalonePreset
                ],
                layout: "BaseLayout",
                persistAuthorization: true
            });
        };
    </script>
</body>
</html>"#;

    axum::response::Html(html)
}

// ============================================================================
// RF/IR Commands Sync
// ============================================================================

impl DiscoveryServer {
    /// Update the RF/IR commands from the frontend
    pub async fn update_rfir_commands(&self, commands: Vec<StoredRfIrCommand>) {
        *self.state.rfir_commands.write().await = commands.clone();

        // Broadcast the updated command list
        let command_infos: Vec<RfIrCommandInfo> = commands
            .iter()
            .map(|c| RfIrCommandInfo {
                id: c.id.clone(),
                name: c.name.clone(),
                slug: c.slug.clone(),
                category: c.category.clone(),
                signal_type: c.signal_type.clone(),
            })
            .collect();

        self.state.broadcast(WsMessage::RfIrCommandList {
            commands: command_infos,
        });
    }

    /// Update the PPT folders from the frontend
    pub async fn update_ppt_folders(&self, folders: Vec<PptFolder>) {
        *self.state.ppt_folders.write().await = folders.clone();

        // Broadcast the updated folder list
        self.state.broadcast(WsMessage::PptFoldersChanged { folders });
    }

    /// Get current PPT folders
    pub async fn get_ppt_folders(&self) -> Vec<PptFolder> {
        self.state.ppt_folders.read().await.clone()
    }
}
