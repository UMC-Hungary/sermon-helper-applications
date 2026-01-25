//! Tauri commands for Broadlink RF/IR device control

use crate::broadlink::{self, DiscoveredDevice, LearnResult, SendResult};

/// Discover Broadlink devices on the network
#[tauri::command]
pub async fn broadlink_discover(timeout: Option<u32>) -> Result<Vec<DiscoveredDevice>, String> {
    let timeout = timeout.unwrap_or(5);
    broadlink::discover_devices(timeout).await
}

/// Enter learning mode on a device
#[tauri::command]
pub async fn broadlink_learn(
    host: String,
    mac: String,
    devtype: String,
    signal_type: String,
) -> Result<LearnResult, String> {
    broadlink::learn_code(&host, &mac, &devtype, &signal_type).await
}

/// Cancel ongoing learning operation
#[tauri::command]
pub async fn broadlink_cancel_learn() -> Result<(), String> {
    broadlink::cancel_learn().await;
    Ok(())
}

/// Send an IR/RF code to a device
#[tauri::command]
pub async fn broadlink_send(
    host: String,
    mac: String,
    devtype: String,
    code: String,
) -> Result<SendResult, String> {
    broadlink::send_code(&host, &mac, &devtype, &code).await
}

/// Test if a device is reachable
#[tauri::command]
pub async fn broadlink_test_device(
    host: String,
    mac: String,
    devtype: String,
) -> Result<bool, String> {
    broadlink::test_device(&host, &mac, &devtype).await
}
