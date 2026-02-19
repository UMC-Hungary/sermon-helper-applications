//! Windows PowerPoint controller using COM automation via `windows-rs`.
//!
//! Uses IDispatch late-binding to control PowerPoint's COM object model:
//! - PowerPoint.Application → .Presentations.Open(path)
//! - .ActivePresentation.SlideShowSettings.Run()
//! - .SlideShowWindows(1).View → .Next(), .Previous(), .GotoSlide(n)

use async_trait::async_trait;
use std::os::windows::process::CommandExt;
use std::sync::Mutex;
use windows::core::{Interface, BSTR, PCWSTR, VARIANT};
use windows::Win32::System::Com::{
    CLSIDFromProgID, CoCreateInstance, CoInitializeEx, CoUninitialize,
    CLSCTX_LOCAL_SERVER, COINIT_APARTMENTTHREADED, DISPATCH_METHOD, DISPATCH_PROPERTYGET,
    DISPATCH_PROPERTYPUT, DISPPARAMS, IDispatch,
};
use windows::Win32::System::Ole::{GetActiveObject, DISPID_PROPERTYPUT};

use super::controller::PresentationController;
use super::types::{PresentationApp, PresentationError, PresentationStatus};

// PowerPoint SlideShowView state constants
const PP_SLIDESHOW_RUNNING: i32 = 1;
#[allow(dead_code)]
const PP_SLIDESHOW_PAUSED: i32 = 2;
const PP_SLIDESHOW_BLACK_SCREEN: i32 = 3;
const PP_SLIDESHOW_WHITE_SCREEN: i32 = 4;
const PP_SLIDESHOW_DONE: i32 = 5;

pub struct WindowsPowerPointController {
    /// Cached COM application object - protected by mutex for thread safety
    app: Mutex<Option<IDispatch>>,
}

// SAFETY: COM access is serialized through the Mutex. All COM calls happen
// on the thread that acquires the lock. The IDispatch pointer is only accessed
// while the Mutex is held.
unsafe impl Send for WindowsPowerPointController {}
unsafe impl Sync for WindowsPowerPointController {}

impl WindowsPowerPointController {
    pub fn new() -> Self {
        Self {
            app: Mutex::new(None),
        }
    }

    /// Initialize COM and get or create PowerPoint.Application
    fn get_or_connect_app(&self) -> Result<IDispatch, PresentationError> {
        let mut app_guard = self.app.lock().map_err(|e| {
            PresentationError::AutomationError(format!("Failed to lock mutex: {}", e))
        })?;

        // Try to use cached dispatch
        if let Some(ref app) = *app_guard {
            // Verify it's still valid by trying a simple call
            if get_dispatch_id(app, "Visible").is_ok() {
                return Ok(app.clone());
            }
            // Invalid, clear cache
            *app_guard = None;
        }

        // Initialize COM
        unsafe {
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        }

        let prog_id: BSTR = "PowerPoint.Application".into();
        let clsid = unsafe {
            CLSIDFromProgID(&prog_id)
                .map_err(|e| PresentationError::AutomationError(format!("CLSIDFromProgID failed: {}", e)))?
        };

        // Try to connect to running instance first
        let app: IDispatch = unsafe {
            let mut punk: Option<windows::core::IUnknown> = None;
            if GetActiveObject(&clsid, None, &mut punk).is_ok() {
                if let Some(unknown) = punk {
                    unknown
                        .cast::<IDispatch>()
                        .map_err(|e| PresentationError::AutomationError(format!("Cast to IDispatch failed: {}", e)))?
                } else {
                    CoCreateInstance(&clsid, None, CLSCTX_LOCAL_SERVER)
                        .map_err(|e| PresentationError::AutomationError(format!("CoCreateInstance failed: {}", e)))?
                }
            } else {
                CoCreateInstance(&clsid, None, CLSCTX_LOCAL_SERVER)
                    .map_err(|e| PresentationError::AutomationError(format!("CoCreateInstance failed: {}", e)))?
            }
        };

        // Make it visible
        let _ = dispatch_put(&app, "Visible", &[VARIANT::from(true)]);

        *app_guard = Some(app.clone());
        Ok(app)
    }

    /// Get SlideShowWindows(1).View - the active slideshow view
    fn get_slideshow_view(&self, app: &IDispatch) -> Result<IDispatch, PresentationError> {
        let windows = dispatch_get_dispatch(app, "SlideShowWindows")?;
        let window_variant = dispatch_call_with_args(&windows, "Item", &mut [VARIANT::from(1i32)])?;
        let window = IDispatch::try_from(&window_variant).map_err(|_| {
            PresentationError::AutomationError("SlideShowWindows.Item(1) did not return IDispatch".to_string())
        })?;
        let view = dispatch_get_dispatch(&window, "View")?;
        Ok(view)
    }

    /// Get the active presentation
    fn get_active_presentation(&self, app: &IDispatch) -> Result<IDispatch, PresentationError> {
        dispatch_get_dispatch(app, "ActivePresentation").map_err(|_| PresentationError::NoPresentationOpen)
    }

    /// Get slideshow window count
    fn get_slideshow_window_count(&self, app: &IDispatch) -> Result<i32, PresentationError> {
        let windows = dispatch_get_dispatch(app, "SlideShowWindows")?;
        dispatch_get_i4(&windows, "Count")
    }
}

impl WindowsPowerPointController {
    fn is_running(&self) -> bool {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        }

        let prog_id: BSTR = "PowerPoint.Application".into();
        let clsid = match unsafe { CLSIDFromProgID(&prog_id) } {
            Ok(c) => c,
            Err(_) => return false,
        };

        unsafe {
            let mut punk: Option<windows::core::IUnknown> = None;
            GetActiveObject(&clsid, None, &mut punk).is_ok() && punk.is_some()
        }
    }
}

#[async_trait]
impl PresentationController for WindowsPowerPointController {
    async fn open(&self, file_path: &str) -> Result<(), PresentationError> {
        if !std::path::Path::new(file_path).exists() {
            return Err(PresentationError::FileNotFound(file_path.to_string()));
        }

        let app = self.get_or_connect_app()?;
        let presentations = dispatch_get_dispatch(&app, "Presentations")?;

        dispatch_call_with_args(
            &presentations,
            "Open",
            &mut [VARIANT::from(file_path)],
        )?;

        Ok(())
    }

    async fn start_slideshow(&self, from_slide: Option<u32>) -> Result<(), PresentationError> {
        let app = self.get_or_connect_app()?;
        let presentation = self.get_active_presentation(&app)?;
        let settings = dispatch_get_dispatch(&presentation, "SlideShowSettings")?;

        if let Some(slide_num) = from_slide {
            dispatch_put(&settings, "StartingSlide", &[VARIANT::from(slide_num as i32)])?;
        }

        dispatch_call(&settings, "Run")?;
        Ok(())
    }

    async fn stop_slideshow(&self) -> Result<(), PresentationError> {
        let app = self.get_or_connect_app()?;
        let count = self.get_slideshow_window_count(&app)?;
        if count == 0 {
            return Err(PresentationError::NoSlideshowActive);
        }

        let view = self.get_slideshow_view(&app)?;
        dispatch_call(&view, "Exit")?;
        Ok(())
    }

    async fn next(&self) -> Result<(), PresentationError> {
        let app = self.get_or_connect_app()?;
        let view = self.get_slideshow_view(&app)?;
        dispatch_call(&view, "Next")?;
        Ok(())
    }

    async fn previous(&self) -> Result<(), PresentationError> {
        let app = self.get_or_connect_app()?;
        let view = self.get_slideshow_view(&app)?;
        dispatch_call(&view, "Previous")?;
        Ok(())
    }

    async fn goto_slide(&self, slide_number: u32) -> Result<(), PresentationError> {
        let app = self.get_or_connect_app()?;
        let view = self.get_slideshow_view(&app)?;
        dispatch_call_with_args(&view, "GotoSlide", &mut [VARIANT::from(slide_number as i32)])?;
        Ok(())
    }

    async fn blank_screen(&self) -> Result<(), PresentationError> {
        let app = self.get_or_connect_app()?;
        let view = self.get_slideshow_view(&app)?;
        dispatch_put(&view, "State", &[VARIANT::from(PP_SLIDESHOW_BLACK_SCREEN)])?;
        Ok(())
    }

    async fn white_screen(&self) -> Result<(), PresentationError> {
        let app = self.get_or_connect_app()?;
        let view = self.get_slideshow_view(&app)?;
        dispatch_put(&view, "State", &[VARIANT::from(PP_SLIDESHOW_WHITE_SCREEN)])?;
        Ok(())
    }

    async fn unblank(&self) -> Result<(), PresentationError> {
        let app = self.get_or_connect_app()?;
        let view = self.get_slideshow_view(&app)?;
        dispatch_put(&view, "State", &[VARIANT::from(PP_SLIDESHOW_RUNNING)])?;
        Ok(())
    }

    async fn close_all(&self) -> Result<(), PresentationError> {
        let app = self.get_or_connect_app()?;

        // Close regular presentations (last to first to avoid index shifting)
        let presentations = dispatch_get_dispatch(&app, "Presentations")?;
        let count = dispatch_get_i4(&presentations, "Count")?;
        for i in (1..=count).rev() {
            let pres = dispatch_call_with_args(&presentations, "Item", &mut [VARIANT::from(i)])?;
            let pres_dispatch = IDispatch::try_from(&pres).map_err(|_| {
                PresentationError::AutomationError("Presentations.Item did not return IDispatch".to_string())
            })?;
            // Mark as saved to suppress "Save changes?" dialog
            let _ = dispatch_put(&pres_dispatch, "Saved", &[VARIANT::from(true)]);
            dispatch_call(&pres_dispatch, "Close")?;
        }

        // Close Protected View windows (files opened from untrusted locations)
        if let Ok(pv_windows) = dispatch_get_dispatch(&app, "ProtectedViewWindows") {
            if let Ok(pv_count) = dispatch_get_i4(&pv_windows, "Count") {
                for i in (1..=pv_count).rev() {
                    if let Ok(pv) = dispatch_call_with_args(&pv_windows, "Item", &mut [VARIANT::from(i)]) {
                        if let Ok(pv_dispatch) = IDispatch::try_from(&pv) {
                            let _ = dispatch_call(&pv_dispatch, "Close");
                        }
                    }
                }
            }
        }

        // Drop COM references before killing the process
        drop(presentations);
        drop(app);

        // Clear cached app connection
        let mut app_guard = self.app.lock().map_err(|e| {
            PresentationError::AutomationError(format!("Failed to lock mutex: {}", e))
        })?;
        *app_guard = None;
        drop(app_guard);

        // Kill PowerPoint process so the next open creates a truly fresh COM instance
        // (Quit() is unreliable — the process lingers and poisons subsequent CoCreateInstance calls)
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/IM", "POWERPNT.EXE"])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .output();

        // Wait for the process to fully exit
        for _ in 0..15 {
            std::thread::sleep(std::time::Duration::from_millis(200));
            let output = std::process::Command::new("tasklist")
                .args(["/FI", "IMAGENAME eq POWERPNT.EXE", "/NH"])
                .creation_flags(0x08000000)
                .output();
            if let Ok(out) = output {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if !stdout.contains("POWERPNT.EXE") {
                    break;
                }
            }
        }

        Ok(())
    }

    async fn close_latest(&self) -> Result<(), PresentationError> {
        let app = self.get_or_connect_app()?;

        // Close the last Protected View window first (most common on unlicensed PP)
        if let Ok(pv_windows) = dispatch_get_dispatch(&app, "ProtectedViewWindows") {
            if let Ok(pv_count) = dispatch_get_i4(&pv_windows, "Count") {
                if pv_count > 0 {
                    if let Ok(pv) = dispatch_call_with_args(&pv_windows, "Item", &mut [VARIANT::from(pv_count)]) {
                        if let Ok(pv_dispatch) = IDispatch::try_from(&pv) {
                            let _ = dispatch_call(&pv_dispatch, "Close");
                            return Ok(());
                        }
                    }
                }
            }
        }

        // Otherwise close the last regular presentation
        let presentations = dispatch_get_dispatch(&app, "Presentations")?;
        let count = dispatch_get_i4(&presentations, "Count")?;
        if count > 0 {
            let pres = dispatch_call_with_args(&presentations, "Item", &mut [VARIANT::from(count)])?;
            let pres_dispatch = IDispatch::try_from(&pres).map_err(|_| {
                PresentationError::AutomationError("Presentations.Item did not return IDispatch".to_string())
            })?;
            let _ = dispatch_put(&pres_dispatch, "Saved", &[VARIANT::from(true)]);
            dispatch_call(&pres_dispatch, "Close")?;
            return Ok(());
        }

        Err(PresentationError::NoPresentationOpen)
    }

    async fn get_status(&self) -> Result<PresentationStatus, PresentationError> {
        let running = self.is_running();
        if !running {
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

        let app = match self.get_or_connect_app() {
            Ok(a) => a,
            Err(_) => {
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
        };

        // Get total slides from active presentation
        let total_slides = self.get_active_presentation(&app).ok().and_then(|pres| {
            let slides = dispatch_get_dispatch(&pres, "Slides").ok()?;
            dispatch_get_i4(&slides, "Count").ok().map(|c| c as u32)
        });

        // Check slideshow status
        let window_count = self.get_slideshow_window_count(&app).unwrap_or(0);
        if window_count == 0 {
            return Ok(PresentationStatus {
                app: PresentationApp::PowerPoint,
                app_running: true,
                slideshow_active: false,
                current_slide: None,
                total_slides,
                current_slide_title: None,
                blanked: false,
            });
        }

        let view = match self.get_slideshow_view(&app) {
            Ok(v) => v,
            Err(_) => {
                return Ok(PresentationStatus {
                    app: PresentationApp::PowerPoint,
                    app_running: true,
                    slideshow_active: false,
                    current_slide: None,
                    total_slides,
                    current_slide_title: None,
                    blanked: false,
                });
            }
        };

        let state = dispatch_get_i4(&view, "State").unwrap_or(PP_SLIDESHOW_DONE);
        let slideshow_active = state != PP_SLIDESHOW_DONE;
        let blanked = state == PP_SLIDESHOW_BLACK_SCREEN || state == PP_SLIDESHOW_WHITE_SCREEN;

        let current_slide = dispatch_get_i4(&view, "CurrentShowPosition")
            .ok()
            .map(|s| s as u32);

        Ok(PresentationStatus {
            app: PresentationApp::PowerPoint,
            app_running: true,
            slideshow_active,
            current_slide,
            total_slides,
            current_slide_title: None,
            blanked,
        })
    }
}

impl Drop for WindowsPowerPointController {
    fn drop(&mut self) {
        let mut app_guard = self.app.lock().unwrap_or_else(|e| e.into_inner());
        *app_guard = None;
        unsafe {
            CoUninitialize();
        }
    }
}

// ============================================================================
// COM IDispatch Helper Functions
// ============================================================================

/// Get the DISPID for a named member
fn get_dispatch_id(disp: &IDispatch, name: &str) -> Result<i32, PresentationError> {
    let wide_name: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
    let name_ptr = PCWSTR(wide_name.as_ptr());
    let mut dispid = 0i32;

    unsafe {
        disp.GetIDsOfNames(
            &windows::core::GUID::zeroed(),
            &name_ptr,
            1,
            0x0400, // LOCALE_USER_DEFAULT
            &mut dispid,
        )
        .map_err(|e| {
            PresentationError::AutomationError(format!(
                "GetIDsOfNames('{}') failed: {}",
                name, e
            ))
        })?;
    }

    Ok(dispid)
}

/// Get a dispatch property that returns an IDispatch
fn dispatch_get_dispatch(disp: &IDispatch, name: &str) -> Result<IDispatch, PresentationError> {
    let dispid = get_dispatch_id(disp, name)?;
    let mut result = VARIANT::new();
    let params = DISPPARAMS::default();

    unsafe {
        disp.Invoke(
            dispid,
            &windows::core::GUID::zeroed(),
            0x0400,
            DISPATCH_PROPERTYGET | DISPATCH_METHOD,
            &params,
            Some(&mut result),
            None,
            None,
        )
        .map_err(|e| {
            PresentationError::AutomationError(format!("Get '{}' failed: {}", name, e))
        })?;
    }

    // Use TryFrom to extract IDispatch
    IDispatch::try_from(&result).map_err(|_| {
        PresentationError::AutomationError(format!(
            "Property '{}' did not return IDispatch",
            name
        ))
    })
}

/// Get an i4 (int32) property
fn dispatch_get_i4(disp: &IDispatch, name: &str) -> Result<i32, PresentationError> {
    let dispid = get_dispatch_id(disp, name)?;
    let mut result = VARIANT::new();
    let params = DISPPARAMS::default();

    unsafe {
        disp.Invoke(
            dispid,
            &windows::core::GUID::zeroed(),
            0x0400,
            DISPATCH_PROPERTYGET | DISPATCH_METHOD,
            &params,
            Some(&mut result),
            None,
            None,
        )
        .map_err(|e| {
            PresentationError::AutomationError(format!("Get '{}' failed: {}", name, e))
        })?;
    }

    i32::try_from(&result).map_err(|_| {
        PresentationError::AutomationError(format!(
            "Property '{}' did not return i32",
            name
        ))
    })
}

/// Call a dispatch method with no arguments
fn dispatch_call(disp: &IDispatch, name: &str) -> Result<(), PresentationError> {
    dispatch_call_with_args(disp, name, &mut [])?;
    Ok(())
}

/// Call a dispatch method with arguments
fn dispatch_call_with_args(
    disp: &IDispatch,
    name: &str,
    args: &mut [VARIANT],
) -> Result<VARIANT, PresentationError> {
    let dispid = get_dispatch_id(disp, name)?;
    let mut result = VARIANT::new();

    // COM expects arguments in reverse order
    args.reverse();

    let params = DISPPARAMS {
        rgvarg: if args.is_empty() {
            std::ptr::null_mut()
        } else {
            args.as_mut_ptr()
        },
        rgdispidNamedArgs: std::ptr::null_mut(),
        cArgs: args.len() as u32,
        cNamedArgs: 0,
    };

    unsafe {
        disp.Invoke(
            dispid,
            &windows::core::GUID::zeroed(),
            0x0400,
            DISPATCH_METHOD,
            &params,
            Some(&mut result),
            None,
            None,
        )
        .map_err(|e| {
            PresentationError::AutomationError(format!("Call '{}' failed: {}", name, e))
        })?;
    }

    Ok(result)
}

/// Put a dispatch property
fn dispatch_put(
    disp: &IDispatch,
    name: &str,
    args: &[VARIANT],
) -> Result<(), PresentationError> {
    let dispid = get_dispatch_id(disp, name)?;

    let mut args_vec: Vec<VARIANT> = args.iter().rev().cloned().collect();
    let mut named_arg = DISPID_PROPERTYPUT;

    let params = DISPPARAMS {
        rgvarg: args_vec.as_mut_ptr(),
        rgdispidNamedArgs: &mut named_arg,
        cArgs: args_vec.len() as u32,
        cNamedArgs: 1,
    };

    unsafe {
        disp.Invoke(
            dispid,
            &windows::core::GUID::zeroed(),
            0x0400,
            DISPATCH_PROPERTYPUT,
            &params,
            None,
            None,
            None,
        )
        .map_err(|e| {
            PresentationError::AutomationError(format!("Put '{}' failed: {}", name, e))
        })?;
    }

    Ok(())
}
