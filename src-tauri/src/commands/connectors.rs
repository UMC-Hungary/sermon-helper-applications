use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_store::StoreExt;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    connectors::{
        AtemConfig, BroadlinkConfig, ConnectorStatus, DiscordConfig, FacebookConfig, ObsConfig,
        VmixConfig, YouTubeConfig,
    },
    server::OAUTH_REDIRECT_URI,
    AppRuntime,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObsStreamSettings {
    pub service_type: String,
    pub server: String,
    pub key: String,
}

fn load_obs_config(app: &AppHandle) -> Result<ObsConfig, String> {
    let store = app.store("app-settings.json").map_err(|e| e.to_string())?;
    Ok(store
        .get("obs_config")
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default())
}

fn load_vmix_config(app: &AppHandle) -> Result<VmixConfig, String> {
    let store = app.store("app-settings.json").map_err(|e| e.to_string())?;
    Ok(store
        .get("vmix_config")
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default())
}

// ── OBS ─────────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_obs_config(app: AppHandle) -> Result<ObsConfig, String> {
    load_obs_config(&app)
}

#[tauri::command]
pub async fn save_obs_config(
    config: ObsConfig,
    app: AppHandle,
    runtime: State<'_, Arc<RwLock<AppRuntime>>>,
) -> Result<(), String> {
    let store = app.store("app-settings.json").map_err(|e| e.to_string())?;
    store.set(
        "obs_config",
        serde_json::to_value(&config).map_err(|e| e.to_string())?,
    );
    store.save().map_err(|e| e.to_string())?;

    let obs_connector = {
        let rt = runtime.read().await;
        Arc::clone(&rt.obs_connector)
    };

    if config.enabled {
        obs_connector.start(config, app).await;
    } else {
        obs_connector.stop().await;
    }

    Ok(())
}

#[tauri::command]
pub async fn get_obs_status(
    runtime: State<'_, Arc<RwLock<AppRuntime>>>,
) -> Result<ConnectorStatus, String> {
    let obs_connector = {
        let rt = runtime.read().await;
        Arc::clone(&rt.obs_connector)
    };
    Ok(obs_connector.get_status().await)
}

#[tauri::command]
pub async fn connect_obs(
    app: AppHandle,
    runtime: State<'_, Arc<RwLock<AppRuntime>>>,
) -> Result<(), String> {
    let config = load_obs_config(&app)?;
    let obs_connector = {
        let rt = runtime.read().await;
        Arc::clone(&rt.obs_connector)
    };
    obs_connector.start(config, app).await;
    Ok(())
}

#[tauri::command]
pub async fn disconnect_obs(
    runtime: State<'_, Arc<RwLock<AppRuntime>>>,
) -> Result<(), String> {
    let obs_connector = {
        let rt = runtime.read().await;
        Arc::clone(&rt.obs_connector)
    };
    obs_connector.stop().await;
    Ok(())
}

/// Returns the current OBS stream service settings (server URL and stream key).
/// Fails if OBS is not connected.
#[tauri::command]
pub async fn get_obs_stream_settings(
    runtime: State<'_, Arc<RwLock<AppRuntime>>>,
) -> Result<ObsStreamSettings, String> {
    let client_opt = {
        let rt = runtime.read().await;
        let guard = rt.obs_connector.client.lock().await;
        guard.as_ref().map(Arc::clone)
    };
    let client = client_opt.ok_or_else(|| "OBS is not connected".to_string())?;
    let settings = client
        .config()
        .stream_service_settings::<serde_json::Value>()
        .await
        .map_err(|e| e.to_string())?;
    let server = settings
        .settings
        .get("server")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let key = settings
        .settings
        .get("key")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    Ok(ObsStreamSettings {
        service_type: settings.r#type,
        server,
        key,
    })
}

/// Applies a custom RTMP stream destination to OBS.
/// Fails if OBS is not connected.
#[tauri::command]
pub async fn set_obs_stream_settings(
    server: String,
    key: String,
    runtime: State<'_, Arc<RwLock<AppRuntime>>>,
) -> Result<(), String> {
    let client_opt = {
        let rt = runtime.read().await;
        let guard = rt.obs_connector.client.lock().await;
        guard.as_ref().map(Arc::clone)
    };
    let client = client_opt.ok_or_else(|| "OBS is not connected".to_string())?;
    client
        .config()
        .set_stream_service_settings(
            "rtmp_custom",
            &serde_json::json!({ "server": server, "key": key }),
        )
        .await
        .map_err(|e| e.to_string())
}

// ── VMix (stubs) ─────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_vmix_config(app: AppHandle) -> Result<VmixConfig, String> {
    load_vmix_config(&app)
}

#[tauri::command]
pub fn save_vmix_config(config: VmixConfig, app: AppHandle) -> Result<(), String> {
    let store = app.store("app-settings.json").map_err(|e| e.to_string())?;
    store.set(
        "vmix_config",
        serde_json::to_value(&config).map_err(|e| e.to_string())?,
    );
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_vmix_status(
    runtime: State<'_, Arc<RwLock<AppRuntime>>>,
) -> Result<ConnectorStatus, String> {
    let rt = runtime.read().await;
    Ok(rt.vmix_connector.get_status())
}

// ── ATEM (stub) ───────────────────────────────────────────────────────────────

fn load_atem_config(app: &AppHandle) -> Result<AtemConfig, String> {
    let store = app.store("app-settings.json").map_err(|e| e.to_string())?;
    Ok(store
        .get("atem_config")
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default())
}

#[tauri::command]
pub fn get_atem_config(app: AppHandle) -> Result<AtemConfig, String> {
    load_atem_config(&app)
}

#[tauri::command]
pub fn save_atem_config(config: AtemConfig, app: AppHandle) -> Result<(), String> {
    let store = app.store("app-settings.json").map_err(|e| e.to_string())?;
    store.set(
        "atem_config",
        serde_json::to_value(&config).map_err(|e| e.to_string())?,
    );
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_atem_status(
    runtime: State<'_, Arc<RwLock<AppRuntime>>>,
) -> Result<ConnectorStatus, String> {
    let rt = runtime.read().await;
    Ok(rt.atem_connector.get_status())
}

// ── Discord (stub) ────────────────────────────────────────────────────────────

fn load_discord_config(app: &AppHandle) -> Result<DiscordConfig, String> {
    let store = app.store("app-settings.json").map_err(|e| e.to_string())?;
    Ok(store
        .get("discord_config")
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default())
}

#[tauri::command]
pub fn get_discord_config(app: AppHandle) -> Result<DiscordConfig, String> {
    load_discord_config(&app)
}

#[tauri::command]
pub fn save_discord_config(config: DiscordConfig, app: AppHandle) -> Result<(), String> {
    let store = app.store("app-settings.json").map_err(|e| e.to_string())?;
    store.set(
        "discord_config",
        serde_json::to_value(&config).map_err(|e| e.to_string())?,
    );
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_discord_status(
    runtime: State<'_, Arc<RwLock<AppRuntime>>>,
) -> Result<ConnectorStatus, String> {
    let rt = runtime.read().await;
    Ok(rt.discord_connector.get_status())
}

// ── YouTube ───────────────────────────────────────────────────────────────────

fn load_youtube_config(app: &AppHandle) -> Result<YouTubeConfig, String> {
    let store = app.store("app-settings.json").map_err(|e| e.to_string())?;
    Ok(store
        .get("youtube_config")
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default())
}

#[tauri::command]
pub fn get_youtube_config(app: AppHandle) -> Result<YouTubeConfig, String> {
    load_youtube_config(&app)
}

#[tauri::command]
pub async fn save_youtube_config(
    config: YouTubeConfig,
    app: AppHandle,
    runtime: State<'_, Arc<RwLock<AppRuntime>>>,
) -> Result<(), String> {
    let store = app.store("app-settings.json").map_err(|e| e.to_string())?;
    store.set(
        "youtube_config",
        serde_json::to_value(&config).map_err(|e| e.to_string())?,
    );
    store.save().map_err(|e| e.to_string())?;

    let (yt_connector, yt_config_arc) = {
        let rt = runtime.read().await;
        (Arc::clone(&rt.youtube_connector), Arc::clone(&rt.youtube_config))
    };

    // Update the shared Arc so Axum OAuth routes see the new config immediately.
    *yt_config_arc.write().await = config.clone();

    if !config.enabled {
        yt_connector.stop().await;
    }

    Ok(())
}

#[tauri::command]
pub async fn get_youtube_status(
    runtime: State<'_, Arc<RwLock<AppRuntime>>>,
) -> Result<ConnectorStatus, String> {
    let yt_connector = {
        let rt = runtime.read().await;
        Arc::clone(&rt.youtube_connector)
    };
    Ok(yt_connector.get_status().await)
}

/// Generate the Google OAuth authorization URL and store the CSRF state token.
/// Called directly via Tauri IPC to avoid the WebView's mixed-content restriction.
#[tauri::command]
pub async fn get_youtube_auth_url(
    runtime: State<'_, Arc<RwLock<AppRuntime>>>,
) -> Result<String, String> {
    let (config, oauth_states) = {
        let rt = runtime.read().await;
        let config = rt.youtube_config.read().await.clone();
        let oauth_states: Arc<RwLock<HashMap<String, (String, Instant)>>> =
            Arc::clone(&rt.oauth_states);
        (config, oauth_states)
    };

    if config.client_id.is_empty() {
        return Err("YouTube not configured. Please save your Client ID and Client Secret first.".to_string());
    }

    let state_token = Uuid::new_v4().to_string();
    oauth_states
        .write()
        .await
        .insert(state_token.clone(), ("youtube".to_string(), Instant::now()));

    Ok(format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope=https://www.googleapis.com/auth/youtube&access_type=offline&prompt=consent&state={}",
        urlencoding::encode(&config.client_id),
        urlencoding::encode(OAUTH_REDIRECT_URI),
        urlencoding::encode(&state_token),
    ))
}

#[tauri::command]
pub async fn youtube_logout(
    runtime: State<'_, Arc<RwLock<AppRuntime>>>,
) -> Result<(), String> {
    let yt_connector = {
        let rt = runtime.read().await;
        Arc::clone(&rt.youtube_connector)
    };
    yt_connector.stop().await;
    Ok(())
}

// ── Facebook ──────────────────────────────────────────────────────────────────

fn load_facebook_config(app: &AppHandle) -> Result<FacebookConfig, String> {
    let store = app.store("app-settings.json").map_err(|e| e.to_string())?;
    Ok(store
        .get("facebook_config")
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default())
}

#[tauri::command]
pub fn get_facebook_config(app: AppHandle) -> Result<FacebookConfig, String> {
    load_facebook_config(&app)
}

#[tauri::command]
pub async fn save_facebook_config(
    config: FacebookConfig,
    app: AppHandle,
    runtime: State<'_, Arc<RwLock<AppRuntime>>>,
) -> Result<(), String> {
    let store = app.store("app-settings.json").map_err(|e| e.to_string())?;
    store.set(
        "facebook_config",
        serde_json::to_value(&config).map_err(|e| e.to_string())?,
    );
    store.save().map_err(|e| e.to_string())?;

    let (fb_connector, fb_config_arc) = {
        let rt = runtime.read().await;
        (Arc::clone(&rt.facebook_connector), Arc::clone(&rt.facebook_config))
    };

    // Update the shared Arc so Axum OAuth routes see the new config immediately.
    *fb_config_arc.write().await = config.clone();

    if !config.enabled {
        fb_connector.stop().await;
    }

    Ok(())
}

#[tauri::command]
pub async fn get_facebook_status(
    runtime: State<'_, Arc<RwLock<AppRuntime>>>,
) -> Result<ConnectorStatus, String> {
    let fb_connector = {
        let rt = runtime.read().await;
        Arc::clone(&rt.facebook_connector)
    };
    Ok(fb_connector.get_status().await)
}

/// Generate the Facebook OAuth authorization URL and store the CSRF state token.
/// Called directly via Tauri IPC to avoid the WebView's mixed-content restriction.
#[tauri::command]
pub async fn get_facebook_auth_url(
    runtime: State<'_, Arc<RwLock<AppRuntime>>>,
) -> Result<String, String> {
    let (config, oauth_states) = {
        let rt = runtime.read().await;
        let config = rt.facebook_config.read().await.clone();
        let oauth_states: Arc<RwLock<HashMap<String, (String, Instant)>>> =
            Arc::clone(&rt.oauth_states);
        (config, oauth_states)
    };

    if config.app_id.is_empty() {
        return Err("Facebook not configured. Please save your App ID, App Secret, and Page ID first.".to_string());
    }

    let state_token = Uuid::new_v4().to_string();
    oauth_states
        .write()
        .await
        .insert(state_token.clone(), ("facebook".to_string(), Instant::now()));

    Ok(format!(
        "https://www.facebook.com/v19.0/dialog/oauth?client_id={}&redirect_uri={}&scope=pages_manage_posts,pages_read_engagement,publish_video&state={}",
        urlencoding::encode(&config.app_id),
        urlencoding::encode(OAUTH_REDIRECT_URI),
        urlencoding::encode(&state_token),
    ))
}

#[tauri::command]
pub async fn facebook_logout(
    runtime: State<'_, Arc<RwLock<AppRuntime>>>,
) -> Result<(), String> {
    let fb_connector = {
        let rt = runtime.read().await;
        Arc::clone(&rt.facebook_connector)
    };
    fb_connector.stop().await;
    Ok(())
}

// ── Broadlink ─────────────────────────────────────────────────────────────────

fn load_broadlink_config(app: &AppHandle) -> Result<BroadlinkConfig, String> {
    let store = app.store("app-settings.json").map_err(|e| e.to_string())?;
    Ok(store
        .get("broadlink_config")
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default())
}

#[tauri::command]
pub fn get_broadlink_config(app: AppHandle) -> Result<BroadlinkConfig, String> {
    load_broadlink_config(&app)
}

#[tauri::command]
pub fn save_broadlink_config(config: BroadlinkConfig, app: AppHandle) -> Result<(), String> {
    let store = app.store("app-settings.json").map_err(|e| e.to_string())?;
    store.set(
        "broadlink_config",
        serde_json::to_value(&config).map_err(|e| e.to_string())?,
    );
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_broadlink_status(
    runtime: State<'_, Arc<RwLock<AppRuntime>>>,
) -> Result<ConnectorStatus, String> {
    let connector = {
        let rt = runtime.read().await;
        Arc::clone(&rt.broadlink_connector)
    };
    Ok(connector.get_status().await)
}

#[tauri::command]
pub async fn broadlink_discover(
    runtime: State<'_, Arc<RwLock<AppRuntime>>>,
) -> Result<Vec<crate::broadlink::DiscoveredDevice>, String> {
    let config = {
        let rt = runtime.read().await;
        let _ = Arc::clone(&rt.broadlink_connector);
        5u32 // default timeout
    };
    crate::broadlink::discover_devices(config).await
}

#[tauri::command]
pub async fn broadlink_learn(
    host: String,
    mac: String,
    devtype: String,
    signal_type: String,
) -> Result<crate::broadlink::LearnResult, String> {
    crate::broadlink::learn_code(&host, &mac, &devtype, &signal_type).await
}

#[tauri::command]
pub async fn broadlink_cancel_learn() {
    crate::broadlink::cancel_learn().await;
}

#[tauri::command]
pub async fn broadlink_send(
    host: String,
    mac: String,
    devtype: String,
    code: String,
) -> Result<crate::broadlink::SendResult, String> {
    crate::broadlink::send_code(&host, &mac, &devtype, &code).await
}

#[tauri::command]
pub async fn broadlink_test_device(
    host: String,
    mac: String,
    devtype: String,
) -> Result<bool, String> {
    crate::broadlink::test_device(&host, &mac, &devtype).await
}

#[tauri::command]
pub async fn broadlink_list_interfaces() -> Result<Vec<(String, String)>, String> {
    crate::broadlink::list_network_interfaces().await
}

// ── Relay config ──────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_relay_config(app: AppHandle) -> Result<crate::mediamtx::RelayConfig, String> {
    let store = app.store("app-settings.json").map_err(|e| e.to_string())?;
    Ok(store
        .get("relay_config")
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default())
}

#[tauri::command]
pub async fn save_relay_config(
    config: crate::mediamtx::RelayConfig,
    app: AppHandle,
    runtime: State<'_, Arc<RwLock<AppRuntime>>>,
) -> Result<(), String> {
    let store = app.store("app-settings.json").map_err(|e| e.to_string())?;
    store.set(
        "relay_config",
        serde_json::to_value(&config).map_err(|e| e.to_string())?,
    );
    store.save().map_err(|e| e.to_string())?;

    let (mediamtx_mgr, relay_config_arc, mode) = {
        let rt = runtime.read().await;
        (
            Arc::clone(&rt.mediamtx_manager),
            Arc::clone(&rt.relay_config),
            rt.mode.clone(),
        )
    };

    *relay_config_arc.write().await = config.clone();

    if mode.as_deref() == Some("server") {
        let data_dir = app.path().app_data_dir().map_err(|e: tauri::Error| e.to_string())?;
        mediamtx_mgr
            .restart(&app, &data_dir, &config)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Returns `true` if the mediamtx binary is present on this machine.
#[tauri::command]
pub async fn get_mediamtx_status(
    app: AppHandle,
    runtime: State<'_, Arc<RwLock<AppRuntime>>>,
) -> Result<bool, String> {
    let data_dir = app.path().app_data_dir().map_err(|e: tauri::Error| e.to_string())?;
    let rt = runtime.read().await;
    Ok(rt.mediamtx_manager.is_installed(&app, &data_dir))
}

/// Download the mediamtx binary. Emits `mediamtx://progress` events during download.
#[tauri::command]
pub async fn download_mediamtx(
    app: AppHandle,
    runtime: State<'_, Arc<RwLock<AppRuntime>>>,
) -> Result<(), String> {
    let data_dir = app.path().app_data_dir().map_err(|e: tauri::Error| e.to_string())?;
    let mgr = {
        let rt = runtime.read().await;
        Arc::clone(&rt.mediamtx_manager)
    };
    mgr.download(&app, &data_dir).await.map_err(|e| e.to_string())
}
