//! Tauri commands for the discovery server.
//!
//! These commands allow the frontend to control the mDNS discovery server
//! and update status that gets broadcast to connected mobile clients.

use crate::discovery_server::{
    create_shared_discovery_server, generate_auth_token, get_categorized_addresses,
    get_local_addresses, DiscoveryServer, DiscoveryServerInfo, DiscoveryServerStatus,
    NetworkAddresses, ObsStatus, SharedDiscoveryServer, StoredRfIrCommand, SystemStatus,
};
use std::sync::OnceLock;
use tauri::{AppHandle, Emitter};

/// Global discovery server instance
static DISCOVERY_SERVER: OnceLock<SharedDiscoveryServer> = OnceLock::new();

/// Get the global discovery server instance
fn get_server() -> &'static SharedDiscoveryServer {
    DISCOVERY_SERVER.get_or_init(create_shared_discovery_server)
}

/// Start the discovery server
#[tauri::command]
pub async fn start_discovery_server(
    app_handle: AppHandle,
    port: Option<u16>,
    auth_token: Option<String>,
    instance_name: Option<String>,
) -> Result<DiscoveryServerInfo, String> {
    let server_lock = get_server();
    let mut server_guard = server_lock.lock().await;

    // Check if already running
    if server_guard.is_some() {
        return Err("Discovery server is already running".to_string());
    }

    let port = port.unwrap_or(crate::discovery_server::DEFAULT_PORT);
    let instance_name = instance_name.unwrap_or_else(|| "Sermon Helper".to_string());

    // Start the server
    let server = DiscoveryServer::start(port, auth_token, &instance_name).await?;
    let info = server.get_info();

    // Store the server instance
    *server_guard = Some(server);

    // Emit event to frontend
    let _ = app_handle.emit("discovery-server-started", &info);

    log::info!("Discovery server started: {:?}", info);
    Ok(info)
}

/// Stop the discovery server
#[tauri::command]
pub async fn stop_discovery_server(app_handle: AppHandle) -> Result<(), String> {
    let server_lock = get_server();
    let mut server_guard = server_lock.lock().await;

    if let Some(mut server) = server_guard.take() {
        server.stop();
        let _ = app_handle.emit("discovery-server-stopped", ());
        log::info!("Discovery server stopped");
        Ok(())
    } else {
        Err("Discovery server is not running".to_string())
    }
}

/// Get the current status of the discovery server
#[tauri::command]
pub async fn get_discovery_server_status() -> Result<DiscoveryServerStatus, String> {
    let server_lock = get_server();
    let server_guard = server_lock.lock().await;

    if let Some(ref server) = *server_guard {
        Ok(server.get_status().await)
    } else {
        Ok(DiscoveryServerStatus {
            running: false,
            port: None,
            addresses: get_local_addresses(),
            connected_clients: 0,
            mdns_registered: false,
        })
    }
}

/// Generate a new random auth token
#[tauri::command]
pub fn generate_discovery_auth_token() -> String {
    generate_auth_token()
}

/// Get all local IP addresses (flat list)
#[tauri::command]
pub fn get_local_ip_addresses() -> Vec<String> {
    get_local_addresses()
}

/// Get categorized network addresses
#[tauri::command]
pub fn get_network_addresses() -> NetworkAddresses {
    get_categorized_addresses()
}

/// Update the system status (called by frontend when status changes)
/// This broadcasts the new status to all connected WebSocket clients
#[tauri::command]
pub async fn update_discovery_system_status(status: SystemStatus) -> Result<(), String> {
    let server_lock = get_server();
    let server_guard = server_lock.lock().await;

    if let Some(ref server) = *server_guard {
        server.update_system_status(status).await;
        Ok(())
    } else {
        // Server not running, ignore silently
        Ok(())
    }
}

/// Update the OBS status (called by frontend when OBS status changes)
/// This broadcasts the new status to all connected WebSocket clients
#[tauri::command]
pub async fn update_discovery_obs_status(status: ObsStatus) -> Result<(), String> {
    let server_lock = get_server();
    let server_guard = server_lock.lock().await;

    if let Some(ref server) = *server_guard {
        server.update_obs_status(status).await;
        Ok(())
    } else {
        // Server not running, ignore silently
        Ok(())
    }
}

/// Update RF/IR commands (called by frontend when commands change)
/// This syncs the commands to the discovery server for API access
#[tauri::command]
pub async fn update_discovery_rfir_commands(commands: Vec<StoredRfIrCommand>) -> Result<(), String> {
    let server_lock = get_server();
    let server_guard = server_lock.lock().await;

    if let Some(ref server) = *server_guard {
        server.update_rfir_commands(commands).await;
        Ok(())
    } else {
        // Server not running, ignore silently
        Ok(())
    }
}
