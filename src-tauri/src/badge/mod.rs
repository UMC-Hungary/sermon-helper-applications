pub mod encrypted_shader;

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BadgeInstallResult {
    pub shaderfilter_installed: bool,
    pub shader_installed: bool,
    pub sources_created: bool,
}

pub fn get_obs_plugin_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
    home.join("Library/Application Support/obs-studio/plugins")
}

pub fn check_shaderfilter_installed() -> bool {
    let plugin_dir = get_obs_plugin_dir();
    
    #[cfg(target_os = "macos")]
    let shaderfilter_path = plugin_dir.join("obs-shaderfilter.dylib");
    
    #[cfg(not(target_os = "macos"))]
    let shaderfilter_path = plugin_dir.join("obs-shaderfilter.so");
    
    shaderfilter_path.exists()
}

pub async fn download_shaderfilter() -> Result<PathBuf, String> {
    use reqwest::Client;
    use std::fs;
    
    let plugin_dir = get_obs_plugin_dir();
    fs::create_dir_all(&plugin_dir).map_err(|e| format!("Failed to create plugin directory: {}", e))?;
    
    let url = "https://github.com/exeldro/obs-shaderfilter/releases/download/2.4.3/obs-shaderfilter-macOS.zip";
    
    let client = Client::new();
    let response = client.get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to download shaderfilter: {}", e))?;
    
    let bytes = response.bytes()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;
    
    let zip_path = plugin_dir.join("obs-shaderfilter.zip");
    fs::write(&zip_path, &bytes).map_err(|e| format!("Failed to save zip: {}", e))?;
    
    Ok(zip_path)
}

pub fn extract_shaderfilter(zip_path: &PathBuf) -> Result<(), String> {
    use std::fs::File;
    
    let file = File::open(zip_path).map_err(|e| format!("Failed to open zip: {}", e))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Failed to read zip: {}", e))?;
    
    let plugin_dir = get_obs_plugin_dir();
    
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| format!("Failed to read zip entry: {}", e))?;
        let outpath = plugin_dir.join(file.name());
        
        if file.is_dir() {
            std::fs::create_dir_all(&outpath).ok();
        } else {
            if let Some(p) = outpath.parent() {
                std::fs::create_dir_all(p).ok();
            }
            let mut outfile = std::fs::File::create(&outpath).map_err(|e| format!("Failed to create file: {}", e))?;
            std::io::copy(&mut file, &mut outfile).map_err(|e| format!("Failed to extract file: {}", e))?;
        }
    }
    
    Ok(())
}

pub fn install_shader() -> Result<PathBuf, String> {
    let shader_content = encrypted_shader::get_shader_content()?;
    
    let plugin_dir = get_obs_plugin_dir();
    std::fs::create_dir_all(&plugin_dir).map_err(|e| format!("Failed to create plugin directory: {}", e))?;
    
    let shader_path = plugin_dir.join("LucidGlass.shader");
    std::fs::write(&shader_path, shader_content).map_err(|e| format!("Failed to write shader: {}", e))?;
    
    Ok(shader_path)
}

pub async fn install_badge() -> Result<BadgeInstallResult, String> {
    let shaderfilter_installed = check_shaderfilter_installed();
    
    if !shaderfilter_installed {
        let zip_path = download_shaderfilter().await?;
        extract_shaderfilter(&zip_path)?;
        std::fs::remove_file(zip_path).ok();
    }
    
    let shader_path = install_shader()?;
    let shader_installed = shader_path.exists();
    
    Ok(BadgeInstallResult {
        shaderfilter_installed: check_shaderfilter_installed(),
        shader_installed,
        sources_created: false,
    })
}

pub fn get_shader_path() -> PathBuf {
    get_obs_plugin_dir().join("LucidGlass.shader")
}
