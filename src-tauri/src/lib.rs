mod bible;
mod local_server;
mod video_upload;

use bible::{fetch_bible_v2, fetch_bible_suggestions, fetch_bible_legacy};
use local_server::{start_oauth_callback_server, start_oauth_flow_with_callback, get_oauth_redirect_uri};
use video_upload::{
    scan_recording_directory, get_video_file_info, init_youtube_upload,
    upload_video_chunk, get_upload_status, cancel_upload
};
use tauri_plugin_deep_link::DeepLinkExt;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_websocket::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .setup(|app| {
            // Register the deep link scheme on Linux/Windows (macOS uses Info.plist)
            // In dev mode, this may fail if the app isn't installed - that's OK
            #[cfg(any(target_os = "linux", all(debug_assertions, windows)))]
            {
                if let Err(e) = app.deep_link().register("sermon-helper") {
                    eprintln!("Warning: Failed to register deep link scheme: {}. Deep links may not work in dev mode.", e);
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            fetch_bible_v2,
            fetch_bible_suggestions,
            fetch_bible_legacy,
            start_oauth_callback_server,
            start_oauth_flow_with_callback,
            get_oauth_redirect_uri,
            // Video upload commands
            scan_recording_directory,
            get_video_file_info,
            init_youtube_upload,
            upload_video_chunk,
            get_upload_status,
            cancel_upload
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
