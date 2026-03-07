use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tauri::State;

use crate::badge::{self, BadgeInstallResult};
use crate::AppRuntime;

#[derive(Debug, Serialize, Deserialize)]
pub struct ObsScene {
    pub name: String,
}

#[tauri::command]
pub async fn install_badge() -> Result<BadgeInstallResult, String> {
    badge::install_badge().await
}

#[tauri::command]
pub async fn get_obs_scenes(
    runtime: State<'_, Arc<RwLock<AppRuntime>>>,
) -> Result<Vec<ObsScene>, String> {
    let rt = runtime.read().await;
    let obs_connector = Arc::clone(&rt.obs_connector);
    
    let client_guard = obs_connector.client.lock().await;
    let client = client_guard.as_ref().ok_or("OBS not connected")?;
    
    let scene_list = client.scenes()
        .list()
        .await
        .map_err(|e| format!("Failed to get scenes: {}", e))?;
    
    Ok(scene_list.scenes.into_iter().map(|s| ObsScene { name: s.id.name }).collect())
}

#[tauri::command]
pub async fn create_badge_sources(
    scene_name: String,
    runtime: State<'_, Arc<RwLock<AppRuntime>>>,
) -> Result<(), String> {
    let rt = runtime.read().await;
    let obs_connector = Arc::clone(&rt.obs_connector);
    
    let client_guard = obs_connector.client.lock().await;
    let client = client_guard.as_ref().ok_or("OBS not connected")?;
    
    let caption_url = "http://localhost:3737/caption?type=caption&resolution=4k&bold=Textus:&light=Lekcio:&color=red&showLogo=true";
    let shader_path = badge::get_shader_path();
    let shader_path_str = shader_path.to_string_lossy().to_string();
    
    let scene_id = obws::requests::scenes::SceneId::Name(&scene_name);
    
    let request = obws::requests::inputs::Create {
        scene: scene_id,
        input: "__caption",
        kind: "browser_source",
        settings: Some(serde_json::json!({
            "url": caption_url,
            "is_local_file": false,
        })),
        enabled: Some(true),
    };
    
    let _ = client.inputs()
        .create(request)
        .await
        .map_err(|e| format!("Failed to create __caption source: {}", e))?;
    
    let request_bg = obws::requests::inputs::Create {
        scene: scene_id,
        input: "__caption-background",
        kind: "browser_source",
        settings: Some(serde_json::json!({
            "url": caption_url,
            "is_local_file": false,
        })),
        enabled: Some(true),
    };
    
    let _ = client.inputs()
        .create(request_bg)
        .await
        .map_err(|e| format!("Failed to create __caption-background source: {}", e))?;
    
    let filter_request = obws::requests::filters::Create {
        source: "__caption-background".into(),
        filter: "LucidGlass",
        kind: "user-defined_shader",
        settings: Some(serde_json::json!({
            "shader_file": shader_path_str,
        })),
    };
    
    let _ = client.filters()
        .create(filter_request)
        .await
        .map_err(|e| format!("Failed to add shader filter: {}", e))?;
    
    Ok(())
}
