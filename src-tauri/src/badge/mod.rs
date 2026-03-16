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
    // Only check the user plugins path — that is the only path OBS searches at runtime.
    // The system-wide /Library path is where the .pkg installer writes, but OBS does
    // not load from there; we copy to the user path in extract_shaderfilter().
    let user_base = get_obs_plugin_dir();
    user_base.join("obs-shaderfilter.plugin").exists()
        || user_base.join("obs-shaderfilter").exists()
}

async fn resolve_latest_download_url(client: &reqwest::Client) -> Result<(String, String), String> {
    #[cfg(target_os = "macos")]
    let asset_suffix = "-macos-universal.pkg";
    #[cfg(not(target_os = "macos"))]
    let asset_suffix = "-windows.zip";

    let api_url = "https://api.github.com/repos/exeldro/obs-shaderfilter/releases/latest";
    let response = client
        .get(api_url)
        .header("User-Agent", "sermon-helper-tauri")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch latest release info: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "GitHub API returned HTTP {} when checking for the latest release",
            response.status()
        ));
    }

    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse release info: {}", e))?;

    let tag = body["tag_name"]
        .as_str()
        .ok_or("Release response missing tag_name")?;

    let assets = body["assets"]
        .as_array()
        .ok_or("Release response missing assets")?;

    let asset = assets
        .iter()
        .find(|a| {
            a["name"]
                .as_str()
                .map(|n| n.ends_with(asset_suffix))
                .unwrap_or(false)
        })
        .ok_or_else(|| format!("No {asset_suffix} asset found in release {tag}"))?;

    let download_url = asset["browser_download_url"]
        .as_str()
        .ok_or("Asset missing browser_download_url")?
        .to_string();

    let file_name = asset["name"]
        .as_str()
        .ok_or("Asset missing name")?
        .to_string();

    Ok((download_url, file_name))
}

pub async fn download_shaderfilter() -> Result<PathBuf, String> {
    use std::fs;

    let plugin_dir = get_obs_plugin_dir();
    fs::create_dir_all(&plugin_dir)
        .map_err(|e| format!("Failed to create plugin directory: {}", e))?;

    let client = reqwest::Client::new();

    let (url, file_name) = resolve_latest_download_url(&client).await?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to download shaderfilter: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Download failed: HTTP {} for {}",
            response.status(),
            url
        ));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read download: {}", e))?;

    let pkg_path = plugin_dir.join(&file_name);
    fs::write(&pkg_path, &bytes).map_err(|e| format!("Failed to save file: {}", e))?;

    Ok(pkg_path)
}

#[cfg(target_os = "macos")]
pub fn extract_shaderfilter(pkg_path: &PathBuf) -> Result<(), String> {
    use std::process::Command;

    // Run the .pkg installer with administrator privileges.
    // This puts the plugin into /Library/Application Support/obs-studio/plugins/
    // (the system-wide path), NOT the user path that OBS actually searches.
    let pkg_str = pkg_path.to_string_lossy();
    let pkg_escaped = pkg_str.replace('\'', "'\"'\"'");
    let script = format!(
        "do shell script \"/usr/sbin/installer -pkg '{}' -target /\" with administrator privileges",
        pkg_escaped
    );

    let output = Command::new("osascript")
        .args(["-e", &script])
        .output()
        .map_err(|e| format!("Failed to run osascript: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "Plugin installation failed (exit {}): {}",
            output.status.code().unwrap_or(-1),
            if !stderr.is_empty() { stderr.as_ref() } else { stdout.as_ref() }
        ));
    }

    // OBS 28+ on macOS only loads plugins from ~/Library/Application Support/obs-studio/plugins/,
    // not from the system-wide /Library path where the installer writes.
    // Copy the freshly installed bundle to the user path so OBS finds it.
    let system_plugin = PathBuf::from(
        "/Library/Application Support/obs-studio/plugins/obs-shaderfilter.plugin",
    );
    let user_plugin_dir = get_obs_plugin_dir();

    if system_plugin.exists() {
        std::fs::create_dir_all(&user_plugin_dir)
            .map_err(|e| format!("Failed to create user plugin dir: {}", e))?;

        let cp = Command::new("cp")
            .args([
                "-Rf",
                &system_plugin.to_string_lossy().to_string(),
                &user_plugin_dir.to_string_lossy().to_string(),
            ])
            .output()
            .map_err(|e| format!("Failed to copy plugin to user path: {}", e))?;

        if !cp.status.success() {
            return Err(format!(
                "Plugin installed but failed to copy to ~/Library path: {}",
                String::from_utf8_lossy(&cp.stderr)
            ));
        }
    }

    Ok(())
}

#[cfg(not(target_os = "macos"))]
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
    // Always download and run the installer so that stale/outdated plugin
    // versions (e.g. 2.4.3 built against an older OBS API) are upgraded.
    let pkg_path = download_shaderfilter().await?;
    extract_shaderfilter(&pkg_path)?;
    std::fs::remove_file(pkg_path).ok();

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
