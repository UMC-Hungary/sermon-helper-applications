//! Tauri commands for Companion API integration

use crate::companion_api::{
    create_ppt_selector_page, CompanionApi, PptSelectorLayout, DEFAULT_COMPANION_PORT,
};
use serde::{Deserialize, Serialize};

/// Result of checking Companion connection
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanionStatus {
    pub available: bool,
    pub host: String,
    pub port: u16,
    pub error: Option<String>,
}

/// Request to create PPT selector page
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePptPageRequest {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_page")]
    pub page: u32,
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    DEFAULT_COMPANION_PORT
}

fn default_page() -> u32 {
    1
}

/// Check if Companion is available
#[tauri::command]
pub async fn check_companion_connection(
    host: Option<String>,
    port: Option<u16>,
) -> Result<CompanionStatus, String> {
    let host = host.unwrap_or_else(default_host);
    let port = port.unwrap_or(DEFAULT_COMPANION_PORT);

    // Try to connect
    let client = reqwest::Client::new();
    let url = format!("http://{}:{}/", host, port);

    match client
        .get(&url)
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await
    {
        Ok(response) => {
            let available = response.status().is_success() || response.status().as_u16() == 304;
            Ok(CompanionStatus {
                available,
                host,
                port,
                error: if available { None } else { Some(format!("HTTP {}", response.status())) },
            })
        }
        Err(e) => {
            let error_msg = if e.is_connect() {
                "Connection refused - is Companion running?".to_string()
            } else if e.is_timeout() {
                "Connection timed out".to_string()
            } else {
                format!("{}", e)
            };

            Ok(CompanionStatus {
                available: false,
                host,
                port,
                error: Some(error_msg),
            })
        }
    }
}

/// Create PPT selector buttons on a Companion page
#[tauri::command]
pub async fn create_companion_ppt_page(request: CreatePptPageRequest) -> Result<String, String> {
    let api = CompanionApi::new(&request.host, request.port);

    // First check if Companion is available
    if !api.check_connection().await.unwrap_or(false) {
        return Err(format!(
            "Cannot connect to Companion at {}:{}.\n\nTroubleshooting:\n1. Make sure Companion is running\n2. Check the port in Companion's settings (default: 8000)\n3. Try opening http://{}:{} in your browser\n4. Check if firewall is blocking the connection",
            request.host, request.port, request.host, request.port
        ));
    }

    let layout = PptSelectorLayout {
        page: request.page,
    };

    match create_ppt_selector_page(&api, &layout).await {
        Ok(_) => Ok(format!(
            "PPT Selector buttons created on page {}. Note: You need to manually configure the button actions in Companion.",
            request.page
        )),
        Err(e) => Err(format!(
            "Connected to Companion but failed to create buttons: {}\n\nThis might be a Companion API version issue. Try importing the .companionconfig file instead.",
            e
        ))
    }
}

/// Get the path to the bundled .companionconfig file
#[tauri::command]
pub fn get_companion_config_path() -> Result<String, String> {
    // Return info about where users can find the config file
    Ok("The PPT Selector page configuration can be imported from the companion-module-sermon-helper package. Look for 'ppt-selector-page.companionconfig' in the module folder.".to_string())
}
