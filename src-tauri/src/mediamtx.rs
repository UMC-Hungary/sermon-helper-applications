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
    /// Does nothing if already running.
    pub async fn start(&self, app: &tauri::AppHandle, data_dir: &Path, relay: &RelayConfig) -> Result<()> {
        let mut guard = self.child.lock().await;
        if guard.is_some() {
            return Ok(());
        }

        let binary = resolve_binary(app)?;
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

fn resolve_binary(app: &tauri::AppHandle) -> Result<PathBuf> {
    // Production: Tauri copies the binary (without triple suffix) into the resource dir.
    if let Ok(dir) = app.path().resource_dir() {
        let path = dir.join("mediamtx");
        if path.exists() {
            return Ok(path);
        }
    }

    // Development: binary lives in src-tauri/binaries/mediamtx-{target}.
    let dev = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("binaries")
        .join(concat!("mediamtx-", env!("TARGET")));
    if dev.exists() {
        return Ok(dev);
    }

    Err(anyhow!(
        "mediamtx binary not found. Run: bash scripts/download-mediamtx.sh"
    ))
}
