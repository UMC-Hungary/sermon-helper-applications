//! macOS Keynote controller using AppleScript via `std::process::Command`.

use async_trait::async_trait;

use super::controller::PresentationController;
use super::types::{PresentationApp, PresentationError, PresentationStatus};

pub struct MacosKeynoteController;

impl MacosKeynoteController {
    pub fn new() -> Self {
        Self
    }

    fn run_applescript(script: &str) -> Result<String, PresentationError> {
        let output = std::process::Command::new("osascript")
            .args(["-e", script])
            .output()
            .map_err(|e| {
                PresentationError::AutomationError(format!("Failed to run osascript: {}", e))
            })?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(PresentationError::AutomationError(format!(
                "AppleScript error: {}",
                stderr
            )))
        }
    }

    fn is_app_running() -> bool {
        let script = r#"tell application "System Events" to (name of every process) contains "Keynote""#;
        Self::run_applescript(script)
            .map(|r| r == "true")
            .unwrap_or(false)
    }
}

#[async_trait]
impl PresentationController for MacosKeynoteController {
    async fn is_running(&self) -> bool {
        Self::is_app_running()
    }

    async fn open(&self, file_path: &str) -> Result<(), PresentationError> {
        if !std::path::Path::new(file_path).exists() {
            return Err(PresentationError::FileNotFound(file_path.to_string()));
        }

        let script = format!(
            r#"tell application "Keynote" to open POSIX file "{}""#,
            file_path.replace('\\', "/").replace('"', "\\\"")
        );
        Self::run_applescript(&script)?;
        Ok(())
    }

    async fn start_slideshow(&self, from_slide: Option<u32>) -> Result<(), PresentationError> {
        let script = match from_slide {
            Some(n) => format!(
                r#"tell application "Keynote" to start front document from slide {} of front document"#,
                n
            ),
            None => r#"tell application "Keynote" to start front document"#.to_string(),
        };
        Self::run_applescript(&script)?;
        Ok(())
    }

    async fn stop_slideshow(&self) -> Result<(), PresentationError> {
        let script = r#"tell application "Keynote" to stop front document"#;
        Self::run_applescript(script)?;
        Ok(())
    }

    async fn next(&self) -> Result<(), PresentationError> {
        Self::run_applescript(r#"tell application "Keynote" to show next"#)?;
        Ok(())
    }

    async fn previous(&self) -> Result<(), PresentationError> {
        Self::run_applescript(r#"tell application "Keynote" to show previous"#)?;
        Ok(())
    }

    async fn goto_slide(&self, slide_number: u32) -> Result<(), PresentationError> {
        // Keynote doesn't have a direct goto in slideshow — restart from that slide
        let script = format!(
            r#"tell application "Keynote" to start front document from slide {} of front document"#,
            slide_number
        );
        Self::run_applescript(&script)?;
        Ok(())
    }

    async fn blank_screen(&self) -> Result<(), PresentationError> {
        // Keynote uses 'b' key to blank screen — send via System Events
        let script = r#"tell application "System Events" to tell process "Keynote" to keystroke "b""#;
        Self::run_applescript(script)?;
        Ok(())
    }

    async fn white_screen(&self) -> Result<(), PresentationError> {
        // Keynote uses 'w' key for white screen
        let script = r#"tell application "System Events" to tell process "Keynote" to keystroke "w""#;
        Self::run_applescript(script)?;
        Ok(())
    }

    async fn unblank(&self) -> Result<(), PresentationError> {
        // Any key unblanks in Keynote — send space or another key
        let script =
            r#"tell application "System Events" to tell process "Keynote" to keystroke " ""#;
        Self::run_applescript(script)?;
        Ok(())
    }

    async fn get_status(&self) -> Result<PresentationStatus, PresentationError> {
        if !Self::is_app_running() {
            return Ok(PresentationStatus {
                app: PresentationApp::Keynote,
                app_running: false,
                slideshow_active: false,
                current_slide: None,
                total_slides: None,
                current_slide_title: None,
                blanked: false,
            });
        }

        let script = r#"
tell application "Keynote"
    if (count of documents) = 0 then
        return "no_doc"
    end if
    set slideCount to count of slides of front document
    set isPlaying to playing
    if isPlaying then
        set currentSlide to slide number of current slide of front document
        return "playing," & currentSlide & "," & slideCount
    else
        return "stopped,0," & slideCount
    end if
end tell
"#;

        match Self::run_applescript(script) {
            Ok(result) => {
                if result == "no_doc" {
                    return Ok(PresentationStatus {
                        app: PresentationApp::Keynote,
                        app_running: true,
                        slideshow_active: false,
                        current_slide: None,
                        total_slides: None,
                        current_slide_title: None,
                        blanked: false,
                    });
                }

                let parts: Vec<&str> = result.split(',').collect();
                if parts.len() >= 3 {
                    let slideshow_active = parts[0] == "playing";
                    let current_slide = parts[1].parse::<u32>().ok();
                    let total_slides = parts[2].parse::<u32>().ok();

                    Ok(PresentationStatus {
                        app: PresentationApp::Keynote,
                        app_running: true,
                        slideshow_active,
                        current_slide: if slideshow_active {
                            current_slide
                        } else {
                            None
                        },
                        total_slides,
                        current_slide_title: None,
                        blanked: false, // Can't easily detect blank state via AppleScript
                    })
                } else {
                    Ok(PresentationStatus {
                        app: PresentationApp::Keynote,
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
                app: PresentationApp::Keynote,
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
