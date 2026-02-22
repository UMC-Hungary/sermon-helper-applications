mod commands;
mod database;
mod models;
mod server;

use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;
use tauri_plugin_store::StoreExt;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct AppRuntime {
    pub mode: String,
    pub server_port: u16,
    pub client_url: Option<String>,
    pub auth_token: Arc<RwLock<String>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(|app| {
            let handle = app.handle().clone();

            tokio::spawn(async move {
                if let Err(e) = start_backend(handle).await {
                    tracing::error!("Backend startup failed: {e}");
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::token::get_token,
            commands::token::refresh_token,
            commands::server::get_server_port,
            commands::server::get_app_mode,
            commands::server::set_app_mode,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn start_backend(app: tauri::AppHandle) -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let store = app.store("app-settings.json")?;

    let mode = store
        .get("mode")
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| "server".to_string());

    let auth_token = store
        .get("auth_token")
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| {
            let t = Uuid::new_v4().to_string();
            store.set("auth_token", serde_json::Value::String(t.clone()));
            let _ = store.save();
            t
        });

    let client_url = store
        .get("server_url")
        .and_then(|v| v.as_str().map(String::from));

    let port: u16 = store
        .get("server_port")
        .and_then(|v| v.as_u64())
        .map(|p| p as u16)
        .unwrap_or(3737);

    let auth_token_arc = Arc::new(RwLock::new(auth_token));

    let runtime = Arc::new(RwLock::new(AppRuntime {
        mode: mode.clone(),
        server_port: port,
        client_url,
        auth_token: auth_token_arc.clone(),
    }));

    app.manage(runtime);

    if mode == "server" {
        let data_dir = app
            .path()
            .app_data_dir()
            .unwrap_or_else(|_| PathBuf::from("./data"));

        tracing::info!("Starting embedded PostgreSQL in {data_dir:?}");
        let embedded = database::embedded::EmbeddedDb::start(data_dir).await?;
        let connection_url = embedded.connection_url.clone();

        tracing::info!("Connecting pool to {connection_url}");
        let pool = database::create_pool(&connection_url).await?;

        tracing::info!("Running migrations");
        database::run_migrations(&pool).await?;

        let static_dir = app
            .path()
            .resource_dir()
            .ok()
            .map(|p| p.join("_up_").to_string_lossy().into_owned());

        tracing::info!("Starting Axum on port {port}");
        server::build_and_serve(pool, auth_token_arc, connection_url, port, static_dir).await?;

        embedded.stop().await?;
    }

    Ok(())
}
