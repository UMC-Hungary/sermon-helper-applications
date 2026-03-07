use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

use crate::AppRuntime;

#[tauri::command]
pub async fn get_server_port(runtime: State<'_, Arc<RwLock<AppRuntime>>>) -> Result<u16, String> {
    let rt = runtime.read().await;
    Ok(rt.server_port)
}

#[tauri::command]
pub async fn get_app_mode(
    runtime: State<'_, Arc<RwLock<AppRuntime>>>,
) -> Result<Option<String>, String> {
    let rt = runtime.read().await;
    Ok(rt.mode.clone())
}

#[tauri::command]
pub async fn set_app_mode(
    mode: String,
    url: Option<String>,
    runtime: State<'_, Arc<RwLock<AppRuntime>>>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut rt = runtime.write().await;
    rt.mode = Some(mode.clone());
    if let Some(u) = url {
        rt.client_url = Some(u.clone());
        save_setting(&app, "server_url", &u).await?;
    }
    save_setting(&app, "mode", &mode).await?;
    Ok(())
}

#[tauri::command]
pub async fn complete_setup(
    mode: String,
    server_url: Option<String>,
    client_token: Option<String>,
    runtime: State<'_, Arc<RwLock<AppRuntime>>>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    if mode != "server" && mode != "client" {
        return Err(format!("Invalid mode: {mode}"));
    }

    // Server mode requires desktop (embedded PostgreSQL + Axum).
    #[cfg(mobile)]
    if mode == "server" {
        return Err("Server mode is not supported on mobile devices".to_string());
    }

    save_setting(&app, "mode", &mode).await?;

    if mode == "client" {
        let url = server_url
            .as_deref()
            .filter(|s| !s.is_empty())
            .ok_or("server_url is required for client mode")?;
        let token = client_token
            .as_deref()
            .filter(|s| !s.is_empty())
            .ok_or("client_token is required for client mode")?;

        save_setting(&app, "server_url", url).await?;
        save_setting(&app, "client_auth_token", token).await?;

        let mut rt = runtime.write().await;
        rt.client_url = Some(url.to_string());
        *rt.auth_token.write().await = token.to_string();
        rt.mode = Some(mode);
    } else {
        // server mode — desktop only
        #[cfg(desktop)]
        {
            let mut rt = runtime.write().await;
            rt.mode = Some(mode);
            let auth_token_arc = rt.auth_token.clone();
            let port = rt.server_port;
            let obs = Arc::clone(&rt.obs_connector);
            let vmix = Arc::clone(&rt.vmix_connector);
            let yt = Arc::clone(&rt.youtube_connector);
            let fb = Arc::clone(&rt.facebook_connector);
            let bl = Arc::clone(&rt.broadlink_connector);
            // Use the shared config Arcs from AppRuntime so that any config
            // saved via Tauri commands is immediately visible to Axum routes.
            let yt_cfg = Arc::clone(&rt.youtube_config);
            let fb_cfg = Arc::clone(&rt.facebook_config);
            let oauth = Arc::clone(&rt.oauth_states);
            #[cfg(target_os = "macos")]
            let kn = Arc::clone(&rt.keynote_connector);
            drop(rt);

            let handle = app.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = crate::start_server(
                    handle,
                    auth_token_arc,
                    port,
                    obs,
                    vmix,
                    yt,
                    fb,
                    bl,
                    yt_cfg,
                    fb_cfg,
                    oauth,
                    #[cfg(target_os = "macos")]
                    kn,
                )
                .await
                {
                    tracing::error!("Backend startup failed: {e}");
                }
            });
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn get_client_url(
    runtime: State<'_, Arc<RwLock<AppRuntime>>>,
) -> Result<Option<String>, String> {
    let rt = runtime.read().await;
    Ok(rt.client_url.clone())
}

#[tauri::command]
pub async fn get_client_token(
    runtime: State<'_, Arc<RwLock<AppRuntime>>>,
) -> Result<String, String> {
    let rt = runtime.read().await;
    let token = rt.auth_token.read().await.clone();
    Ok(token)
}

#[tauri::command]
pub async fn reset_setup(
    runtime: State<'_, Arc<RwLock<AppRuntime>>>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    use tauri_plugin_store::StoreExt;
    let store = app
        .store("app-settings.json")
        .map_err(|e| e.to_string())?;
    store.delete("mode");
    store.save().map_err(|e| e.to_string())?;

    let mut rt = runtime.write().await;
    rt.mode = None;
    Ok(())
}

#[tauri::command]
pub fn get_local_ip() -> Option<String> {
    use std::net::UdpSocket;
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    Some(socket.local_addr().ok()?.ip().to_string())
}

async fn save_setting(app: &tauri::AppHandle, key: &str, value: &str) -> Result<(), String> {
    use tauri_plugin_store::StoreExt;
    let store = app
        .store("app-settings.json")
        .map_err(|e| e.to_string())?;
    store.set(key, serde_json::Value::String(value.to_string()));
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}
