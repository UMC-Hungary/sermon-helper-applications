use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::Manager;
use tokio::process::Child;
use tokio::sync::Mutex;

pub const RTMP_PORT: u16 = 1935;
pub const HLS_PORT: u16 = 8888;
pub const API_PORT: u16 = 9997;
/// RTMP path that OBS should stream to (app name, no stream key).
pub const STREAM_PATH: &str = "live";

const MEDIAMTX_VERSION: &str = "v1.12.2";

// Platform-specific names used for both the archive download and the binary.
#[cfg(target_os = "windows")]
const MEDIAMTX_OS: &str = "windows";
#[cfg(target_os = "macos")]
const MEDIAMTX_OS: &str = "darwin";
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
const MEDIAMTX_OS: &str = "linux";

#[cfg(target_arch = "aarch64")]
const MEDIAMTX_ARCH: &str = "arm64";
#[cfg(not(target_arch = "aarch64"))]
const MEDIAMTX_ARCH: &str = "amd64";

/// Name of the mediamtx binary on the current platform.
#[cfg(target_os = "windows")]
const MEDIAMTX_BINARY: &str = "mediamtx.exe";
#[cfg(not(target_os = "windows"))]
const MEDIAMTX_BINARY: &str = "mediamtx";

fn default_true() -> bool {
    true
}

/// Destination URLs for the multi-stream relay.
/// Empty strings mean that destination is disabled.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayConfig {
    pub youtube_rtmp_url: String,
    pub facebook_rtmp_url: String,
    /// When `true` (default), mediamtx binds RTMP on all interfaces (:1935).
    /// When `false`, it binds only on loopback (127.0.0.1:1935) so other LAN
    /// devices cannot connect.
    #[serde(default = "default_true")]
    pub rtmp_restream_enabled: bool,
}

impl Default for RelayConfig {
    fn default() -> Self {
        Self {
            youtube_rtmp_url: String::new(),
            facebook_rtmp_url: String::new(),
            rtmp_restream_enabled: true,
        }
    }
}

/// Build the mediamtx YAML config (v1.x flat-key format), optionally including
/// a FFmpeg runOnReady relay command when one or both relay destinations are set.
fn build_config(relay: &RelayConfig) -> String {
    let yt = relay.youtube_rtmp_url.trim();
    let fb = relay.facebook_rtmp_url.trim();
    let rtmp_addr = if relay.rtmp_restream_enabled { ":1935" } else { "127.0.0.1:1935" };

    // mediamtx v1.x uses flat top-level keys, not nested objects.
    let mut config = format!("\
logLevel: warn
api: yes
apiAddress: :9997
rtmp: yes
rtmpAddress: {rtmp_addr}
hls: yes
hlsAddress: :8888
hlsAllowOrigin: '*'
paths:
  live:
    source: publisher
");

    if !yt.is_empty() || !fb.is_empty() {
        let mut cmd = "ffmpeg -re -i rtmp://localhost:1935/live".to_string();
        if !yt.is_empty() {
            cmd.push_str(&format!(" -c copy -f flv {yt}"));
        }
        if !fb.is_empty() {
            cmd.push_str(&format!(" -c copy -f flv {fb}"));
        }
        // v1.x hook name is runOnReady (fires when a publisher connects).
        config.push_str(&format!("    runOnReady: {cmd}\n"));
        config.push_str("    runOnReadyRestart: yes\n");
    }

    config
}

pub struct MediamtxManager {
    child: Mutex<Option<Child>>,
}

impl MediamtxManager {
    pub fn new() -> Self {
        Self {
            child: Mutex::new(None),
        }
    }

    /// Start mediamtx. Writes a config file into `data_dir` and spawns the process.
    /// If the binary is not present it is downloaded automatically.
    /// Does nothing if already running.
    pub async fn start(&self, app: &tauri::AppHandle, data_dir: &Path, relay: &RelayConfig) -> Result<()> {
        let mut guard = self.child.lock().await;
        if guard.is_some() {
            return Ok(());
        }

        let binary = match resolve_binary(app, data_dir) {
            Some(p) => p,
            None => {
                tracing::info!("mediamtx binary not found; downloading {MEDIAMTX_VERSION}…");
                download_binary(data_dir).await?
            }
        };

        let config_path = data_dir.join("mediamtx.yml");
        std::fs::write(&config_path, build_config(relay))
            .map_err(|e| anyhow!("Failed to write mediamtx config: {e}"))?;

        let child = tokio::process::Command::new(&binary)
            .arg(&config_path)
            .kill_on_drop(true)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| anyhow!("Failed to spawn mediamtx at {binary:?}: {e}"))?;

        *guard = Some(child);
        tracing::info!(
            "mediamtx started (RTMP :{RTMP_PORT}, HLS :{HLS_PORT}, path /{STREAM_PATH})"
        );
        Ok(())
    }

    pub async fn stop(&self) {
        let mut guard = self.child.lock().await;
        if let Some(mut child) = guard.take() {
            let _ = child.kill().await;
            tracing::info!("mediamtx stopped");
        }
    }

    /// Stop and restart mediamtx with an updated relay config.
    pub async fn restart(&self, app: &tauri::AppHandle, data_dir: &Path, relay: &RelayConfig) -> Result<()> {
        self.stop().await;
        self.start(app, data_dir, relay).await
    }
}

/// Look for the mediamtx binary in the following order:
/// 1. App data dir — previously auto-downloaded.
/// 2. Tauri resource dir — legacy bundled path (kept for safety).
/// 3. Development path: src-tauri/binaries/mediamtx-{target}[.exe].
fn resolve_binary(app: &tauri::AppHandle, data_dir: &Path) -> Option<PathBuf> {
    // 1. Previously downloaded into app data dir.
    let path = data_dir.join(MEDIAMTX_BINARY);
    if path.exists() {
        return Some(path);
    }

    // 2. Bundled in Tauri resource dir (legacy).
    if let Ok(dir) = app.path().resource_dir() {
        let path = dir.join(MEDIAMTX_BINARY);
        if path.exists() {
            return Some(path);
        }
    }

    // 3. Development: src-tauri/binaries/mediamtx-{target}[.exe].
    #[cfg(target_os = "windows")]
    let dev_name = concat!("mediamtx-", env!("TARGET"), ".exe");
    #[cfg(not(target_os = "windows"))]
    let dev_name = concat!("mediamtx-", env!("TARGET"));

    let dev = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("binaries")
        .join(dev_name);
    if dev.exists() {
        return Some(dev);
    }

    None
}

/// Download the mediamtx binary for the current platform into `dest_dir`.
/// Uses `tar` (available on all platforms — Windows 10+ ships BSD tar) to
/// extract the binary from the release archive.
async fn download_binary(dest_dir: &Path) -> Result<PathBuf> {
    std::fs::create_dir_all(dest_dir)?;

    #[cfg(target_os = "windows")]
    let archive_ext = "zip";
    #[cfg(not(target_os = "windows"))]
    let archive_ext = "tar.gz";

    let archive_name = format!(
        "mediamtx_{MEDIAMTX_VERSION}_{MEDIAMTX_OS}_{MEDIAMTX_ARCH}.{archive_ext}"
    );
    let url = format!(
        "https://github.com/bluenviron/mediamtx/releases/download/{MEDIAMTX_VERSION}/{archive_name}"
    );

    tracing::info!("Downloading {url}");
    let response = reqwest::get(&url)
        .await
        .map_err(|e| anyhow!("Download request failed: {e}"))?;
    if !response.status().is_success() {
        return Err(anyhow!("Download failed: HTTP {}", response.status()));
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|e| anyhow!("Download read failed: {e}"))?;

    let archive_path = dest_dir.join(&archive_name);
    std::fs::write(&archive_path, &bytes)
        .map_err(|e| anyhow!("Failed to save archive: {e}"))?;

    let binary_path = dest_dir.join(MEDIAMTX_BINARY);

    // tar is available on all platforms (Windows 10+ ships BSD tar that handles ZIP).
    #[cfg(target_os = "windows")]
    let tar_flags = "xf"; // ZIP — no decompression flag needed
    #[cfg(not(target_os = "windows"))]
    let tar_flags = "xzf"; // tar.gz

    let status = tokio::process::Command::new("tar")
        .arg(tar_flags)
        .arg(&archive_path)
        .arg("-C")
        .arg(dest_dir)
        .arg(MEDIAMTX_BINARY)
        .status()
        .await
        .map_err(|e| anyhow!("Failed to run tar: {e}"))?;

    let _ = std::fs::remove_file(&archive_path);

    if !status.success() {
        return Err(anyhow!("Failed to extract mediamtx archive (tar exited with {status})"));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&binary_path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&binary_path, perms)?;
    }

    tracing::info!("mediamtx saved to {binary_path:?}");
    Ok(binary_path)
}
