//! Display-free core bootstrap shared by the Tauri desktop app and the headless
//! server binary: embedded PostgreSQL → migrations → pool → connectors → Axum.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

use crate::connectors::{
    broadlink::BroadlinkConnector, facebook::FacebookConnector, obs::ObsConnector,
    vmix::VmixConnector, youtube::YouTubeConnector, ConnectorConfig, ConnectorStatus,
    FacebookConfig, YouTubeConfig,
};
use crate::{database, scheduler::CronScheduler, server};

/// Everything the core needs to boot. `new()` fills in stand-alone defaults;
/// the desktop app overwrites the fields it shares with `AppRuntime`.
pub struct CoreOptions {
    pub data_dir: PathBuf,
    pub port: u16,
    pub auth_token: Arc<RwLock<String>>,
    /// Directory served for unmatched routes. Falls back to the Tauri asset
    /// resolver when `None` and an `app_handle` is present.
    pub static_dir: Option<String>,
    /// `None` in headless mode — OAuth flows and desktop events are unavailable.
    pub app_handle: Option<tauri::AppHandle>,
    /// Grants read access to stored upstream credentials. Regenerated every run
    /// and never persisted; the desktop app receives it over Tauri IPC, which no
    /// remote client can reach. `METOCAST_ADMIN_TOKEN` overrides it for headless
    /// operators and tests.
    pub admin_token: Arc<String>,
    pub obs_connector: Arc<ObsConnector>,
    pub vmix_connector: Arc<VmixConnector>,
    pub youtube_connector: Arc<YouTubeConnector>,
    pub facebook_connector: Arc<FacebookConnector>,
    pub broadlink_connector: Arc<BroadlinkConnector>,
    pub youtube_config: Arc<RwLock<YouTubeConfig>>,
    pub facebook_config: Arc<RwLock<FacebookConfig>>,
    pub oauth_states: Arc<RwLock<HashMap<String, (String, Instant)>>>,
    /// Legacy desktop-store configs to import into `app_settings` on first boot.
    pub seed_configs: Vec<(String, serde_json::Value)>,
    #[cfg(target_os = "macos")]
    pub keynote_connector: Arc<crate::connectors::keynote::KeynoteConnector>,
}

impl CoreOptions {
    pub fn new(data_dir: PathBuf, port: u16, auth_token: Arc<RwLock<String>>) -> Self {
        Self {
            data_dir,
            port,
            auth_token,
            static_dir: default_static_dir(),
            app_handle: None,
            admin_token: Arc::new(
                std::env::var("METOCAST_ADMIN_TOKEN")
                    .unwrap_or_else(|_| uuid::Uuid::new_v4().to_string()),
            ),
            obs_connector: Arc::new(ObsConnector::new()),
            vmix_connector: Arc::new(VmixConnector::new()),
            youtube_connector: Arc::new(YouTubeConnector::new()),
            facebook_connector: Arc::new(FacebookConnector::new()),
            broadlink_connector: Arc::new(BroadlinkConnector::new()),
            youtube_config: Arc::new(RwLock::new(YouTubeConfig::default())),
            facebook_config: Arc::new(RwLock::new(FacebookConfig::default())),
            oauth_states: Arc::new(RwLock::new(HashMap::new())),
            seed_configs: Vec::new(),
            #[cfg(target_os = "macos")]
            keynote_connector: Arc::new(crate::connectors::keynote::KeynoteConnector::new()),
        }
    }

    /// Reuses the Arcs `AppRuntime` already holds, so config saved through a
    /// Tauri command is immediately visible to Axum routes.
    fn from_desktop(rt: &crate::AppRuntime, app: tauri::AppHandle, data_dir: PathBuf) -> Self {
        Self {
            app_handle: Some(app),
            obs_connector: Arc::clone(&rt.obs_connector),
            vmix_connector: Arc::clone(&rt.vmix_connector),
            youtube_connector: Arc::clone(&rt.youtube_connector),
            facebook_connector: Arc::clone(&rt.facebook_connector),
            broadlink_connector: Arc::clone(&rt.broadlink_connector),
            youtube_config: Arc::clone(&rt.youtube_config),
            facebook_config: Arc::clone(&rt.facebook_config),
            oauth_states: Arc::clone(&rt.oauth_states),
            admin_token: Arc::clone(&rt.admin_token),
            #[cfg(target_os = "macos")]
            keynote_connector: Arc::clone(&rt.keynote_connector),
            ..Self::new(data_dir, rt.server_port, Arc::clone(&rt.auth_token))
        }
    }
}

/// `METOCAST_STATIC_DIR` wins; otherwise dev builds serve the sibling `build/`
/// directory produced by `pnpm build`. Release builds embed the frontend in the
/// executable and serve it through the Tauri asset resolver instead.
fn default_static_dir() -> Option<String> {
    if let Ok(dir) = std::env::var("METOCAST_STATIC_DIR") {
        return Some(dir);
    }
    #[cfg(debug_assertions)]
    {
        let build_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()?
            .join("build");
        build_dir
            .is_dir()
            .then(|| build_dir.to_string_lossy().into_owned())
    }
    #[cfg(not(debug_assertions))]
    {
        None
    }
}

/// Desktop entry point: boots the core from the managed `AppRuntime`, using the
/// Tauri app data directory for PostgreSQL.
pub async fn start_from_app_runtime(
    app_runtime: &RwLock<crate::AppRuntime>,
    app: tauri::AppHandle,
) -> anyhow::Result<()> {
    use tauri::Manager;
    use tauri_plugin_store::StoreExt;

    let data_dir = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("./data"));

    // Connector configs used to live in the desktop store; carry any still-there
    // values over to `app_settings` on this boot.
    let seed_configs = app
        .store("app-settings.json")
        .map(|store| {
            LEGACY_CONFIG_KEYS
                .iter()
                .filter_map(|key| store.get(*key).map(|value| ((*key).to_string(), value)))
                .collect()
        })
        .unwrap_or_default();

    let options = {
        let rt = app_runtime.read().await;
        CoreOptions {
            seed_configs,
            ..CoreOptions::from_desktop(&rt, app, data_dir)
        }
    };
    start(options).await
}

const LEGACY_CONFIG_KEYS: [&str; 7] = [
    "obs_config",
    "vmix_config",
    "atem_config",
    "broadlink_config",
    "discord_config",
    "youtube_config",
    "facebook_config",
];

/// Boots the full stack and blocks until the server stops.
pub async fn start(options: CoreOptions) -> anyhow::Result<()> {
    tracing::info!("Starting embedded PostgreSQL in {:?}", options.data_dir);
    let embedded = database::embedded::EmbeddedDb::start(options.data_dir).await?;
    let connection_url = embedded.connection_url.clone();

    tracing::info!("Connecting pool to embedded PostgreSQL");
    let pool = database::create_pool(&connection_url).await?;

    tracing::info!("Running migrations");
    database::run_migrations(&pool).await?;

    // One-time import of configs the desktop app used to keep in its Tauri store.
    for (key, value) in &options.seed_configs {
        database::settings::seed_json(&pool, key, value).await?;
    }

    // Connector configs live in `app_settings` so headless runs read the same
    // values the UI writes over HTTP.
    *options.youtube_config.write().await =
        database::settings::get_json(&pool, "youtube_config").await;
    *options.facebook_config.write().await =
        database::settings::get_json(&pool, "facebook_config").await;

    // Auto-start connectors now that the pool is available. They check for stored
    // tokens/config and stay Disconnected if none exist.
    let obs_cfg: crate::connectors::ObsConfig =
        database::settings::get_json(&pool, "obs_config").await;
    if obs_cfg.is_configured() {
        options
            .obs_connector
            .start(obs_cfg, options.app_handle.clone())
            .await;
    }
    let yt_cfg = options.youtube_config.read().await.clone();
    if yt_cfg.is_configured() {
        options
            .youtube_connector
            .start(pool.clone(), yt_cfg, options.app_handle.clone())
            .await;
    }
    if options.facebook_config.read().await.is_configured() {
        options
            .facebook_connector
            .start(pool.clone(), options.app_handle.clone())
            .await;
    }

    // Initialise Broadlink status from DB: Connected if at least one device exists.
    let device_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM broadlink_devices")
        .fetch_one(&pool)
        .await
        .unwrap_or(0);
    if device_count > 0 {
        options
            .broadlink_connector
            .set_status(ConnectorStatus::Connected)
            .await;
    }

    tracing::info!("Starting Axum on port {}", options.port);
    server::build_and_serve(
        pool,
        options.auth_token,
        connection_url,
        options.port,
        options.static_dir,
        options.obs_connector,
        options.vmix_connector,
        options.youtube_connector,
        options.facebook_connector,
        options.broadlink_connector,
        options.youtube_config,
        options.facebook_config,
        options.oauth_states,
        options.app_handle,
        options.admin_token,
        Arc::new(CronScheduler::new()),
        #[cfg(target_os = "macos")]
        options.keynote_connector,
    )
    .await?;

    embedded.stop().await?;

    Ok(())
}
