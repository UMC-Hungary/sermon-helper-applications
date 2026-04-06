use serde::{Deserialize, Serialize};
use tauri::AppHandle;

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    html_url: String,
    body: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub release_url: String,
    pub release_notes: String,
}

#[tauri::command]
pub async fn check_for_updates(app: AppHandle) -> Result<Option<UpdateInfo>, String> {
    if cfg!(debug_assertions) {
        return Ok(None);
    }

    let current = app.package_info().version.to_string();
    if current == "0.0.0" {
        return Ok(None);
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get("https://api.github.com/repos/UMC-Hungary/sermon-helper-applications/releases/latest")
        .header("User-Agent", "sermon-helper-tauri")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("GitHub API returned {}", response.status()));
    }

    let release: GithubRelease = response.json().await.map_err(|e| e.to_string())?;

    let latest = release.tag_name.trim_start_matches('v').to_string();

    let current_ver = semver::Version::parse(&current).map_err(|e| e.to_string())?;
    let latest_ver = semver::Version::parse(&latest).map_err(|e| e.to_string())?;

    if latest_ver > current_ver {
        Ok(Some(UpdateInfo {
            current_version: current,
            latest_version: latest,
            release_url: release.html_url,
            release_notes: release.body.unwrap_or_default(),
        }))
    } else {
        Ok(None)
    }
}
