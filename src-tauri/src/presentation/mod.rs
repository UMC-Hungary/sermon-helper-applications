//! Presentation controller module.
//!
//! Provides cross-platform control of presentation applications:
//! - Windows: PowerPoint via COM automation
//! - macOS: Keynote and PowerPoint via AppleScript
//! - Linux: LibreOffice Impress via Python-UNO sidecar

pub mod controller;
pub mod types;

#[cfg(target_os = "windows")]
pub mod windows_powerpoint;

#[cfg(target_os = "macos")]
pub mod macos_keynote;

#[cfg(target_os = "macos")]
pub mod macos_powerpoint;

#[cfg(target_os = "linux")]
pub mod linux_impress;

use std::sync::Arc;

pub use controller::PresentationController;
pub use types::{PresentationApp, PresentationError, PresentationStatus};

/// Detect available presentation applications and return the best controller.
///
/// Priority:
/// - Windows: PowerPoint (only option)
/// - macOS: Keynote (preferred), then PowerPoint for Mac
/// - Linux: LibreOffice Impress (only option)
pub fn detect_controller() -> Arc<dyn PresentationController> {
    #[cfg(target_os = "windows")]
    {
        Arc::new(windows_powerpoint::WindowsPowerPointController::new())
    }

    #[cfg(target_os = "macos")]
    {
        // Check if Keynote is available (preferred on macOS)
        // Default to Keynote since it's the native macOS app
        // The user can configure this later if needed
        Arc::new(macos_keynote::MacosKeynoteController::new())
    }

    #[cfg(target_os = "linux")]
    {
        Arc::new(linux_impress::LinuxImpressController::new())
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Arc::new(NullController)
    }
}

/// Detect all available controllers on macOS where multiple apps may be present
#[cfg(target_os = "macos")]
pub fn detect_all_controllers() -> Vec<(PresentationApp, Arc<dyn PresentationController>)> {
    vec![
        (
            PresentationApp::Keynote,
            Arc::new(macos_keynote::MacosKeynoteController::new()),
        ),
        (
            PresentationApp::PowerPoint,
            Arc::new(macos_powerpoint::MacosPowerPointController::new()),
        ),
    ]
}

/// A no-op controller for unsupported platforms
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
struct NullController;

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
#[async_trait::async_trait]
impl PresentationController for NullController {
    async fn is_running(&self) -> bool {
        false
    }
    async fn open(&self, _file_path: &str) -> Result<(), PresentationError> {
        Err(PresentationError::PlatformNotSupported(
            "No presentation app supported on this platform".to_string(),
        ))
    }
    async fn start_slideshow(&self, _from_slide: Option<u32>) -> Result<(), PresentationError> {
        Err(PresentationError::PlatformNotSupported(
            "No presentation app supported on this platform".to_string(),
        ))
    }
    async fn stop_slideshow(&self) -> Result<(), PresentationError> {
        Err(PresentationError::PlatformNotSupported(
            "No presentation app supported on this platform".to_string(),
        ))
    }
    async fn next(&self) -> Result<(), PresentationError> {
        Err(PresentationError::PlatformNotSupported(
            "No presentation app supported on this platform".to_string(),
        ))
    }
    async fn previous(&self) -> Result<(), PresentationError> {
        Err(PresentationError::PlatformNotSupported(
            "No presentation app supported on this platform".to_string(),
        ))
    }
    async fn goto_slide(&self, _slide_number: u32) -> Result<(), PresentationError> {
        Err(PresentationError::PlatformNotSupported(
            "No presentation app supported on this platform".to_string(),
        ))
    }
    async fn blank_screen(&self) -> Result<(), PresentationError> {
        Err(PresentationError::PlatformNotSupported(
            "No presentation app supported on this platform".to_string(),
        ))
    }
    async fn white_screen(&self) -> Result<(), PresentationError> {
        Err(PresentationError::PlatformNotSupported(
            "No presentation app supported on this platform".to_string(),
        ))
    }
    async fn unblank(&self) -> Result<(), PresentationError> {
        Err(PresentationError::PlatformNotSupported(
            "No presentation app supported on this platform".to_string(),
        ))
    }
    async fn get_status(&self) -> Result<PresentationStatus, PresentationError> {
        Err(PresentationError::PlatformNotSupported(
            "No presentation app supported on this platform".to_string(),
        ))
    }
}
