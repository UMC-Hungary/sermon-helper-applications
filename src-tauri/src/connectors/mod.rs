use serde::{Deserialize, Serialize};

pub mod atem;
pub mod blackmagic_camera;
pub mod broadlink;
pub mod discord;
pub mod facebook;
#[cfg(target_os = "macos")]
pub mod keynote;
pub mod obs;
pub mod vmix;
pub mod youtube;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ConnectorStatus {
    Disconnected,
    Connecting,
    Connected,
    Error { message: String },
}

/// Shared contract: every connector config must report whether it has been
/// fully filled in (enabled + all required credential fields non-empty).
pub trait ConnectorConfig {
    fn is_configured(&self) -> bool;
}

// ── Config structs ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObsConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
}

impl Default for ObsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            host: "localhost".to_string(),
            port: 4455,
            password: None,
        }
    }
}

impl ConnectorConfig for ObsConfig {
    fn is_configured(&self) -> bool {
        self.enabled && !self.host.is_empty() && self.port > 0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VmixConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
}

impl Default for VmixConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            host: "localhost".to_string(),
            port: 8088,
        }
    }
}

impl ConnectorConfig for VmixConfig {
    fn is_configured(&self) -> bool {
        self.enabled && !self.host.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtemConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
}

impl Default for AtemConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            host: String::new(),
            port: 9910,
        }
    }
}

impl ConnectorConfig for AtemConfig {
    fn is_configured(&self) -> bool {
        self.enabled && !self.host.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YouTubeConfig {
    pub enabled: bool,
    pub client_id: String,
    pub client_secret: String,
}

impl Default for YouTubeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            client_id: String::new(),
            client_secret: String::new(),
        }
    }
}

impl ConnectorConfig for YouTubeConfig {
    fn is_configured(&self) -> bool {
        self.enabled && !self.client_id.is_empty() && !self.client_secret.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FacebookConfig {
    pub enabled: bool,
    pub app_id: String,
    pub app_secret: String,
    pub page_id: String,
}

impl Default for FacebookConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            app_id: String::new(),
            app_secret: String::new(),
            page_id: String::new(),
        }
    }
}

impl ConnectorConfig for FacebookConfig {
    fn is_configured(&self) -> bool {
        self.enabled && !self.app_id.is_empty() && !self.app_secret.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscordConfig {
    pub enabled: bool,
    pub webhook_url: String,
}

impl Default for DiscordConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            webhook_url: String::new(),
        }
    }
}

impl ConnectorConfig for DiscordConfig {
    fn is_configured(&self) -> bool {
        self.enabled && !self.webhook_url.is_empty()
    }
}

/// Szentiras.eu Bible API. Every `/api/*` call needs a free API key, so the
/// connector exists purely to hold it.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SzentirasConfig {
    pub enabled: bool,
    pub api_key: String,
}

impl ConnectorConfig for SzentirasConfig {
    fn is_configured(&self) -> bool {
        self.enabled && !self.api_key.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BroadlinkConfig {
    pub enabled: bool,
}

impl ConnectorConfig for BroadlinkConfig {
    fn is_configured(&self) -> bool {
        self.enabled
    }
}

/// Blackmagic camera (REST + livestream control). `fingerprint` pins the camera's
/// self-signed certificate: blank means trust-on-first-use, which is also how the
/// operator learns the fingerprint to pin. Plain-HTTP cameras never need one.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BlackmagicCameraConfig {
    pub enabled: bool,
    /// `http://Cinema-Camera-6K.local`, an IP, or a bare host for HTTPS.
    pub host: String,
    pub fingerprint: String,
    pub username: String,
    pub password: String,
}

impl ConnectorConfig for BlackmagicCameraConfig {
    fn is_configured(&self) -> bool {
        self.enabled && !self.host.is_empty()
    }
}
