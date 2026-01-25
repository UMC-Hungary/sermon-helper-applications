//! Broadlink RF/IR device integration module
//!
//! This module provides integration with Broadlink devices (RM4, RM Pro, etc.)
//! for sending and learning IR/RF codes using the native Rust rbroadlink crate.

use rbroadlink::Device;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr};
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

/// Get all IPv4 addresses from network interfaces (excluding loopback and virtual)
fn get_local_ipv4_addresses() -> Vec<Ipv4Addr> {
    let mut addresses = Vec::new();

    if let Ok(interfaces) = local_ip_address::list_afinet_netifas() {
        for (name, ip) in interfaces {
            if let IpAddr::V4(ipv4) = ip {
                // Skip loopback
                if ipv4.is_loopback() {
                    continue;
                }

                // Skip common virtual interface prefixes
                let name_lower = name.to_lowercase();
                if name_lower.starts_with("veth")
                    || name_lower.starts_with("docker")
                    || name_lower.starts_with("br-")
                    || name_lower.starts_with("virbr")
                    || name_lower.contains("wsl")
                    || name_lower.contains("hyper-v")
                    || name_lower.contains("virtualbox")
                {
                    log::debug!("Skipping virtual interface: {} ({})", name, ipv4);
                    continue;
                }

                log::info!("Found network interface: {} ({})", name, ipv4);
                addresses.push(ipv4);
            }
        }
    }

    addresses
}

/// Discover Broadlink devices on the network by trying all interfaces
pub async fn discover_devices(_timeout: u32) -> Result<Vec<DiscoveredDevice>, String> {
    tokio::task::spawn_blocking(|| {
        let local_ips = get_local_ipv4_addresses();

        if local_ips.is_empty() {
            return Err("No suitable network interfaces found".to_string());
        }

        log::info!("Attempting discovery on {} network interface(s)", local_ips.len());

        let mut all_discovered = Vec::new();
        let mut seen_macs = HashSet::new();
        let mut last_error = String::new();

        // Try discovery on each interface
        for local_ip in local_ips {
            log::info!("Trying discovery on interface: {}", local_ip);

            match Device::list(Some(local_ip)) {
                Ok(devices) => {
                    log::info!("Found {} device(s) on interface {}", devices.len(), local_ip);

                    for device in devices {
                        match device {
                            Device::Remote { ref remote } => {
                                let info = &remote.info;
                                let mac = format_mac(&info.mac);

                                // Skip duplicates (device might respond on multiple interfaces)
                                if seen_macs.contains(&mac) {
                                    continue;
                                }
                                seen_macs.insert(mac.clone());

                                all_discovered.push(DiscoveredDevice {
                                    device_type: info.friendly_type.clone(),
                                    model: info.friendly_model.clone(),
                                    host: info.address.to_string(),
                                    mac,
                                    name: if info.name.is_empty() {
                                        info.friendly_model.clone()
                                    } else {
                                        info.name.clone()
                                    },
                                });
                            }
                            Device::Hvac { ref hvac } => {
                                let info = &hvac.info;
                                let mac = format_mac(&info.mac);

                                if seen_macs.contains(&mac) {
                                    continue;
                                }
                                seen_macs.insert(mac.clone());

                                all_discovered.push(DiscoveredDevice {
                                    device_type: info.friendly_type.clone(),
                                    model: info.friendly_model.clone(),
                                    host: info.address.to_string(),
                                    mac,
                                    name: if info.name.is_empty() {
                                        info.friendly_model.clone()
                                    } else {
                                        info.name.clone()
                                    },
                                });
                            }
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Discovery failed on interface {}: {}", local_ip, e);
                    last_error = format!("Discovery on {} failed: {}", local_ip, e);
                }
            }
        }

        if all_discovered.is_empty() && !last_error.is_empty() {
            log::error!("No devices found. Last error: {}", last_error);
        }

        log::info!("Total discovered devices: {}", all_discovered.len());
        Ok(all_discovered)
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Get the best local IP for communicating with a specific device IP
fn get_local_ip_for_device(device_ip: Ipv4Addr) -> Option<Ipv4Addr> {
    let local_ips = get_local_ipv4_addresses();

    // Try to find an IP in the same subnet (simple heuristic: same first 3 octets)
    let device_octets = device_ip.octets();
    for local_ip in &local_ips {
        let local_octets = local_ip.octets();
        if local_octets[0] == device_octets[0]
            && local_octets[1] == device_octets[1]
            && local_octets[2] == device_octets[2]
        {
            return Some(*local_ip);
        }
    }

    // Fall back to first available IP
    local_ips.into_iter().next()
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

        // Get the best local IP for this device
        let local_ip = get_local_ip_for_device(ip);
        log::info!("Learning from device {} using local IP {:?}", ip, local_ip);

        // Connect to the device
        let device = Device::from_ip(ip, local_ip)
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

        // Get the best local IP for this device
        let local_ip = get_local_ip_for_device(ip);
        log::info!("Sending to device {} using local IP {:?}", ip, local_ip);

        // Connect to the device
        let device = Device::from_ip(ip, local_ip)
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

        // Get the best local IP for this device
        let local_ip = get_local_ip_for_device(ip);

        // Try to connect to the device
        match Device::from_ip(ip, local_ip) {
            Ok(Device::Remote { .. }) => Ok(true),
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// List available network interfaces (for debugging/UI)
pub async fn list_network_interfaces() -> Result<Vec<(String, String)>, String> {
    tokio::task::spawn_blocking(|| {
        let mut interfaces = Vec::new();

        if let Ok(netifs) = local_ip_address::list_afinet_netifas() {
            for (name, ip) in netifs {
                if let IpAddr::V4(ipv4) = ip {
                    if !ipv4.is_loopback() {
                        interfaces.push((name, ipv4.to_string()));
                    }
                }
            }
        }

        Ok(interfaces)
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}
