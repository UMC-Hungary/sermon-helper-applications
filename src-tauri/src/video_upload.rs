// Video upload module for sermon-helper
// Handles file scanning, chunked uploads, and progress tracking

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

/// Information about a recording file
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecordingFile {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub duration: f64,     // seconds
    pub created_at: u64,   // unix timestamp ms
    pub modified_at: u64,  // unix timestamp ms
}

/// Information about a video file (for upload)
#[derive(Debug, Serialize, Deserialize)]
pub struct VideoFileInfo {
    pub path: String,
    pub size: u64,
    pub exists: bool,
}

/// Result of uploading a chunk
#[derive(Debug, Serialize, Deserialize)]
pub struct UploadChunkResult {
    pub bytes_uploaded: u64,
    pub total_bytes: u64,
    pub completed: bool,
    pub video_id: Option<String>,
}

/// Scan a directory for video files within a time window
/// Uses ffprobe (if available) to get accurate duration, falls back to file size estimation
#[tauri::command]
pub async fn scan_recording_directory(
    directory: String,
    session_start: u64, // unix timestamp ms
    session_end: u64,   // unix timestamp ms
) -> Result<Vec<RecordingFile>, String> {
    let dir = Path::new(&directory);
    if !dir.exists() {
        return Err(format!("Recording directory does not exist: {}", directory));
    }

    if !dir.is_dir() {
        return Err(format!("Path is not a directory: {}", directory));
    }

    let video_extensions = ["mp4", "mkv", "flv", "mov", "avi", "webm", "ts"];
    let mut recordings = Vec::new();

    let entries = std::fs::read_dir(dir).map_err(|e| format!("Failed to read directory: {}", e))?;

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                log::warn!("Failed to read directory entry: {}", e);
                continue;
            }
        };

        let path = entry.path();

        // Skip directories
        if path.is_dir() {
            continue;
        }

        // Check if video file by extension
        let ext = match path.extension() {
            Some(e) => e.to_str().unwrap_or("").to_lowercase(),
            None => continue,
        };

        if !video_extensions.contains(&ext.as_str()) {
            continue;
        }

        // Get file metadata
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(e) => {
                log::warn!("Failed to get metadata for {:?}: {}", path, e);
                continue;
            }
        };

        // Get modification time
        let modified = match metadata.modified() {
            Ok(t) => t
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            Err(_) => continue,
        };

        // Filter by time window
        if modified < session_start || modified > session_end {
            log::debug!(
                "Skipping {:?}: modified {} not in range [{}, {}]",
                path,
                modified,
                session_start,
                session_end
            );
            continue;
        }

        // Get duration using ffprobe or estimation
        let duration = get_video_duration(&path).unwrap_or_else(|| {
            // Fallback: estimate from file size (~5MB per minute for 1080p)
            let size = metadata.len();
            (size as f64) / (5.0 * 1024.0 * 1024.0) * 60.0
        });

        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        recordings.push(RecordingFile {
            path: path.to_string_lossy().to_string(),
            name: file_name,
            size: metadata.len(),
            duration,
            created_at: modified, // Use modified as proxy for created
            modified_at: modified,
        });
    }

    // Sort by modified time (newest first)
    recordings.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));

    log::info!(
        "Found {} video files in {} within time window",
        recordings.len(),
        directory
    );

    Ok(recordings)
}

/// Get video duration using ffprobe if available
fn get_video_duration(path: &Path) -> Option<f64> {
    let path_str = path.to_str()?;

    // Try ffprobe first
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            path_str,
        ])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let duration_str = String::from_utf8_lossy(&output.stdout);
            let duration: f64 = duration_str.trim().parse().ok()?;
            log::debug!("ffprobe duration for {:?}: {}s", path, duration);
            Some(duration)
        }
        Ok(_) => {
            log::debug!("ffprobe failed for {:?}, using estimation", path);
            None
        }
        Err(e) => {
            log::debug!("ffprobe not available: {}, using estimation", e);
            None
        }
    }
}

/// Get video file information
#[tauri::command]
pub async fn get_video_file_info(path: String) -> Result<VideoFileInfo, String> {
    let file_path = Path::new(&path);

    if !file_path.exists() {
        return Ok(VideoFileInfo {
            path,
            size: 0,
            exists: false,
        });
    }

    let metadata = std::fs::metadata(file_path)
        .map_err(|e| format!("Failed to get file metadata: {}", e))?;

    Ok(VideoFileInfo {
        path,
        size: metadata.len(),
        exists: true,
    })
}

/// Initialize a YouTube resumable upload session
/// Returns the upload URI for subsequent chunk uploads
#[tauri::command]
pub async fn init_youtube_upload(
    access_token: String,
    file_path: String,
    title: String,
    description: String,
    privacy_status: String,
) -> Result<String, String> {
    let file_info = get_video_file_info(file_path.clone()).await?;
    if !file_info.exists {
        return Err(format!("Video file does not exist: {}", file_path));
    }

    // Determine content type from extension
    let content_type = get_content_type(&file_path);

    let client = reqwest::Client::new();

    // Create the metadata
    let metadata = serde_json::json!({
        "snippet": {
            "title": title,
            "description": description,
            "categoryId": "22" // People & Blogs (common for church content)
        },
        "status": {
            "privacyStatus": privacy_status,
            "selfDeclaredMadeForKids": false
        }
    });

    // Initialize resumable upload
    let response = client
        .post("https://www.googleapis.com/upload/youtube/v3/videos?uploadType=resumable&part=snippet,status")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "application/json; charset=UTF-8")
        .header("X-Upload-Content-Length", file_info.size.to_string())
        .header("X-Upload-Content-Type", content_type)
        .json(&metadata)
        .send()
        .await
        .map_err(|e| format!("Failed to initialize upload: {}", e))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("YouTube API error: {}", error_text));
    }

    // Get the upload URI from the Location header
    let upload_uri = response
        .headers()
        .get("location")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
        .ok_or_else(|| "No upload URI in response".to_string())?;

    log::info!("Initialized YouTube upload, URI: {}", upload_uri);

    Ok(upload_uri)
}

/// Upload a chunk of the video file
#[tauri::command]
pub async fn upload_video_chunk(
    app: tauri::AppHandle,
    upload_uri: String,
    file_path: String,
    start_byte: u64,
    chunk_size: u64,
) -> Result<UploadChunkResult, String> {
    use std::io::{Read, Seek, SeekFrom};
    use tauri::Emitter;

    let file_info = get_video_file_info(file_path.clone()).await?;
    let total_bytes = file_info.size;

    // Calculate actual chunk size (may be smaller for last chunk)
    let actual_chunk_size = std::cmp::min(chunk_size, total_bytes - start_byte);
    let end_byte = start_byte + actual_chunk_size - 1;

    // Read the chunk from file
    let mut file =
        std::fs::File::open(&file_path).map_err(|e| format!("Failed to open file: {}", e))?;

    file.seek(SeekFrom::Start(start_byte))
        .map_err(|e| format!("Failed to seek: {}", e))?;

    let mut buffer = vec![0u8; actual_chunk_size as usize];
    file.read_exact(&mut buffer)
        .map_err(|e| format!("Failed to read chunk: {}", e))?;

    // Determine content type
    let content_type = get_content_type(&file_path);

    // Upload the chunk
    let client = reqwest::Client::new();
    let content_range = format!("bytes {}-{}/{}", start_byte, end_byte, total_bytes);

    log::debug!("Uploading chunk: {}", content_range);

    let response = client
        .put(&upload_uri)
        .header("Content-Length", actual_chunk_size.to_string())
        .header("Content-Type", content_type)
        .header("Content-Range", content_range)
        .body(buffer)
        .send()
        .await
        .map_err(|e| format!("Failed to upload chunk: {}", e))?;

    let status = response.status();
    let bytes_uploaded = end_byte + 1;

    // Emit progress event
    let _ = app.emit(
        "upload-progress",
        serde_json::json!({
            "bytesUploaded": bytes_uploaded,
            "totalBytes": total_bytes,
            "percentage": (bytes_uploaded as f64 / total_bytes as f64) * 100.0
        }),
    );

    if status.as_u16() == 308 {
        // Resume incomplete - more chunks needed
        Ok(UploadChunkResult {
            bytes_uploaded,
            total_bytes,
            completed: false,
            video_id: None,
        })
    } else if status.is_success() {
        // Upload complete - parse response for video ID
        let response_text = response.text().await.unwrap_or_default();
        let video_id = extract_video_id(&response_text);

        log::info!("Upload complete, video ID: {:?}", video_id);

        Ok(UploadChunkResult {
            bytes_uploaded: total_bytes,
            total_bytes,
            completed: true,
            video_id,
        })
    } else {
        let error_text = response.text().await.unwrap_or_default();
        Err(format!("Upload failed with status {}: {}", status, error_text))
    }
}

/// Query upload status (for resuming interrupted uploads)
#[tauri::command]
pub async fn get_upload_status(upload_uri: String, total_size: u64) -> Result<u64, String> {
    let client = reqwest::Client::new();

    let response = client
        .put(&upload_uri)
        .header("Content-Length", "0")
        .header("Content-Range", format!("bytes */{}", total_size))
        .send()
        .await
        .map_err(|e| format!("Failed to query upload status: {}", e))?;

    if response.status().as_u16() == 308 {
        // Get the Range header to find out how many bytes were uploaded
        if let Some(range) = response.headers().get("range") {
            if let Ok(range_str) = range.to_str() {
                // Format: "bytes=0-12345"
                if let Some(end) = range_str.strip_prefix("bytes=0-") {
                    if let Ok(end_byte) = end.parse::<u64>() {
                        return Ok(end_byte + 1);
                    }
                }
            }
        }
        Ok(0) // No bytes uploaded yet
    } else if response.status().is_success() {
        // Upload already complete
        Ok(total_size)
    } else {
        Err(format!(
            "Failed to query upload status: {}",
            response.status()
        ))
    }
}

/// Cancel an in-progress upload
#[tauri::command]
pub async fn cancel_upload(upload_uri: String) -> Result<(), String> {
    let client = reqwest::Client::new();

    let response = client
        .delete(&upload_uri)
        .header("Content-Length", "0")
        .send()
        .await
        .map_err(|e| format!("Failed to cancel upload: {}", e))?;

    if response.status().is_success() || response.status().as_u16() == 499 {
        log::info!("Upload cancelled successfully");
        Ok(())
    } else {
        log::warn!("Cancel upload returned status: {}", response.status());
        Ok(()) // Consider it cancelled anyway
    }
}

/// Get content type from file extension
fn get_content_type(file_path: &str) -> &'static str {
    let path = Path::new(file_path);
    match path.extension().and_then(|e| e.to_str()) {
        Some("mp4") => "video/mp4",
        Some("mkv") => "video/x-matroska",
        Some("webm") => "video/webm",
        Some("mov") => "video/quicktime",
        Some("avi") => "video/x-msvideo",
        Some("flv") => "video/x-flv",
        Some("ts") => "video/mp2t",
        _ => "video/mp4", // Default
    }
}

/// Extract video ID from YouTube API response
fn extract_video_id(response: &str) -> Option<String> {
    // Try to parse as JSON and extract id field
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(response) {
        if let Some(id) = json.get("id").and_then(|v| v.as_str()) {
            return Some(id.to_string());
        }
    }
    None
}
