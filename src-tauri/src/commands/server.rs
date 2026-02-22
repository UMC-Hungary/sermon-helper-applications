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
pub async fn get_app_mode(runtime: State<'_, Arc<RwLock<AppRuntime>>>) -> Result<String, String> {
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
    rt.mode = mode.clone();
    if let Some(u) = url {
        rt.client_url = Some(u.clone());
        save_setting(&app, "server_url", &u).await?;
    }
    save_setting(&app, "mode", &mode).await?;
    Ok(())
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
