//! Reusable local HTTP server for handling callbacks and local API requests.
//!
//! This module provides a flexible local HTTP server that can be used for:
//! - OAuth callbacks (Google, etc.)
//! - Local network interface discovery
//! - Any other local HTTP endpoints needed by the app

use axum::{
    extract::Query,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::net::TcpListener;
use tokio::sync::{oneshot, Mutex};

/// Result of an OAuth callback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthCallbackResult {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

/// Server handle that allows stopping the server and getting results
pub struct ServerHandle {
    pub port: u16,
    pub shutdown_tx: oneshot::Sender<()>,
}

/// Shared state for the OAuth server
struct OAuthServerState {
    result_tx: Option<oneshot::Sender<OAuthCallbackResult>>,
    app_handle: Option<AppHandle>,
}

/// Query parameters for OAuth callback
#[derive(Debug, Deserialize)]
struct OAuthQuery {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

/// Start a one-shot OAuth callback server.
/// Returns the port and a receiver for the OAuth result.
/// The server automatically shuts down after receiving the callback.
pub async fn start_oauth_server(app_handle: Option<AppHandle>) -> Result<(u16, oneshot::Receiver<OAuthCallbackResult>), String> {
    // Find an available port
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| format!("Failed to bind to port: {}", e))?;

    let port = listener
        .local_addr()
        .map_err(|e| format!("Failed to get local address: {}", e))?
        .port();

    // Create channel for OAuth result
    let (result_tx, result_rx) = oneshot::channel();

    // Create shared state
    let state = Arc::new(Mutex::new(OAuthServerState {
        result_tx: Some(result_tx),
        app_handle,
    }));

    // Build the router
    let app = Router::new()
        .route("/callback", get(oauth_callback_handler))
        .route("/", get(health_check))
        .with_state(state.clone());

    // Spawn the server
    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("OAuth server error");
    });

    Ok((port, result_rx))
}

/// OAuth callback handler
async fn oauth_callback_handler(
    Query(params): Query<OAuthQuery>,
    axum::extract::State(state): axum::extract::State<Arc<Mutex<OAuthServerState>>>,
) -> impl IntoResponse {
    let result = OAuthCallbackResult {
        code: params.code,
        state: params.state,
        error: params.error,
        error_description: params.error_description,
    };

    // Send result through channel and emit event
    let mut state_guard = state.lock().await;

    // Emit event to frontend
    if let Some(ref app_handle) = state_guard.app_handle {
        let _ = app_handle.emit("oauth-callback", result.clone());
    }

    // Also send through channel for blocking API
    if let Some(tx) = state_guard.result_tx.take() {
        let _ = tx.send(result.clone());
    }

    // Return a nice HTML page that auto-closes
    let html = if result.error.is_some() {
        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Authentication Failed</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
               display: flex; justify-content: center; align-items: center; height: 100vh;
               margin: 0; background: #fef2f2; }}
        .container {{ text-align: center; padding: 40px; background: white; border-radius: 12px;
                     box-shadow: 0 4px 6px rgba(0,0,0,0.1); max-width: 400px; }}
        h1 {{ color: #dc2626; margin-bottom: 16px; }}
        p {{ color: #6b7280; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>Authentication Failed</h1>
        <p>{}</p>
        <p>You can close this window and try again.</p>
    </div>
</body>
</html>"#,
            result.error_description.as_deref().unwrap_or("Unknown error")
        )
    } else {
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Authentication Successful</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
               display: flex; justify-content: center; align-items: center; height: 100vh;
               margin: 0; background: #f0fdf4; }
        .container { text-align: center; padding: 40px; background: white; border-radius: 12px;
                     box-shadow: 0 4px 6px rgba(0,0,0,0.1); max-width: 400px; }
        h1 { color: #16a34a; margin-bottom: 16px; }
        p { color: #6b7280; }
        .checkmark { font-size: 48px; margin-bottom: 16px; }
    </style>
</head>
<body>
    <div class="container">
        <div class="checkmark">✓</div>
        <h1>Authentication Successful</h1>
        <p>You can close this window and return to the app.</p>
    </div>
    <script>
        // Try to close the window after a short delay
        setTimeout(() => { window.close(); }, 2000);
    </script>
</body>
</html>"#
            .to_string()
    };

    Html(html)
}

/// Simple health check endpoint
async fn health_check() -> &'static str {
    "Sermon Helper OAuth Server"
}

// ============================================================================
// Generic Local Server (for future use cases)
// ============================================================================

/// Configuration for a generic local server
#[derive(Clone)]
pub struct LocalServerConfig {
    /// Optional specific port (0 for random)
    pub port: u16,
    /// Routes to register
    pub routes: Vec<LocalServerRoute>,
}

/// A route definition for the local server
#[derive(Clone)]
pub struct LocalServerRoute {
    pub path: String,
    pub method: HttpMethod,
    pub handler_type: RouteHandler,
}

#[derive(Clone)]
pub enum HttpMethod {
    Get,
    Post,
}

#[derive(Clone)]
pub enum RouteHandler {
    /// Return a static JSON response
    StaticJson(String),
    /// Return static HTML
    StaticHtml(String),
    /// Health check
    HealthCheck,
}

/// Start a generic local HTTP server with custom routes
pub async fn start_local_server(config: LocalServerConfig) -> Result<ServerHandle, String> {
    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));

    let listener = TcpListener::bind(addr)
        .await
        .map_err(|e| format!("Failed to bind to port: {}", e))?;

    let port = listener
        .local_addr()
        .map_err(|e| format!("Failed to get local address: {}", e))?
        .port();

    // Build router with configured routes
    let mut router = Router::new();

    for route in config.routes {
        let handler = route.handler_type.clone();
        match (route.method, handler) {
            (HttpMethod::Get, RouteHandler::StaticJson(json)) => {
                router = router.route(
                    &route.path,
                    get(move || async move {
                        (
                            [(axum::http::header::CONTENT_TYPE, "application/json")],
                            json.clone(),
                        )
                    }),
                );
            }
            (HttpMethod::Get, RouteHandler::StaticHtml(html)) => {
                router = router.route(&route.path, get(move || async move { Html(html.clone()) }));
            }
            (HttpMethod::Get, RouteHandler::HealthCheck) => {
                router = router.route(&route.path, get(|| async { "OK" }));
            }
            _ => {
                // Add more handlers as needed
            }
        }
    }

    // Create shutdown channel
    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    // Spawn the server
    tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
            })
            .await
            .expect("Local server error");
    });

    Ok(ServerHandle { port, shutdown_tx })
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// Start OAuth callback server and return the port
/// The server will emit an "oauth-callback" event when the callback is received
#[tauri::command]
pub async fn start_oauth_callback_server(app_handle: AppHandle) -> Result<u16, String> {
    let (port, _rx) = start_oauth_server(Some(app_handle)).await?;
    Ok(port)
}

/// Start OAuth flow and wait for the callback result (blocking version)
#[tauri::command]
pub async fn start_oauth_flow_with_callback(app_handle: AppHandle) -> Result<OAuthCallbackResult, String> {
    let (port, rx) = start_oauth_server(Some(app_handle)).await?;

    // Note: The frontend should open the browser with the OAuth URL using this port
    // This command will block until the callback is received

    // Wait for the callback (with timeout)
    let result = tokio::time::timeout(std::time::Duration::from_secs(300), rx)
        .await
        .map_err(|_| "OAuth timeout - no callback received within 5 minutes".to_string())?
        .map_err(|_| "OAuth callback channel closed".to_string())?;

    Ok(result)
}

/// Get the OAuth redirect URI for a given port
#[tauri::command]
pub fn get_oauth_redirect_uri(port: u16) -> String {
    format!("http://127.0.0.1:{}/callback", port)
}
