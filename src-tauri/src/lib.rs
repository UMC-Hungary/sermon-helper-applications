mod commands;

// Models, database, and server are desktop-only (depend on sqlx, PostgreSQL, Axum).
#[cfg(desktop)]
mod models;
#[cfg(desktop)]
mod database;
#[cfg(desktop)]
mod server;

use std::sync::Arc;
use tauri::Manager;
use tauri_plugin_store::StoreExt;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct AppRuntime {
    pub mode: Option<String>,
    pub server_port: u16,
    pub client_url: Option<String>,
    pub auth_token: Arc<RwLock<String>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build());

    #[cfg(desktop)]
    let builder = builder.plugin(tauri_plugin_dialog::init());

    // Pass generate_handler! directly into invoke_handler so the closure type
    // is inferred at the call site — storing it in a let binding loses the type.
    #[cfg(desktop)]
    let builder = builder.invoke_handler(tauri::generate_handler![
        commands::collections::save_bruno_collection,
        commands::token::get_token,
        commands::token::refresh_token,
        commands::server::get_server_port,
        commands::server::get_app_mode,
        commands::server::set_app_mode,
        commands::server::complete_setup,
        commands::server::get_client_url,
        commands::server::get_client_token,
        commands::server::reset_setup,
    ]);

    // Mobile is client-only — no server or Bruno collection commands.
    #[cfg(mobile)]
    let builder = builder.invoke_handler(tauri::generate_handler![
        commands::token::get_token,
        commands::token::refresh_token,
        commands::server::get_server_port,
        commands::server::get_app_mode,
        commands::server::set_app_mode,
        commands::server::complete_setup,
        commands::server::get_client_url,
        commands::server::get_client_token,
        commands::server::reset_setup,
    ]);

    builder
        .setup(|app| {
            tracing_subscriber::fmt()
                .with_env_filter(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| "info".into()),
                )
                .init();

            // Load settings synchronously so AppRuntime is managed before the
            // UI can call any Tauri command.
            let store = app.store("app-settings.json")?;

            let mode = store
                .get("mode")
                .and_then(|v| v.as_str().map(String::from));

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

            // Managed here — guaranteed to exist before any invoke() call.
            app.manage(runtime);

            // Only start the server if mode was already configured as "server".
            // Server mode is desktop-only (requires embedded PostgreSQL + Axum).
            #[cfg(desktop)]
            if mode.as_deref() == Some("server") {
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = start_server(handle, auth_token_arc, port).await {
                        tracing::error!("Backend startup failed: {e}");
                    }
                });
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(desktop)]
pub(crate) async fn start_server(
    app: tauri::AppHandle,
    auth_token: Arc<RwLock<String>>,
    port: u16,
) -> anyhow::Result<()> {
    use std::path::PathBuf;

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
    server::build_and_serve(pool, auth_token, connection_url, port, static_dir).await?;

    embedded.stop().await?;

    Ok(())
}
