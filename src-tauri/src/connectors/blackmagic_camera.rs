use std::sync::Arc;

use blackmagic_camera::discovery;
pub use blackmagic_camera::discovery::Discovered;
pub use blackmagic_camera::Camera;
use blackmagic_camera::{Error, LivestreamPlatform, PlatformConfig, Trust};
use tauri::Emitter;
use tokio::sync::{broadcast, watch, Mutex, RwLock};
use tokio::time::Duration;

use super::{BlackmagicCameraConfig, ConnectorStatus};

/// ponytail: the camera's notification websocket carries no livestream property
/// (see blackmagic-camera/README.md), so both states are polled. Swap the record
/// half for `notify::watch` if this interval ever shows up as latency.
const POLL_INTERVAL: Duration = Duration::from_secs(2);

/// Snapshot of the camera's outputs, broadcast whenever either changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CameraState {
    pub is_streaming: bool,
    pub is_recording: bool,
}

pub struct BlackmagicCameraConnector {
    pub status: Arc<RwLock<ConnectorStatus>>,
    /// Last known record/livestream state; `None` while disconnected.
    pub state: Arc<RwLock<Option<CameraState>>>,
    /// Live camera client; `None` while disconnected.
    pub camera: Arc<Mutex<Option<Arc<Camera>>>>,
    pub status_tx: broadcast::Sender<ConnectorStatus>,
    pub state_tx: broadcast::Sender<CameraState>,
    stop_tx: Mutex<Option<watch::Sender<bool>>>,
}

impl BlackmagicCameraConnector {
    pub fn new() -> Self {
        let (status_tx, _) = broadcast::channel(16);
        let (state_tx, _) = broadcast::channel(16);
        Self {
            status: Arc::new(RwLock::new(ConnectorStatus::Disconnected)),
            state: Arc::new(RwLock::new(None)),
            camera: Arc::new(Mutex::new(None)),
            status_tx,
            state_tx,
            stop_tx: Mutex::new(None),
        }
    }

    pub async fn start(&self, config: BlackmagicCameraConfig, app: Option<tauri::AppHandle>) {
        self.stop_internal().await;

        let (stop_tx, stop_rx) = watch::channel(false);
        *self.stop_tx.lock().await = Some(stop_tx);

        let worker = Worker {
            config,
            app,
            status: Arc::clone(&self.status),
            state: Arc::clone(&self.state),
            camera: Arc::clone(&self.camera),
            status_tx: self.status_tx.clone(),
            state_tx: self.state_tx.clone(),
        };
        tauri::async_runtime::spawn(async move { worker.run(stop_rx).await });
    }

    pub async fn stop(&self) {
        self.stop_internal().await;
    }

    async fn stop_internal(&self) {
        let mut guard = self.stop_tx.lock().await;
        if let Some(tx) = guard.take() {
            let _ = tx.send(true);
        }
    }

    pub async fn get_status(&self) -> ConnectorStatus {
        self.status.read().await.clone()
    }

    pub async fn get_state(&self) -> Option<CameraState> {
        *self.state.read().await
    }

    /// The connected camera, for callers that drive it directly (record, livestream).
    pub async fn client(&self) -> Option<Arc<Camera>> {
        self.camera.lock().await.as_ref().map(Arc::clone)
    }
}

impl Default for BlackmagicCameraConnector {
    fn default() -> Self {
        Self::new()
    }
}

/// mDNS scan for cameras on the LAN. An empty list is a normal outcome: on
/// networks where multicast does not arrive, cameras are added by host instead.
pub async fn discover(timeout: Duration) -> Result<Vec<Discovered>, String> {
    discovery::browse(discovery::DEFAULT_SERVICE, timeout).await
}

/// A blank fingerprint means "not accepted yet" — trust whatever the camera
/// presents, so `presented_fingerprint()` can show the operator what to pin.
pub fn connect(config: &BlackmagicCameraConfig) -> Result<Camera, Error> {
    let trust = if config.fingerprint.is_empty() {
        Trust::OnFirstUse
    } else {
        Trust::Pinned(config.fingerprint.clone())
    };
    let auth = (!config.username.is_empty())
        .then_some((config.username.as_str(), config.password.as_str()));
    Camera::connect(&config.host, auth, trust)
}

/// Points the camera's livestream at YouTube. Prefers the camera's own YouTube
/// platform, which only needs the stream key; falls back to a custom RTMP URL on
/// models whose platform list has no YouTube entry.
pub async fn push_youtube(
    camera: &Camera,
    ingestion_address: &str,
    stream_key: &str,
) -> Result<LivestreamPlatform, Error> {
    let platforms = camera.livestream_platforms().await?;
    let Some(name) = platforms.iter().find(|p| is_youtube_rtmp(p)) else {
        let url = format!("{}/{stream_key}", ingestion_address.trim_end_matches('/'));
        return camera.stream_to(&url, None).await;
    };

    let config = camera.livestream_platform_config(name).await?;
    let platform = youtube_platform(config, stream_key)?;
    camera.set_livestream_active_platform(&platform).await?;
    Ok(platform)
}

/// Cameras name their entries "YouTube RTMP" and "YouTube SRT (Beta)", never a bare
/// "YouTube". Only the RTMP one takes the ingestion address and key we hold.
fn is_youtube_rtmp(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    name.contains("youtube") && !name.contains("srt")
}

/// The camera's YouTube platform entry filled in with our stream key: its default
/// quality profile when it names one, otherwise the first it lists.
fn youtube_platform(config: PlatformConfig, stream_key: &str) -> Result<LivestreamPlatform, Error> {
    let name = &config.platform;
    let quality = config
        .default_profile
        .clone()
        .or_else(|| config.profiles.first().map(|p| p.profile.clone()))
        .ok_or_else(|| Error::Unexpected(format!("platform {name} lists no quality profiles")))?;
    let server = config
        .servers
        .first()
        .map(|s| s.server.clone())
        .ok_or_else(|| Error::Unexpected(format!("platform {name} lists no servers")))?;
    Ok(LivestreamPlatform {
        platform: config.platform,
        server,
        key: Some(stream_key.to_string()),
        passphrase: None,
        quality,
        url: None,
    })
}

/// Livestreaming is absent on some models; that is "not streaming", not a failure.
async fn read_state(camera: &Camera) -> Result<CameraState, Error> {
    let is_recording = camera.record_state().await?.recording;
    let is_streaming = match camera.livestream_status().await {
        Ok(status) => status.status == "Streaming",
        Err(e) if e.is_unsupported() || matches!(e, Error::NotFound(_)) => false,
        Err(e) => return Err(e),
    };
    Ok(CameraState {
        is_streaming,
        is_recording,
    })
}

/// Outcome of one connection attempt: the caller asked us to stop, or the camera
/// dropped out and the loop should back off and retry.
enum Ended {
    ByCaller,
    ByError(String),
}

struct Worker {
    config: BlackmagicCameraConfig,
    app: Option<tauri::AppHandle>,
    status: Arc<RwLock<ConnectorStatus>>,
    state: Arc<RwLock<Option<CameraState>>>,
    camera: Arc<Mutex<Option<Arc<Camera>>>>,
    status_tx: broadcast::Sender<ConnectorStatus>,
    state_tx: broadcast::Sender<CameraState>,
}

impl Worker {
    async fn run(self, mut stop_rx: watch::Receiver<bool>) {
        let mut backoff = Duration::from_secs(5);
        loop {
            self.set_status(ConnectorStatus::Connecting).await;

            let ended = self.session(&mut stop_rx).await;
            *self.camera.lock().await = None;
            *self.state.write().await = None;

            match ended {
                Ended::ByCaller => {
                    self.set_status(ConnectorStatus::Disconnected).await;
                    return;
                }
                Ended::ByError(message) => {
                    self.set_status(ConnectorStatus::Error { message }).await;
                }
            }

            tokio::select! {
                () = tokio::time::sleep(backoff) => {}
                result = stop_rx.changed() => {
                    let _ = result;
                    self.set_status(ConnectorStatus::Disconnected).await;
                    return;
                }
            }
            backoff = (backoff * 2).min(Duration::from_secs(60));
        }
    }

    async fn session(&self, stop_rx: &mut watch::Receiver<bool>) -> Ended {
        let camera = match connect(&self.config) {
            Ok(camera) => Arc::new(camera),
            Err(e) => return Ended::ByError(e.to_string()),
        };

        // `product` is the reachability, auth and certificate check in one call.
        if let Err(e) = camera.product().await {
            return Ended::ByError(e.to_string());
        }

        *self.camera.lock().await = Some(Arc::clone(&camera));
        self.set_status(ConnectorStatus::Connected).await;

        loop {
            match read_state(&camera).await {
                Ok(current) => {
                    let mut guard = self.state.write().await;
                    if *guard != Some(current) {
                        *guard = Some(current);
                        let _ = self.state_tx.send(current);
                    }
                }
                Err(e) => return Ended::ByError(e.to_string()),
            }

            tokio::select! {
                () = tokio::time::sleep(POLL_INTERVAL) => {}
                result = stop_rx.changed() => {
                    let _ = result;
                    return Ended::ByCaller;
                }
            }
        }
    }

    async fn set_status(&self, new_status: ConnectorStatus) {
        *self.status.write().await = new_status;
        let current = self.status.read().await.clone();
        let _ = self.status_tx.send(current.clone());
        if let Some(app) = &self.app {
            if let Err(e) = app.emit("connector://blackmagic-camera-status", current) {
                tracing::warn!("Failed to emit Blackmagic camera status: {e}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn platform_config(json: serde_json::Value) -> PlatformConfig {
        serde_json::from_value(json).expect("platform config")
    }

    #[test]
    fn youtube_rtmp_is_picked_out_of_a_real_platform_list() {
        let platforms = [
            "YouTube RTMP",
            "YouTube SRT (Beta)",
            "Microsoft Teams",
            "Twitch",
        ];
        let picked: Vec<_> = platforms
            .iter()
            .filter(|p| is_youtube_rtmp(p))
            .collect();
        assert_eq!(picked, ["YouTube RTMP"].iter().collect::<Vec<_>>());
    }

    #[test]
    fn youtube_platform_prefers_the_default_profile() {
        let config = platform_config(serde_json::json!({
            "platform": "YouTube",
            "servers": [{ "server": "Primary", "url": "rtmp://a.rtmp.youtube.com/live2" }],
            "profiles": [{ "profile": "1080p" }, { "profile": "720p" }],
            "defaultProfile": "720p",
        }));
        let platform = youtube_platform(config, "abcd-1234").expect("platform");
        assert_eq!(platform.quality, "720p");
        assert_eq!(platform.server, "Primary");
        assert_eq!(platform.key.as_deref(), Some("abcd-1234"));
        // A platform entry carries the key, never a custom URL.
        assert_eq!(platform.url, None);
    }

    #[test]
    fn youtube_platform_falls_back_to_the_first_profile() {
        let config = platform_config(serde_json::json!({
            "platform": "YouTube",
            "servers": [{ "server": "Primary", "url": "rtmp://a.rtmp.youtube.com/live2" }],
            "profiles": [{ "profile": "1080p" }],
        }));
        assert_eq!(youtube_platform(config, "k").unwrap().quality, "1080p");
    }

    #[test]
    fn youtube_platform_reports_a_camera_with_nothing_to_select() {
        let config = platform_config(serde_json::json!({
            "platform": "YouTube",
            "servers": [{ "server": "Primary", "url": "rtmp://x" }],
            "profiles": [],
        }));
        assert!(youtube_platform(config, "k").is_err());
    }
}
