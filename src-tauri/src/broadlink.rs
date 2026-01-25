//! Broadlink RF/IR device integration module
//!
//! This module provides integration with Broadlink devices (RM4, RM Pro, etc.)
//! for sending and learning IR/RF codes. It uses a Python bridge script to
//! communicate with the devices since the broadlink protocol is complex.

use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;
use std::sync::Arc;
use std::time::Duration;

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
static LEARN_CANCEL: Mutex<bool> = Mutex::const_new(false);

/// Get the path to the Python bridge script
fn get_bridge_script_path() -> std::path::PathBuf {
    // In development, use the scripts directory
    // In production, use the resource directory
    let script_name = "broadlink_bridge.py";

    // Try relative path first (development)
    let dev_path = std::path::PathBuf::from("scripts").join(script_name);
    if dev_path.exists() {
        return dev_path;
    }

    // Try resource directory (production)
    // This would be set up in tauri.conf.json to bundle the script
    std::path::PathBuf::from("resources").join(script_name)
}

/// Discover Broadlink devices on the network
pub async fn discover_devices(timeout: u32) -> Result<Vec<DiscoveredDevice>, String> {
    let script_path = get_bridge_script_path();

    let output = Command::new("python3")
        .arg(&script_path)
        .arg("discover")
        .arg(timeout.to_string())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("Failed to run discovery script: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Discovery failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse discovery result: {} (output: {})", e, stdout))
}

/// Enter learning mode and wait for IR/RF signal
pub async fn learn_code(
    host: &str,
    mac: &str,
    devtype: &str,
    signal_type: &str,
) -> Result<LearnResult, String> {
    // Reset cancellation flag
    {
        let mut cancel = LEARN_CANCEL.lock().await;
        *cancel = false;
    }

    let script_path = get_bridge_script_path();

    let mut child = Command::new("python3")
        .arg(&script_path)
        .arg("learn")
        .arg(host)
        .arg(mac)
        .arg(devtype)
        .arg(signal_type)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start learning: {}", e))?;

    // Wait for the process with timeout and cancellation check
    let timeout_secs = 30;
    let start = std::time::Instant::now();

    loop {
        // Check for cancellation
        {
            let cancel = LEARN_CANCEL.lock().await;
            if *cancel {
                let _ = child.kill().await;
                return Ok(LearnResult {
                    code: None,
                    error: Some("Learning cancelled".to_string()),
                });
            }
        }

        // Check if process has completed
        match child.try_wait() {
            Ok(Some(status)) => {
                let output = child.wait_with_output().await
                    .map_err(|e| format!("Failed to get output: {}", e))?;

                if !status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Ok(LearnResult {
                        code: None,
                        error: Some(format!("Learning failed: {}", stderr)),
                    });
                }

                let stdout = String::from_utf8_lossy(&output.stdout);
                return serde_json::from_str(&stdout)
                    .map_err(|e| format!("Failed to parse learn result: {}", e));
            }
            Ok(None) => {
                // Process still running
                if start.elapsed() > Duration::from_secs(timeout_secs) {
                    let _ = child.kill().await;
                    return Ok(LearnResult {
                        code: None,
                        error: Some("Learning timeout".to_string()),
                    });
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            Err(e) => {
                return Err(format!("Error checking process: {}", e));
            }
        }
    }
}

/// Cancel ongoing learning operation
pub async fn cancel_learn() {
    let mut cancel = LEARN_CANCEL.lock().await;
    *cancel = true;
}

/// Send an IR/RF code to a device
pub async fn send_code(
    host: &str,
    mac: &str,
    devtype: &str,
    code: &str,
) -> Result<SendResult, String> {
    let script_path = get_bridge_script_path();

    let output = Command::new("python3")
        .arg(&script_path)
        .arg("send")
        .arg(host)
        .arg(mac)
        .arg(devtype)
        .arg(code)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("Failed to send code: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Ok(SendResult {
            success: false,
            error: Some(format!("Send failed: {}", stderr)),
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse send result: {}", e))
}

/// Test if a device is reachable
pub async fn test_device(
    host: &str,
    mac: &str,
    devtype: &str,
) -> Result<bool, String> {
    let script_path = get_bridge_script_path();

    let output = Command::new("python3")
        .arg(&script_path)
        .arg("test")
        .arg(host)
        .arg(mac)
        .arg(devtype)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("Failed to test device: {}", e))?;

    if !output.status.success() {
        return Ok(false);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    #[derive(Deserialize)]
    struct TestResult {
        online: bool,
    }

    serde_json::from_str::<TestResult>(&stdout)
        .map(|r| r.online)
        .map_err(|e| format!("Failed to parse test result: {}", e))
}
