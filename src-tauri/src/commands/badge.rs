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

    // Create __caption browser source (ignore if already exists)
    let caption_request = obws::requests::inputs::Create {
        scene: scene_id,
        input: "__caption",
        kind: "browser_source",
        settings: Some(serde_json::json!({
            "url": caption_url,
            "is_local_file": false,
        })),
        enabled: Some(true),
    };

    match client.inputs().create(caption_request).await {
        Ok(_) => {}
        Err(e) => {
            let error_msg = format!("{:?}", e);
            if !error_msg.contains("ResourceAlreadyExists") {
                return Err(format!("Failed to create __caption source: {}", e));
            }
        }
    }

    // Always recreate __caption-background as a Color Source so the LucidGlass
    // shader has a solid-colour canvas to draw its glass card onto.
    // (A browser source cannot work here — the shader only sees the source's own
    //  rendered pixels, not the OBS scene content behind it.)
    client.inputs().remove("__caption-background".into()).await.ok();

    let bg_request = obws::requests::inputs::Create {
        scene: scene_id,
        input: "__caption-background",
        kind: "color_source_v3",
        settings: Some(serde_json::json!({
            "color": -1_i64,   // 0xFFFFFFFF — fully opaque white
        })),
        enabled: Some(true),
    };

    // Catch ResourceAlreadyExists in case OBS hasn't finished processing the
    // remove() above before the create() arrives (WebSocket acknowledgement races).
    match client.inputs().create(bg_request).await {
        Ok(_) => {}
        Err(e) => {
            let error_msg = format!("{:?}", e);
            if !error_msg.contains("ResourceAlreadyExists") {
                return Err(format!("Failed to create __caption-background source: {}", e));
            }
            // Source already exists from a previous run — reuse it.
        }
    }

    // Detect the shader filter kind from what OBS has actually loaded.
    let all_kinds = client
        .filters()
        .list_kinds()
        .await
        .map_err(|e| format!("Failed to list filter kinds: {}", e))?;

    let shader_kind = all_kinds
        .iter()
        .find(|k| k.as_str() == "obs_shaderfilter")
        .or_else(|| all_kinds.iter().find(|k| k.to_lowercase().contains("shader")))
        .map(|s| s.as_str())
        .ok_or_else(|| {
            let shader_related: Vec<&str> = all_kinds
                .iter()
                .filter(|k| k.to_lowercase().contains("shader") || k.to_lowercase().contains("filter"))
                .map(|s| s.as_str())
                .collect();
            if shader_related.is_empty() {
                "obs-shaderfilter plugin is not loaded in OBS. \
                 Install the plugin and restart OBS, then try again."
                    .to_string()
            } else {
                format!(
                    "obs-shaderfilter plugin is not loaded in OBS. \
                     Available shader/filter kinds: {}. \
                     Restart OBS after installing the plugin.",
                    shader_related.join(", ")
                )
            }
        })?;

    // Remove existing LucidGlass filter (if any) so shader settings are always
    // reloaded from the current file on each call to create_badge_sources.
    client.filters().remove("__caption-background".into(), "LucidGlass").await.ok();

    // Add LucidGlass shader filter to the colour source.
    let filter_request = obws::requests::filters::Create {
        source: "__caption-background".into(),
        filter: "LucidGlass",
        kind: shader_kind,
        settings: Some(serde_json::json!({
            "shader_file_name": shader_path_str,
        })),
    };

    client.filters().create(filter_request).await
        .map_err(|e| format!("Failed to add LucidGlass shader filter: {}", e))?;

    Ok(())
}
