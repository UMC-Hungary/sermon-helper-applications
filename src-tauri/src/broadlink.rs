//! Broadlink RF/IR device integration module
//!
//! This module provides integration with Broadlink devices (RM4, RM Pro, etc.)
//! for sending and learning IR/RF codes using the native Rust rbroadlink crate.

use rbroadlink::Device;
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;
use std::sync::atomic::{AtomicBool, Ordering};

/// Discovered Broadlink device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredDevice {
    #[serde(rename = "type")]
    pub device_type: String,
    pub model: String,
    pub host: String,
    pub mac: String,
    pub name: String,
}

/// Format MAC address bytes as colon-separated hex string
fn format_mac(mac: &[u8; 6]) -> String {
    format!(
        "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
    )
}

/// Result of a learning operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnResult {
    pub code: Option<String>,
    pub error: Option<String>,
}

/// Result of a send operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendResult {
    pub success: bool,
    pub error: Option<String>,
}

/// Global state for managing learn cancellation
static LEARN_CANCEL: AtomicBool = AtomicBool::new(false);

/// Discover Broadlink devices on the network
pub async fn discover_devices(_timeout: u32) -> Result<Vec<DiscoveredDevice>, String> {
    tokio::task::spawn_blocking(|| {
        match Device::list(None) {
            Ok(devices) => {
                let mut discovered = Vec::new();

                for device in devices {
                    match device {
                        Device::Remote { ref remote } => {
                            let info = &remote.info;
                            discovered.push(DiscoveredDevice {
                                device_type: info.friendly_type.clone(),
                                model: info.friendly_model.clone(),
                                host: info.address.to_string(),
                                mac: format_mac(&info.mac),
                                name: if info.name.is_empty() {
                                    info.friendly_model.clone()
                                } else {
                                    info.name.clone()
                                },
                            });
                        }
                        Device::Hvac { ref hvac } => {
                            let info = &hvac.info;
                            discovered.push(DiscoveredDevice {
                                device_type: info.friendly_type.clone(),
                                model: info.friendly_model.clone(),
                                host: info.address.to_string(),
                                mac: format_mac(&info.mac),
                                name: if info.name.is_empty() {
                                    info.friendly_model.clone()
                                } else {
                                    info.name.clone()
                                },
                            });
                        }
                    }
                }

                Ok(discovered)
            }
            Err(e) => Err(format!("Discovery failed: {}", e)),
        }
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Enter learning mode and wait for IR/RF signal
pub async fn learn_code(
    host: &str,
    _mac: &str,
    _devtype: &str,
    signal_type: &str,
) -> Result<LearnResult, String> {
    // Reset cancellation flag
    LEARN_CANCEL.store(false, Ordering::SeqCst);

    let host = host.to_string();
    let signal_type = signal_type.to_string();

    tokio::task::spawn_blocking(move || {
        // Parse the IP address
        let ip: Ipv4Addr = host
            .parse()
            .map_err(|e| format!("Invalid IP address '{}': {}", host, e))?;

        // Connect to the device
        let device = Device::from_ip(ip, None)
            .map_err(|e| format!("Failed to connect to device: {}", e))?;

        // Get the remote device
        let remote = match device {
            Device::Remote { remote } => remote,
            _ => return Err("Device is not a remote control".to_string()),
        };

        // Learn based on signal type
        let code_result = if signal_type == "rf" {
            // RF learning is a two-step process
            remote.learn_rf()
        } else {
            // Default to IR
            remote.learn_ir()
        };

        match code_result {
            Ok(code) => {
                // Convert bytes to hex string
                let hex_code = hex::encode(&code);
                Ok(LearnResult {
                    code: Some(hex_code),
                    error: None,
                })
            }
            Err(e) => Ok(LearnResult {
                code: None,
                error: Some(format!("Learning failed: {}", e)),
            }),
        }
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Cancel ongoing learning operation
pub async fn cancel_learn() {
    LEARN_CANCEL.store(true, Ordering::SeqCst);
}

/// Send an IR/RF code to a device
pub async fn send_code(
    host: &str,
    _mac: &str,
    _devtype: &str,
    code: &str,
) -> Result<SendResult, String> {
    let host = host.to_string();
    let code = code.to_string();

    tokio::task::spawn_blocking(move || {
        // Parse the IP address
        let ip: Ipv4Addr = host
            .parse()
            .map_err(|e| format!("Invalid IP address '{}': {}", host, e))?;

        // Decode the hex code
        let code_bytes = hex::decode(&code)
            .map_err(|e| format!("Invalid hex code: {}", e))?;

        // Connect to the device
        let device = Device::from_ip(ip, None)
            .map_err(|e| format!("Failed to connect to device: {}", e))?;

        // Get the remote device
        let remote = match device {
            Device::Remote { remote } => remote,
            _ => return Err("Device is not a remote control".to_string()),
        };

        // Send the code
        match remote.send_code(&code_bytes) {
            Ok(_) => Ok(SendResult {
                success: true,
                error: None,
            }),
            Err(e) => Ok(SendResult {
                success: false,
                error: Some(format!("Send failed: {}", e)),
            }),
        }
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Test if a device is reachable
pub async fn test_device(
    host: &str,
    _mac: &str,
    _devtype: &str,
) -> Result<bool, String> {
    let host = host.to_string();

    tokio::task::spawn_blocking(move || {
        // Parse the IP address
        let ip: Ipv4Addr = match host.parse() {
            Ok(ip) => ip,
            Err(_) => return Ok(false),
        };

        // Try to connect to the device
        match Device::from_ip(ip, None) {
            Ok(Device::Remote { .. }) => Ok(true),
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}
