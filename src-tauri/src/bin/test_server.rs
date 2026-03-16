//! Standalone HTTP/WS server for E2E testing.
//!
//! Does NOT require Tauri or a display — suitable for headless CI runners.
//! Starts embedded PostgreSQL and Axum, then blocks until SIGINT/SIGTERM.
//!
//! Required env var:
//!   TAURI_AUTH_TOKEN  — bearer token the E2E tests must present
//!
//! Optional env var:
//!   TEST_SERVER_PORT  — port to listen on (default: 3738)

use std::sync::Arc;
use tokio::sync::RwLock;

use sermon_helper_tauri_lib::{
    connectors::{
        broadlink::BroadlinkConnector, facebook::FacebookConnector, obs::ObsConnector,
        vmix::VmixConnector, youtube::YouTubeConnector, FacebookConfig, YouTubeConfig,
    },
    database,
    scheduler::CronScheduler,
    server,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let token = std::env::var("TAURI_AUTH_TOKEN")
        .expect("TAURI_AUTH_TOKEN env var must be set to run the test server");

    let data_dir = std::path::PathBuf::from("./test-server-data");
    std::fs::create_dir_all(&data_dir)?;

    tracing::info!("Starting embedded PostgreSQL in {data_dir:?}");
    let embedded = database::embedded::EmbeddedDb::start(data_dir).await?;
    let connection_url = embedded.connection_url.clone();

    tracing::info!("Connecting pool");
    let pool = database::create_pool(&connection_url).await?;

    tracing::info!("Running migrations");
    database::run_migrations(&pool).await?;

    let auth_token = Arc::new(RwLock::new(token));

    let obs_connector = Arc::new(ObsConnector::new());
    let vmix_connector = Arc::new(VmixConnector::new());
    let youtube_connector = Arc::new(YouTubeConnector::new());
    let facebook_connector = Arc::new(FacebookConnector::new());
    let broadlink_connector = Arc::new(BroadlinkConnector::new());
    let youtube_config = Arc::new(RwLock::new(YouTubeConfig::default()));
    let facebook_config = Arc::new(RwLock::new(FacebookConfig::default()));
    let oauth_states = Arc::new(RwLock::new(
        std::collections::HashMap::<String, (String, std::time::Instant)>::new(),
    ));
    let cron_scheduler = Arc::new(CronScheduler::new());

    let port: u16 = std::env::var("TEST_SERVER_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3738);

    tracing::info!("Starting Axum on port {port}");
    server::build_and_serve(
        pool,
        auth_token,
        connection_url,
        port,
        None, // no static file serving in test mode
        obs_connector,
        vmix_connector,
        youtube_connector,
        facebook_connector,
        broadlink_connector,
        youtube_config,
        facebook_config,
        oauth_states,
        None, // no AppHandle — OAuth flows are unavailable in test mode
        cron_scheduler,
        #[cfg(target_os = "macos")]
        Arc::new(sermon_helper_tauri_lib::connectors::keynote::KeynoteConnector::new()),
    )
    .await?;

    embedded.stop().await?;
    Ok(())
}
