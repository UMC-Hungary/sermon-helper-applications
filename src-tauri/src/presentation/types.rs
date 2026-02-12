use serde::{Deserialize, Serialize};
use std::fmt;

/// Supported presentation applications
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum PresentationApp {
    PowerPoint,
    Keynote,
    Impress,
}

impl fmt::Display for PresentationApp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PresentationApp::PowerPoint => write!(f, "PowerPoint"),
            PresentationApp::Keynote => write!(f, "Keynote"),
            PresentationApp::Impress => write!(f, "Impress"),
        }
    }
}

/// Current presentation status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresentationStatus {
    pub app: PresentationApp,
    pub app_running: bool,
    pub slideshow_active: bool,
    pub current_slide: Option<u32>,
    pub total_slides: Option<u32>,
    pub current_slide_title: Option<String>,
    pub blanked: bool,
}

/// Errors that can occur during presentation control
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PresentationError {
    /// The target application is not running
    AppNotRunning(String),
    /// No presentation is currently open
    NoPresentationOpen,
    /// No slideshow is currently active
    NoSlideshowActive,
    /// The file was not found
    FileNotFound(String),
    /// COM/AppleScript/UNO automation failed
    AutomationError(String),
    /// Platform not supported for this operation
    PlatformNotSupported(String),
}

impl fmt::Display for PresentationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PresentationError::AppNotRunning(app) => write!(f, "{} is not running", app),
            PresentationError::NoPresentationOpen => write!(f, "No presentation is open"),
            PresentationError::NoSlideshowActive => write!(f, "No slideshow is active"),
            PresentationError::FileNotFound(path) => write!(f, "File not found: {}", path),
            PresentationError::AutomationError(msg) => write!(f, "Automation error: {}", msg),
            PresentationError::PlatformNotSupported(msg) => {
                write!(f, "Platform not supported: {}", msg)
            }
        }
    }
}

impl std::error::Error for PresentationError {}
