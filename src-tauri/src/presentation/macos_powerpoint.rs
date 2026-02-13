//! macOS PowerPoint controller using AppleScript via `std::process::Command`.

use async_trait::async_trait;

use super::controller::PresentationController;
use super::types::{PresentationApp, PresentationError, PresentationStatus};

pub struct MacosPowerPointController;

impl MacosPowerPointController {
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
        let script = r#"tell application "System Events" to (name of every process) contains "Microsoft PowerPoint""#;
        Self::run_applescript(script)
            .map(|r| r == "true")
            .unwrap_or(false)
    }
}

#[async_trait]
impl PresentationController for MacosPowerPointController {
    async fn is_running(&self) -> bool {
        Self::is_app_running()
    }

    async fn open(&self, file_path: &str) -> Result<(), PresentationError> {
        if !std::path::Path::new(file_path).exists() {
            return Err(PresentationError::FileNotFound(file_path.to_string()));
        }

        let script = format!(
            r#"tell application "Microsoft PowerPoint" to open POSIX file "{}""#,
            file_path.replace('\\', "/").replace('"', "\\\"")
        );
        Self::run_applescript(&script)?;
        Ok(())
    }

    async fn start_slideshow(&self, from_slide: Option<u32>) -> Result<(), PresentationError> {
        // First start the slideshow
        let script = r#"tell application "Microsoft PowerPoint"
    run slide show (slide show settings of active presentation)
end tell"#;
        Self::run_applescript(script)?;

        // Then navigate to the target slide if specified
        if let Some(n) = from_slide {
            // Give it a moment to start
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            let goto_script = format!(
                r#"tell application "Microsoft PowerPoint"
    go to slide {n} of slide show view of slide show window 1
end tell"#
            );
            Self::run_applescript(&goto_script)?;
        }

        Ok(())
    }

    async fn stop_slideshow(&self) -> Result<(), PresentationError> {
        let script = r#"tell application "Microsoft PowerPoint"
    exit slide show of slide show view of slide show window 1
end tell"#;
        Self::run_applescript(script)?;
        Ok(())
    }

    async fn next(&self) -> Result<(), PresentationError> {
        let script = r#"tell application "Microsoft PowerPoint"
    go to next slide of slide show view of slide show window 1
end tell"#;
        Self::run_applescript(script)?;
        Ok(())
    }

    async fn previous(&self) -> Result<(), PresentationError> {
        let script = r#"tell application "Microsoft PowerPoint"
    go to previous slide of slide show view of slide show window 1
end tell"#;
        Self::run_applescript(script)?;
        Ok(())
    }

    async fn goto_slide(&self, slide_number: u32) -> Result<(), PresentationError> {
        let script = format!(
            r#"tell application "Microsoft PowerPoint"
    go to slide {} of slide show view of slide show window 1
end tell"#,
            slide_number
        );
        Self::run_applescript(&script)?;
        Ok(())
    }

    async fn blank_screen(&self) -> Result<(), PresentationError> {
        // PowerPoint for Mac: use keyboard shortcut 'b' via System Events
        let script = r#"tell application "System Events" to tell process "Microsoft PowerPoint" to keystroke "b""#;
        Self::run_applescript(script)?;
        Ok(())
    }

    async fn white_screen(&self) -> Result<(), PresentationError> {
        let script = r#"tell application "System Events" to tell process "Microsoft PowerPoint" to keystroke "w""#;
        Self::run_applescript(script)?;
        Ok(())
    }

    async fn unblank(&self) -> Result<(), PresentationError> {
        // Any key unblanks
        let script = r#"tell application "System Events" to tell process "Microsoft PowerPoint" to keystroke " ""#;
        Self::run_applescript(script)?;
        Ok(())
    }

    async fn close_all(&self) -> Result<(), PresentationError> {
        let script = r#"tell application "Microsoft PowerPoint" to close every presentation saving no"#;
        Self::run_applescript(script)?;
        Ok(())
    }

    async fn close_latest(&self) -> Result<(), PresentationError> {
        let script = r#"tell application "Microsoft PowerPoint" to close active presentation saving no"#;
        Self::run_applescript(script)?;
        Ok(())
    }

    async fn get_status(&self) -> Result<PresentationStatus, PresentationError> {
        if !Self::is_app_running() {
            return Ok(PresentationStatus {
                app: PresentationApp::PowerPoint,
                app_running: false,
                slideshow_active: false,
                current_slide: None,
                total_slides: None,
                current_slide_title: None,
                blanked: false,
            });
        }

        let script = r#"
tell application "Microsoft PowerPoint"
    if (count of presentations) = 0 then
        return "no_doc"
    end if
    set slideCount to count of slides of active presentation
    try
        set ssView to slide show view of slide show window 1
        set currentSlide to slide index of slide of ssView
        return "playing," & currentSlide & "," & slideCount
    on error
        return "stopped,0," & slideCount
    end try
end tell
"#;

        match Self::run_applescript(script) {
            Ok(result) => {
                if result == "no_doc" {
                    return Ok(PresentationStatus {
                        app: PresentationApp::PowerPoint,
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
                        app: PresentationApp::PowerPoint,
                        app_running: true,
                        slideshow_active,
                        current_slide: if slideshow_active {
                            current_slide
                        } else {
                            None
                        },
                        total_slides,
                        current_slide_title: None,
                        blanked: false,
                    })
                } else {
                    Ok(PresentationStatus {
                        app: PresentationApp::PowerPoint,
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
                app: PresentationApp::PowerPoint,
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
