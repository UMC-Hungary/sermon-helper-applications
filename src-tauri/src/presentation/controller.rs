use async_trait::async_trait;

use super::types::{PresentationError, PresentationStatus};

/// Trait for controlling presentation applications.
///
/// Each platform implementation (Windows PowerPoint, macOS Keynote, etc.)
/// implements this trait to provide a unified control interface.
#[async_trait]
pub trait PresentationController: Send + Sync {

    /// Open a presentation file
    async fn open(&self, file_path: &str) -> Result<(), PresentationError>;

    /// Start slideshow from beginning or from a specific slide
    async fn start_slideshow(&self, from_slide: Option<u32>) -> Result<(), PresentationError>;

    /// Stop/end the slideshow
    async fn stop_slideshow(&self) -> Result<(), PresentationError>;

    /// Navigate: next slide/build
    async fn next(&self) -> Result<(), PresentationError>;

    /// Navigate: previous slide/build
    async fn previous(&self) -> Result<(), PresentationError>;

    /// Navigate: go to specific slide number
    async fn goto_slide(&self, slide_number: u32) -> Result<(), PresentationError>;

    /// Blank screen (black)
    async fn blank_screen(&self) -> Result<(), PresentationError>;

    /// White screen
    async fn white_screen(&self) -> Result<(), PresentationError>;

    /// Restore from blank/white
    async fn unblank(&self) -> Result<(), PresentationError>;

    /// Close all open presentations/documents
    async fn close_all(&self) -> Result<(), PresentationError>;

    /// Close the most recently opened presentation/document
    async fn close_latest(&self) -> Result<(), PresentationError>;

    /// Get current status (slide number, total slides, slideshow running, etc.)
    async fn get_status(&self) -> Result<PresentationStatus, PresentationError>;
}
