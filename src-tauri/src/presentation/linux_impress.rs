//! Linux LibreOffice Impress controller using a Python-UNO sidecar script.
//!
//! Communicates with `impress_controller.py` via stdin/stdout JSON messages.
//! The sidecar connects to LibreOffice via UNO socket.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

use super::controller::PresentationController;
use super::types::{PresentationApp, PresentationError, PresentationStatus};

/// Command sent to the Python sidecar
#[derive(Debug, Serialize)]
#[serde(tag = "command")]
#[serde(rename_all = "snake_case")]
enum SidecarCommand {
    IsRunning,
    Open { file_path: String },
    StartSlideshow { from_slide: Option<u32> },
    StopSlideshow,
    Next,
    Previous,
    GotoSlide { slide_number: u32 },
    BlankScreen,
    WhiteScreen,
    Unblank,
    CloseAll,
    CloseLatest,
    GetStatus,
    Quit,
}

/// Response from the Python sidecar
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct SidecarResponse {
    success: bool,
    error: Option<String>,
    #[serde(default)]
    data: Option<SidecarStatusData>,
    #[serde(default)]
    running: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct SidecarStatusData {
    slideshow_active: bool,
    current_slide: Option<u32>,
    total_slides: Option<u32>,
    blanked: bool,
}

pub struct LinuxImpressController {
    sidecar: Mutex<Option<SidecarProcess>>,
}

struct SidecarProcess {
    child: Child,
    stdin: tokio::process::ChildStdin,
    reader: BufReader<tokio::process::ChildStdout>,
}

impl LinuxImpressController {
    pub fn new() -> Self {
        Self {
            sidecar: Mutex::new(None),
        }
    }

    /// Find the sidecar script path
    fn find_sidecar_script() -> Option<String> {
        // Look relative to the executable
        if let Ok(exe_path) = std::env::current_exe() {
            let dir = exe_path.parent()?;

            // Check alongside executable (production)
            let beside_exe = dir.join("impress_controller.py");
            if beside_exe.exists() {
                return Some(beside_exe.to_string_lossy().to_string());
            }

            // Check in sidecars subdirectory
            let in_sidecars = dir.join("sidecars").join("impress_controller.py");
            if in_sidecars.exists() {
                return Some(in_sidecars.to_string_lossy().to_string());
            }
        }

        // Dev mode: check in src-tauri/sidecars
        let dev_path = std::path::PathBuf::from("sidecars/impress_controller.py");
        if dev_path.exists() {
            return Some(dev_path.to_string_lossy().to_string());
        }

        None
    }

    async fn ensure_sidecar(&self) -> Result<(), PresentationError> {
        let mut guard = self.sidecar.lock().await;

        // Check if existing process is still alive
        if let Some(ref mut proc) = *guard {
            match proc.child.try_wait() {
                Ok(Some(_)) => {
                    // Process exited, clear it
                    *guard = None;
                }
                Ok(None) => {
                    // Still running
                    return Ok(());
                }
                Err(_) => {
                    *guard = None;
                }
            }
        }

        let script_path = Self::find_sidecar_script().ok_or_else(|| {
            PresentationError::AutomationError(
                "impress_controller.py sidecar not found".to_string(),
            )
        })?;

        let mut child = Command::new("python3")
            .arg(&script_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| {
                PresentationError::AutomationError(format!(
                    "Failed to start impress sidecar: {}",
                    e
                ))
            })?;

        let stdin = child.stdin.take().ok_or_else(|| {
            PresentationError::AutomationError("Failed to get sidecar stdin".to_string())
        })?;

        let stdout = child.stdout.take().ok_or_else(|| {
            PresentationError::AutomationError("Failed to get sidecar stdout".to_string())
        })?;

        let reader = BufReader::new(stdout);

        *guard = Some(SidecarProcess {
            child,
            stdin,
            reader,
        });

        Ok(())
    }

    async fn send_command(
        &self,
        cmd: SidecarCommand,
    ) -> Result<SidecarResponse, PresentationError> {
        self.ensure_sidecar().await?;

        let mut guard = self.sidecar.lock().await;
        let proc = guard.as_mut().ok_or_else(|| {
            PresentationError::AutomationError("Sidecar not running".to_string())
        })?;

        let json = serde_json::to_string(&cmd).map_err(|e| {
            PresentationError::AutomationError(format!("Failed to serialize command: {}", e))
        })?;

        proc.stdin
            .write_all(json.as_bytes())
            .await
            .map_err(|e| {
                PresentationError::AutomationError(format!("Failed to write to sidecar: {}", e))
            })?;
        proc.stdin
            .write_all(b"\n")
            .await
            .map_err(|e| {
                PresentationError::AutomationError(format!("Failed to write to sidecar: {}", e))
            })?;
        proc.stdin.flush().await.map_err(|e| {
            PresentationError::AutomationError(format!("Failed to flush sidecar stdin: {}", e))
        })?;

        let mut line = String::new();
        proc.reader.read_line(&mut line).await.map_err(|e| {
            PresentationError::AutomationError(format!("Failed to read from sidecar: {}", e))
        })?;

        let response: SidecarResponse = serde_json::from_str(line.trim()).map_err(|e| {
            PresentationError::AutomationError(format!(
                "Failed to parse sidecar response: {} (raw: {})",
                e,
                line.trim()
            ))
        })?;

        if !response.success {
            return Err(PresentationError::AutomationError(
                response.error.unwrap_or_else(|| "Unknown error".to_string()),
            ));
        }

        Ok(response)
    }
}

#[async_trait]
impl PresentationController for LinuxImpressController {
    async fn is_running(&self) -> bool {
        // Check if soffice process is running
        std::process::Command::new("pgrep")
            .args(["-x", "soffice.bin"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    async fn open(&self, file_path: &str) -> Result<(), PresentationError> {
        if !std::path::Path::new(file_path).exists() {
            return Err(PresentationError::FileNotFound(file_path.to_string()));
        }

        self.send_command(SidecarCommand::Open {
            file_path: file_path.to_string(),
        })
        .await?;
        Ok(())
    }

    async fn start_slideshow(&self, from_slide: Option<u32>) -> Result<(), PresentationError> {
        self.send_command(SidecarCommand::StartSlideshow { from_slide })
            .await?;
        Ok(())
    }

    async fn stop_slideshow(&self) -> Result<(), PresentationError> {
        self.send_command(SidecarCommand::StopSlideshow).await?;
        Ok(())
    }

    async fn next(&self) -> Result<(), PresentationError> {
        self.send_command(SidecarCommand::Next).await?;
        Ok(())
    }

    async fn previous(&self) -> Result<(), PresentationError> {
        self.send_command(SidecarCommand::Previous).await?;
        Ok(())
    }

    async fn goto_slide(&self, slide_number: u32) -> Result<(), PresentationError> {
        self.send_command(SidecarCommand::GotoSlide { slide_number })
            .await?;
        Ok(())
    }

    async fn blank_screen(&self) -> Result<(), PresentationError> {
        self.send_command(SidecarCommand::BlankScreen).await?;
        Ok(())
    }

    async fn white_screen(&self) -> Result<(), PresentationError> {
        self.send_command(SidecarCommand::WhiteScreen).await?;
        Ok(())
    }

    async fn unblank(&self) -> Result<(), PresentationError> {
        self.send_command(SidecarCommand::Unblank).await?;
        Ok(())
    }

    async fn close_all(&self) -> Result<(), PresentationError> {
        self.send_command(SidecarCommand::CloseAll).await?;
        Ok(())
    }

    async fn close_latest(&self) -> Result<(), PresentationError> {
        self.send_command(SidecarCommand::CloseLatest).await?;
        Ok(())
    }

    async fn get_status(&self) -> Result<PresentationStatus, PresentationError> {
        let is_running = self.is_running().await;
        if !is_running {
            return Ok(PresentationStatus {
                app: PresentationApp::Impress,
                app_running: false,
                slideshow_active: false,
                current_slide: None,
                total_slides: None,
                current_slide_title: None,
                blanked: false,
            });
        }

        match self.send_command(SidecarCommand::GetStatus).await {
            Ok(response) => {
                if let Some(data) = response.data {
                    Ok(PresentationStatus {
                        app: PresentationApp::Impress,
                        app_running: true,
                        slideshow_active: data.slideshow_active,
                        current_slide: data.current_slide,
                        total_slides: data.total_slides,
                        current_slide_title: None,
                        blanked: data.blanked,
                    })
                } else {
                    Ok(PresentationStatus {
                        app: PresentationApp::Impress,
                        app_running: true,
                        slideshow_active: false,
                        current_slide: None,
                        total_slides: None,
                        current_slide_title: None,
                        blanked: false,
                    })
                }
            }
            Err(_) => Ok(PresentationStatus {
                app: PresentationApp::Impress,
                app_running: true,
                slideshow_active: false,
                current_slide: None,
                total_slides: None,
                current_slide_title: None,
                blanked: false,
            }),
        }
    }
}

impl Drop for LinuxImpressController {
    fn drop(&mut self) {
        // Try to gracefully stop the sidecar
        if let Ok(mut guard) = self.sidecar.try_lock() {
            if let Some(ref mut proc) = *guard {
                let _ = proc.child.start_kill();
            }
        }
    }
}
