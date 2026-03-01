use std::sync::Arc;

use futures_util::StreamExt;
use tauri::Emitter;
use tokio::sync::{broadcast, watch, Mutex, RwLock};
use tokio::time::Duration;

use super::{ConnectorStatus, ObsConfig};

pub struct ObsConnector {
    pub status: Arc<RwLock<ConnectorStatus>>,
    /// Broadcast channel — subscribe to receive every status change.
    pub status_tx: broadcast::Sender<ConnectorStatus>,
    stop_tx: Mutex<Option<watch::Sender<bool>>>,
}

impl ObsConnector {
    pub fn new() -> Self {
        let (status_tx, _) = broadcast::channel(16);
        Self {
            status: Arc::new(RwLock::new(ConnectorStatus::Disconnected)),
            status_tx,
            stop_tx: Mutex::new(None),
        }
    }

    pub async fn start(&self, config: ObsConfig, app: tauri::AppHandle) {
        self.stop_internal().await;

        let (stop_tx, stop_rx) = watch::channel(false);
        *self.stop_tx.lock().await = Some(stop_tx);

        let status = Arc::clone(&self.status);
        let status_tx = self.status_tx.clone();
        tauri::async_runtime::spawn(async move {
            run_obs_loop(config, app, status, status_tx, stop_rx).await;
        });
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
}

impl Default for ObsConnector {
    fn default() -> Self {
        Self::new()
    }
}

async fn set_status(
    status: &Arc<RwLock<ConnectorStatus>>,
    status_tx: &broadcast::Sender<ConnectorStatus>,
    app: &tauri::AppHandle,
    new_status: ConnectorStatus,
) {
    *status.write().await = new_status;
    let current = status.read().await.clone();
    // Broadcast to any server WS subscribers (ignored if no receivers)
    let _ = status_tx.send(current.clone());
    // Tauri IPC event for the local desktop window
    if let Err(e) = app.emit("connector://obs-status", current) {
        tracing::warn!("Failed to emit OBS status: {e}");
    }
}

async fn run_obs_loop(
    config: ObsConfig,
    app: tauri::AppHandle,
    status: Arc<RwLock<ConnectorStatus>>,
    status_tx: broadcast::Sender<ConnectorStatus>,
    mut stop_rx: watch::Receiver<bool>,
) {
    loop {
        set_status(&status, &status_tx, &app, ConnectorStatus::Connecting).await;

        let connect_result = tokio::select! {
            result = obws::Client::connect(
                config.host.as_str(),
                config.port,
                config.password.as_deref(),
            ) => result,
            result = stop_rx.changed() => {
                let _ = result;
                set_status(&status, &status_tx, &app, ConnectorStatus::Disconnected).await;
                return;
            }
        };

        match connect_result {
            Ok(client) => {
                set_status(&status, &status_tx, &app, ConnectorStatus::Connected).await;

                match client.events() {
                    Ok(events) => {
                        futures_util::pin_mut!(events);
                        'events: loop {
                            tokio::select! {
                                maybe_event = events.next() => {
                                    if maybe_event.is_none() {
                                        // OBS closed the connection
                                        break 'events;
                                    }
                                }
                                result = stop_rx.changed() => {
                                    let _ = result;
                                    set_status(&status, &status_tx, &app, ConnectorStatus::Disconnected).await;
                                    return;
                                }
                            }
                        }
                        set_status(&status, &status_tx, &app, ConnectorStatus::Disconnected).await;
                    }
                    Err(e) => {
                        set_status(
                            &status,
                            &status_tx,
                            &app,
                            ConnectorStatus::Error {
                                message: e.to_string(),
                            },
                        )
                        .await;
                    }
                }
            }
            Err(e) => {
                set_status(
                    &status,
                    &status_tx,
                    &app,
                    ConnectorStatus::Error {
                        message: e.to_string(),
                    },
                )
                .await;
            }
        }

        // Check stop before the retry delay
        if *stop_rx.borrow() {
            set_status(&status, &status_tx, &app, ConnectorStatus::Disconnected).await;
            return;
        }

        // Wait 5 seconds before retrying, bail early if stop is requested
        tokio::select! {
            () = tokio::time::sleep(Duration::from_secs(5)) => {}
            result = stop_rx.changed() => {
                let _ = result;
                set_status(&status, &status_tx, &app, ConnectorStatus::Disconnected).await;
                return;
            }
        }
    }
}
