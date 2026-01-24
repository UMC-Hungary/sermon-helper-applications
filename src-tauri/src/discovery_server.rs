//! Discovery server for mobile companion app integration.
//!
//! This module provides:
//! - mDNS/DNS-SD service registration for network discovery
//! - HTTP REST API for system status and control
//! - WebSocket for real-time status updates

use crate::mdns_service::{MdnsService, SERVICE_TYPE};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
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

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum WsMessage {
    StatusUpdate(SystemStatus),
    ObsStatusChanged(ObsStatus),
    StreamStateChanged { streaming: bool },
    RecordStateChanged { recording: bool },
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
}

impl DiscoveryServerState {
    pub fn new(auth_token: Option<String>) -> Self {
        let (ws_broadcast, _) = broadcast::channel(100);
        Self {
            system_status: RwLock::new(SystemStatus::default()),
            obs_status: RwLock::new(ObsStatus::default()),
            ws_broadcast,
            auth_token,
            connected_clients: RwLock::new(0),
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

        // Create shared state
        let state = Arc::new(DiscoveryServerState::new(auth_token.clone()));

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
        }
    }

    /// Get detailed server status
    pub async fn get_status(&self) -> DiscoveryServerStatus {
        let connected_clients = *self.state.connected_clients.read().await;
        DiscoveryServerStatus {
            running: true,
            port: Some(self.port),
            addresses: get_local_addresses(),
            connected_clients,
            mdns_registered: self.mdns_service.is_some(),
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

    loop {
        tokio::select! {
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
