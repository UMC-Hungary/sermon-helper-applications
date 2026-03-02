use std::sync::Arc;

use futures_util::StreamExt;
use obws::events::Event;
use tauri::Emitter;
use tokio::sync::{broadcast, watch, Mutex, RwLock};
use tokio::time::Duration;

use super::{ConnectorStatus, ObsConfig};

/// Snapshot of OBS output states, broadcast whenever either changes.
#[derive(Debug, Clone, Copy)]
pub struct ObsOutputState {
    pub is_streaming: bool,
    pub is_recording: bool,
}

pub struct ObsConnector {
    pub status: Arc<RwLock<ConnectorStatus>>,
    /// Last known streaming/recording state; `None` when OBS is disconnected.
    pub output_state: Arc<RwLock<Option<ObsOutputState>>>,
    /// Live OBS client; `None` when disconnected.
    pub client: Arc<Mutex<Option<Arc<obws::Client>>>>,
    /// Broadcast channel — subscribe to receive every connection status change.
    pub status_tx: broadcast::Sender<ConnectorStatus>,
    /// Broadcast channel — subscribe to receive streaming/recording state changes.
    pub output_state_tx: broadcast::Sender<ObsOutputState>,
    stop_tx: Mutex<Option<watch::Sender<bool>>>,
}

impl ObsConnector {
    pub fn new() -> Self {
        let (status_tx, _) = broadcast::channel(16);
        let (output_state_tx, _) = broadcast::channel(16);
        Self {
            status: Arc::new(RwLock::new(ConnectorStatus::Disconnected)),
            output_state: Arc::new(RwLock::new(None)),
            client: Arc::new(Mutex::new(None)),
            status_tx,
            output_state_tx,
            stop_tx: Mutex::new(None),
        }
    }

    pub async fn start(&self, config: ObsConfig, app: tauri::AppHandle) {
        self.stop_internal().await;

        let (stop_tx, stop_rx) = watch::channel(false);
        *self.stop_tx.lock().await = Some(stop_tx);

        let status = Arc::clone(&self.status);
        let output_state = Arc::clone(&self.output_state);
        let client_arc = Arc::clone(&self.client);
        let status_tx = self.status_tx.clone();
        let output_state_tx = self.output_state_tx.clone();
        tauri::async_runtime::spawn(async move {
            run_obs_loop(config, app, status, output_state, client_arc, status_tx, output_state_tx, stop_rx).await;
        });
    }

    pub async fn get_output_state(&self) -> Option<ObsOutputState> {
        *self.output_state.read().await
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
    let _ = status_tx.send(current.clone());
    if let Err(e) = app.emit("connector://obs-status", current) {
        tracing::warn!("Failed to emit OBS status: {e}");
    }
}

async fn run_obs_loop(
    config: ObsConfig,
    app: tauri::AppHandle,
    status: Arc<RwLock<ConnectorStatus>>,
    output_state: Arc<RwLock<Option<ObsOutputState>>>,
    client_arc: Arc<Mutex<Option<Arc<obws::Client>>>>,
    status_tx: broadcast::Sender<ConnectorStatus>,
    output_state_tx: broadcast::Sender<ObsOutputState>,
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
            Ok(raw_client) => {
                let client = Arc::new(raw_client);
                *client_arc.lock().await = Some(Arc::clone(&client));
                set_status(&status, &status_tx, &app, ConnectorStatus::Connected).await;

                // Query initial streaming/recording state, persist it, and broadcast it.
                let initial = query_output_state(&client).await;
                *output_state.write().await = Some(initial);
                let _ = output_state_tx.send(initial);
                // Use the persistent Arc as the mutable tracker during the session.
                let current_output = Arc::clone(&output_state);

                match client.events() {
                    Ok(events) => {
                        futures_util::pin_mut!(events);
                        'events: loop {
                            tokio::select! {
                                maybe_event = events.next() => {
                                    match maybe_event {
                                        None => break 'events,
                                        Some(event) => {
                                            handle_event(
                                                &event,
                                                &current_output,
                                                &output_state_tx,
                                            ).await;
                                        }
                                    }
                                }
                                result = stop_rx.changed() => {
                                    let _ = result;
                                    *client_arc.lock().await = None;
                                    *output_state.write().await = None;
                                    set_status(&status, &status_tx, &app, ConnectorStatus::Disconnected).await;
                                    return;
                                }
                            }
                        }
                        *client_arc.lock().await = None;
                        *output_state.write().await = None;
                        set_status(&status, &status_tx, &app, ConnectorStatus::Disconnected).await;
                    }
                    Err(e) => {
                        *client_arc.lock().await = None;
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

        if *stop_rx.borrow() {
            *client_arc.lock().await = None;
            *output_state.write().await = None;
            set_status(&status, &status_tx, &app, ConnectorStatus::Disconnected).await;
            return;
        }

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

async fn query_output_state(client: &obws::Client) -> ObsOutputState {
    let is_streaming = client
        .streaming()
        .status()
        .await
        .map(|s| s.active)
        .unwrap_or(false);
    let is_recording = client
        .recording()
        .status()
        .await
        .map(|s| s.active)
        .unwrap_or(false);
    ObsOutputState { is_streaming, is_recording }
}

async fn handle_event(
    event: &Event,
    current: &Arc<RwLock<Option<ObsOutputState>>>,
    output_state_tx: &broadcast::Sender<ObsOutputState>,
) {
    let updated = match event {
        Event::StreamStateChanged { active, .. } => {
            let mut guard = current.write().await;
            let state = guard.get_or_insert(ObsOutputState { is_streaming: false, is_recording: false });
            state.is_streaming = *active;
            Some(*state)
        }
        Event::RecordStateChanged { active, .. } => {
            let mut guard = current.write().await;
            let state = guard.get_or_insert(ObsOutputState { is_streaming: false, is_recording: false });
            state.is_recording = *active;
            Some(*state)
        }
        _ => None,
    };
    if let Some(state) = updated {
        let _ = output_state_tx.send(state);
    }
}
