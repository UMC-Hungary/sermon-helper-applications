# Event Lifecycle & Post-Event Automation Feature Plan

## Overview

Implement automatic detection of event completion and post-event automation including:
- Recording upload to YouTube with livestream metadata
- Publishing the uploaded recording
- Unpublishing/completing the original live broadcast
- Connection resilience with pause/resume capability

---

## Current State Analysis

### Existing Infrastructure
- **OBS WebSocket** (`src/lib/utils/obs-websocket.ts`): Stream/record state tracking via events
- **YouTube API** (`src/lib/utils/youtube-api.ts`): Broadcast management, NO video upload
- **Streaming Store** (`src/lib/stores/streaming-store.ts`): Derived stores for UI
- **Refresh Store** (`src/lib/stores/refresh-store.ts`): 5-minute sync interval
- **Event Type** (`src/lib/types/event.ts`): Has unused `youtubeUploadedId` and `videoUploadState`

### Event Type Extension

Add new fields to `ServiceEvent` in `src/lib/types/event.ts`:

```typescript
// Existing YouTube live broadcast fields
youtubeScheduledId?: string;
youtubePrivacyStatus: YouTubePrivacyStatus;
youtubeLifeCycleStatus?: YouTubeLifeCycleStatus;

// New fields for multi-platform upload tracking
uploads?: {
  youtube?: {
    videoId?: string;           // Uploaded video ID
    privacyStatus: YouTubePrivacyStatus;  // Privacy for uploaded recording
    uploadState: VideoUploadState;
    uploadProgress?: number;    // 0-100
    uploadError?: string;
  };
  facebook?: {
    videoId?: string;
    privacyStatus: 'EVERYONE' | 'ALL_FRIENDS' | 'SELF';
    uploadState: VideoUploadState;
    uploadProgress?: number;
    uploadError?: string;
  };
};

// Backward compatibility alias (maps to uploads.youtube)
youtubeUploadedId?: string;     // Deprecated - use uploads.youtube.videoId
videoUploadState?: VideoUploadState;  // Deprecated - use uploads.youtube.uploadState
```

This structure:
- Supports multiple platform uploads per event
- Tracks upload state/progress independently per platform
- Allows different privacy settings per platform
- Maintains backward compatibility with existing `youtubeUploadedId` field

### Gaps to Fill
1. No way to get OBS recording file path
2. No YouTube video upload implementation
3. No event lifecycle state machine
4. No post-event automation workflow

---

## Architecture Design

### Multi-Platform Upload Configuration

To support YouTube now and Facebook (and potentially other platforms) later, we'll use a plugin-based architecture with a common interface.

**New File:** `src/lib/types/upload-config.ts`

```typescript
// Supported upload platforms
export type UploadPlatform = 'youtube' | 'facebook' | 'custom';

// Base configuration for all platforms
export interface UploadPlatformConfig {
  platform: UploadPlatform;
  enabled: boolean;
  autoUpload: boolean;  // Upload automatically when event ends
}

// YouTube-specific config
export interface YouTubeUploadConfig extends UploadPlatformConfig {
  platform: 'youtube';
  useEventPrivacy: boolean;  // Use event's youtubeUploadPrivacyStatus
  defaultPrivacy: 'public' | 'unlisted' | 'private';
  publishAfterUpload: boolean;  // Auto-publish when processing complete
}

// Facebook-specific config (for future)
export interface FacebookUploadConfig extends UploadPlatformConfig {
  platform: 'facebook';
  pageId?: string;  // Facebook Page to upload to
  defaultPrivacy: 'EVERYONE' | 'ALL_FRIENDS' | 'SELF';
  crosspostToTimeline: boolean;
}

// Custom webhook config (for future extensibility)
export interface CustomUploadConfig extends UploadPlatformConfig {
  platform: 'custom';
  webhookUrl: string;
  headers?: Record<string, string>;
}

// Union type for all platform configs
export type PlatformUploadConfig =
  | YouTubeUploadConfig
  | FacebookUploadConfig
  | CustomUploadConfig;

// Main upload settings
export interface UploadSettings {
  platforms: PlatformUploadConfig[];
  minDurationMinutes: number;      // Minimum duration to trigger auto-upload (default: 45)
  shortVideoThresholdMinutes: number;  // Videos shorter than this are "short" (default: 10)
  retryAttempts: number;           // Number of retry attempts (default: 3)
  chunkSizeMB: number;             // Upload chunk size (default: 10)
}

// Default settings
export const DEFAULT_UPLOAD_SETTINGS: UploadSettings = {
  platforms: [
    {
      platform: 'youtube',
      enabled: true,
      autoUpload: true,
      useEventPrivacy: true,
      defaultPrivacy: 'public',
      publishAfterUpload: true,
    }
  ],
  minDurationMinutes: 45,
  shortVideoThresholdMinutes: 10,  // Videos <= 10min are considered "short"
  retryAttempts: 3,
  chunkSizeMB: 10,
};
```

**New File:** `src/lib/stores/upload-settings-store.ts`

```typescript
// Persisted store for upload configuration
// Stored via Tauri store with fallback to localStorage

export const uploadSettingsStore = {
  getSettings(): Promise<UploadSettings>;
  updateSettings(settings: Partial<UploadSettings>): Promise<void>;
  getPlatformConfig<T extends PlatformUploadConfig>(platform: UploadPlatform): T | undefined;
  setPlatformConfig(config: PlatformUploadConfig): Promise<void>;
  enablePlatform(platform: UploadPlatform, enabled: boolean): Promise<void>;
};
```

### Upload Service Interface (Plugin Pattern)

**New File:** `src/lib/services/upload/upload-service.interface.ts`

```typescript
// Common interface for all upload platforms
export interface IUploadService {
  readonly platform: UploadPlatform;

  // Check if platform is configured and ready
  isConfigured(): Promise<boolean>;

  // Initialize upload session
  initializeUpload(
    filePath: string,
    metadata: UploadMetadata
  ): Promise<UploadSession>;

  // Upload file with progress
  upload(
    session: UploadSession,
    onProgress: (progress: UploadProgress) => void
  ): Promise<UploadResult>;

  // Resume interrupted upload
  resume(session: UploadSession): Promise<UploadSession>;

  // Cancel upload
  cancel(session: UploadSession): Promise<void>;

  // Post-upload actions (publish, etc.)
  finalize(result: UploadResult): Promise<void>;
}

// Common metadata for all platforms
export interface UploadMetadata {
  title: string;
  description: string;
  privacy: string;  // Platform-specific privacy value
  tags?: string[];
  thumbnailPath?: string;
}

// Upload session (persisted for resume capability)
export interface UploadSession {
  id: string;
  platform: UploadPlatform;
  filePath: string;
  fileSize: number;
  metadata: UploadMetadata;
  uploadUri: string;  // Platform-specific upload URI
  bytesUploaded: number;
  startedAt: number;
  status: 'pending' | 'uploading' | 'paused' | 'completed' | 'failed';
}

// Upload result
export interface UploadResult {
  platform: UploadPlatform;
  videoId: string;
  videoUrl: string;
  processingStatus: 'processing' | 'ready' | 'failed';
}
```

### Platform Implementations

```
src/lib/services/upload/
├── upload-service.interface.ts   # Common interface
├── upload-manager.ts             # Orchestrates multiple platforms
├── youtube-upload.service.ts     # YouTube implementation
└── facebook-upload.service.ts    # Facebook implementation (future)
```

**New File:** `src/lib/services/upload/upload-manager.ts`

```typescript
// Manages uploads across all configured platforms
export class UploadManager {
  private services: Map<UploadPlatform, IUploadService> = new Map();

  // Register platform services
  registerService(service: IUploadService): void;

  // Upload to all enabled platforms
  async uploadToAllPlatforms(
    filePath: string,
    metadata: UploadMetadata,
    onProgress: (platform: UploadPlatform, progress: UploadProgress) => void
  ): Promise<Map<UploadPlatform, UploadResult>>;

  // Upload to specific platform
  async uploadToPlatform(
    platform: UploadPlatform,
    filePath: string,
    metadata: UploadMetadata,
    onProgress: (progress: UploadProgress) => void
  ): Promise<UploadResult>;

  // Get all pending/paused uploads (for resume on app restart)
  async getPendingUploads(): Promise<UploadSession[]>;

  // Resume all pending uploads
  async resumeAllPending(): Promise<void>;
}

export const uploadManager = new UploadManager();
```

---

### New State Machine: Event Lifecycle

```
┌─────────────┐
│   IDLE      │ ← Initial state, nothing active
└──────┬──────┘
       │ OBS connected + event selected
       ▼
┌─────────────┐
│  PREPARING  │ ← OBS connected, waiting for stream/record
└──────┬──────┘
       │ Stream OR record started
       ▼
┌─────────────┐
│   ACTIVE    │ ← Streaming and/or recording in progress
└──────┬──────┘
       │ All outputs stopped + duration > 45min
       ▼
┌─────────────┐
│  FINALIZING │ ← Post-event automation in progress
└──────┬──────┘
       │ Upload complete + broadcast ended
       ▼
┌─────────────┐
│  COMPLETED  │ ← Event fully processed
└─────────────┘

Special States:
┌─────────────┐
│   PAUSED    │ ← Connection lost during ACTIVE/FINALIZING
└─────────────┘
```

### Event Session Tracking

Track the following for each active event session:

```typescript
interface EventSession {
  eventId: string;
  state: EventSessionState;

  // Timestamps
  sessionStartedAt: number;
  streamStartedAt?: number;
  streamEndedAt?: number;
  recordStartedAt?: number;
  recordEndedAt?: number;

  // Peak states (were these ever active during session?)
  wasOBSConnected: boolean;
  wasStreaming: boolean;
  wasRecording: boolean;
  wasYouTubeLive: boolean;

  // Recording file
  recordingFilePath?: string;

  // Upload progress
  uploadProgress?: number;
  uploadError?: string;

  // Connection resilience
  pausedAt?: number;
  pauseReason?: string;
}
```

---

## Implementation Plan

### Phase 1: OBS Recording Path Detection & File Selection

**File:** `src/lib/utils/obs-websocket.ts`

Add method to get the recording directory from OBS:

```typescript
async getRecordDirectory(): Promise<string> {
  // Call GetRecordDirectory to get output path
}
```

**OBS WebSocket calls needed:**
- `GetRecordDirectory` - Get the recording output directory
- `RecordStateChanged` event has `outputPath` in some OBS versions

---

**New File:** `src/lib/utils/recording-file-selector.ts`

Handles automatic selection of the correct recording file when multiple exist:

```typescript
import { invoke } from '@tauri-apps/api/core';

interface RecordingFile {
  path: string;
  name: string;
  size: number;           // bytes
  duration: number;       // seconds (from file metadata)
  createdAt: number;      // timestamp
  modifiedAt: number;     // timestamp
}

interface RecordingSelectionResult {
  autoSelected: boolean;
  selectedFile: RecordingFile | null;
  candidates: RecordingFile[];  // All files that could be the recording
  reason: 'single_long' | 'multiple_long' | 'no_long' | 'user_selected';
}

// Thresholds (configurable in UploadSettings)
const DEFAULT_SHORT_VIDEO_THRESHOLD = 10 * 60;  // 10 minutes in seconds
const DEFAULT_MIN_UPLOAD_DURATION = 45 * 60;    // 45 minutes in seconds

class RecordingFileSelector {
  /**
   * Find and select the appropriate recording file from OBS output directory
   *
   * Logic:
   * 1. Scan recording directory for video files created during session
   * 2. Filter by session time window (sessionStart - 5min to sessionEnd + 5min)
   * 3. Categorize as "long" (>10min) or "short" (<=10min)
   * 4. Auto-select if exactly 1 long video exists
   * 5. Prompt user if multiple long videos exist
   * 6. Return null if no suitable recordings found
   */
  async selectRecording(
    recordingDir: string,
    sessionStartTime: number,
    sessionEndTime: number,
    options?: {
      shortVideoThreshold?: number;  // seconds, default 10min
      minUploadDuration?: number;    // seconds, default 45min
    }
  ): Promise<RecordingSelectionResult> {
    const shortThreshold = options?.shortVideoThreshold ?? DEFAULT_SHORT_VIDEO_THRESHOLD;
    const minDuration = options?.minUploadDuration ?? DEFAULT_MIN_UPLOAD_DURATION;

    // 1. Get all video files from recording directory (via Rust)
    const allFiles = await invoke<RecordingFile[]>('scan_recording_directory', {
      directory: recordingDir,
      sessionStart: sessionStartTime - (5 * 60 * 1000),  // 5min buffer before
      sessionEnd: sessionEndTime + (5 * 60 * 1000),      // 5min buffer after
    });

    // 2. Categorize by duration
    const longVideos = allFiles.filter(f => f.duration >= shortThreshold);
    const shortVideos = allFiles.filter(f => f.duration < shortThreshold);

    // 3. Apply selection logic
    if (longVideos.length === 0) {
      // No long videos found
      return {
        autoSelected: false,
        selectedFile: null,
        candidates: allFiles,
        reason: 'no_long',
      };
    }

    if (longVideos.length === 1) {
      // Exactly 1 long video - auto-select it
      const selected = longVideos[0];

      // But only if it meets minimum upload duration
      if (selected.duration >= minDuration) {
        return {
          autoSelected: true,
          selectedFile: selected,
          candidates: longVideos,
          reason: 'single_long',
        };
      } else {
        // Long enough to not be "short", but not long enough to upload
        return {
          autoSelected: false,
          selectedFile: null,
          candidates: longVideos,
          reason: 'no_long',  // Treat as no valid candidates
        };
      }
    }

    // Multiple long videos - need user selection
    return {
      autoSelected: false,
      selectedFile: null,
      candidates: longVideos,
      reason: 'multiple_long',
    };
  }

  /**
   * Let user manually select from candidates
   */
  async promptUserSelection(candidates: RecordingFile[]): Promise<RecordingFile | null> {
    // This will be handled by UI component
    // Returns the user's selection or null if cancelled
  }
}

export const recordingFileSelector = new RecordingFileSelector();
```

---

**Rust Backend:** `src-tauri/src/video_upload.rs`

Add command to scan recording directory:

```rust
#[derive(serde::Serialize)]
pub struct RecordingFile {
    path: String,
    name: String,
    size: u64,
    duration: f64,      // seconds
    created_at: u64,    // unix timestamp ms
    modified_at: u64,   // unix timestamp ms
}

/// Scan a directory for video files within a time window
/// Uses ffprobe (if available) or file metadata to get duration
#[tauri::command]
pub async fn scan_recording_directory(
    directory: String,
    session_start: u64,  // unix timestamp ms
    session_end: u64,    // unix timestamp ms
) -> Result<Vec<RecordingFile>, String> {
    let dir = std::path::Path::new(&directory);
    if !dir.exists() {
        return Err("Recording directory does not exist".into());
    }

    let mut recordings = Vec::new();
    let video_extensions = ["mp4", "mkv", "flv", "mov", "avi", "webm"];

    for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        // Check if video file
        if let Some(ext) = path.extension() {
            if !video_extensions.contains(&ext.to_str().unwrap_or("").to_lowercase().as_str()) {
                continue;
            }
        } else {
            continue;
        }

        let metadata = entry.metadata().map_err(|e| e.to_string())?;
        let modified = metadata.modified()
            .map_err(|e| e.to_string())?
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        // Filter by time window
        if modified < session_start || modified > session_end {
            continue;
        }

        // Get duration (try ffprobe, fallback to estimation from file size)
        let duration = get_video_duration(&path).unwrap_or(0.0);

        recordings.push(RecordingFile {
            path: path.to_string_lossy().to_string(),
            name: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
            size: metadata.len(),
            duration,
            created_at: modified,  // Use modified as proxy for created
            modified_at: modified,
        });
    }

    // Sort by modified time (newest first)
    recordings.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));

    Ok(recordings)
}

/// Get video duration using ffprobe if available
fn get_video_duration(path: &std::path::Path) -> Option<f64> {
    // Try ffprobe first
    let output = std::process::Command::new("ffprobe")
        .args([
            "-v", "error",
            "-show_entries", "format=duration",
            "-of", "default=noprint_wrappers=1:nokey=1",
            path.to_str()?,
        ])
        .output()
        .ok()?;

    if output.status.success() {
        let duration_str = String::from_utf8_lossy(&output.stdout);
        return duration_str.trim().parse().ok();
    }

    // Fallback: estimate from file size (rough: ~5MB per minute for 1080p)
    let size = std::fs::metadata(path).ok()?.len();
    Some((size as f64) / (5.0 * 1024.0 * 1024.0) * 60.0)
}
```

---

**New UI Component:** `src/lib/components/recording-file-picker.svelte`

Dialog for user to select from multiple recording candidates:

```svelte
<script lang="ts">
  import type { RecordingFile } from '$lib/utils/recording-file-selector';
  import { formatDuration, formatFileSize } from '$lib/utils/format';

  export let candidates: RecordingFile[];
  export let onSelect: (file: RecordingFile) => void;
  export let onCancel: () => void;
</script>

<Dialog open={true}>
  <DialogContent>
    <DialogHeader>
      <DialogTitle>{$t('recording.selectFile.title')}</DialogTitle>
      <DialogDescription>
        {$t('recording.selectFile.description', { count: candidates.length })}
      </DialogDescription>
    </DialogHeader>

    <div class="recording-list">
      {#each candidates as file}
        <button
          class="recording-item"
          onclick={() => onSelect(file)}
        >
          <div class="recording-info">
            <span class="filename">{file.name}</span>
            <div class="meta">
              <span class="duration">{formatDuration(file.duration)}</span>
              <span class="size">{formatFileSize(file.size)}</span>
              <span class="date">{new Date(file.modifiedAt).toLocaleTimeString()}</span>
            </div>
          </div>
          <ChevronRight />
        </button>
      {/each}
    </div>

    <DialogFooter>
      <Button variant="ghost" onclick={onCancel}>
        {$t('common.cancel')}
      </Button>
      <Button variant="outline" onclick={() => onSelect(candidates[0])}>
        {$t('recording.selectFile.useLatest')}
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
```

---

### Phase 2: Event Session Store

**New File:** `src/lib/stores/event-session-store.ts`

```typescript
// Manages the current event session lifecycle
interface EventSessionStore {
  // Current session state
  session: EventSession | null;

  // Actions
  startSession(eventId: string): void;
  endSession(): void;
  updateState(state: EventSessionState): void;

  // Event handlers (called from OBS/YouTube state changes)
  onOBSConnected(): void;
  onOBSDisconnected(): void;
  onStreamStarted(): void;
  onStreamStopped(): void;
  onRecordStarted(): void;
  onRecordStopped(filePath?: string): void;
  onYouTubeLive(): void;
  onYouTubeComplete(): void;

  // Pause/Resume
  pause(reason: string): void;
  resume(): void;
}
```

**Persistence:** Store session state in Tauri store to survive app restarts.

---

### Phase 3: Rust Backend for Video Upload

**New File:** `src-tauri/src/video_upload.rs`

Rust module for efficient video file streaming and upload:

```rust
use std::path::PathBuf;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use tauri::Emitter;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE};

/// Get file metadata (size, exists)
#[tauri::command]
pub async fn get_video_file_info(path: String) -> Result<VideoFileInfo, String> {
    let metadata = std::fs::metadata(&path).map_err(|e| e.to_string())?;
    Ok(VideoFileInfo {
        path,
        size: metadata.len(),
        exists: true,
    })
}

/// Initialize YouTube resumable upload session
/// Returns the upload URI for subsequent chunk uploads
#[tauri::command]
pub async fn init_youtube_upload(
    access_token: String,
    file_path: String,
    title: String,
    description: String,
    privacy_status: String,
) -> Result<String, String> {
    // POST to YouTube API to get resumable upload URI
    // Returns: upload_uri
}

/// Upload a chunk of the video file
/// Handles resumable upload protocol with proper Content-Range headers
#[tauri::command]
pub async fn upload_video_chunk(
    app: tauri::AppHandle,
    upload_uri: String,
    file_path: String,
    start_byte: u64,
    chunk_size: u64,
) -> Result<UploadChunkResult, String> {
    // Read chunk from file at offset
    // PUT to upload_uri with Content-Range header
    // Emit progress event to frontend
    // Return bytes uploaded or video ID if complete
}

/// Query upload status (for resuming interrupted uploads)
#[tauri::command]
pub async fn get_upload_status(upload_uri: String) -> Result<u64, String> {
    // PUT with Content-Range: bytes */total
    // Returns last byte received by YouTube
}

/// Cancel an in-progress upload
#[tauri::command]
pub async fn cancel_upload(upload_uri: String) -> Result<(), String> {
    // DELETE the upload URI
}

#[derive(serde::Serialize)]
pub struct VideoFileInfo {
    path: String,
    size: u64,
    exists: bool,
}

#[derive(serde::Serialize)]
pub struct UploadChunkResult {
    bytes_uploaded: u64,
    total_bytes: u64,
    completed: bool,
    video_id: Option<String>,  // Set when upload completes
}
```

**Cargo.toml additions:**
```toml
[dependencies]
reqwest = { version = "0.12", features = ["json", "stream"] }
tokio = { version = "1", features = ["fs", "io-util"] }
```

**Progress Events:**
The Rust backend emits events to the frontend for real-time progress:
```rust
app.emit("upload-progress", UploadProgress {
    bytes_uploaded: current,
    total_bytes: total,
    percentage: (current as f64 / total as f64) * 100.0,
});
```

---

### Phase 4: Frontend Upload Service

**New File:** `src/lib/utils/youtube-upload.ts`

TypeScript wrapper that calls Rust commands:

```typescript
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

interface UploadProgress {
  bytesUploaded: number;
  totalBytes: number;
  percentage: number;
  status: 'initializing' | 'uploading' | 'processing' | 'completed' | 'failed';
}

const CHUNK_SIZE = 10 * 1024 * 1024; // 10MB chunks (YouTube recommends 5-10MB)

class YouTubeUploadService {
  private uploadUri: string | null = null;
  private abortController: AbortController | null = null;

  // Initialize upload session
  async initializeUpload(
    filePath: string,
    metadata: { title: string; description: string; privacyStatus: string },
    accessToken: string
  ): Promise<string> {
    const uploadUri = await invoke<string>('init_youtube_upload', {
      accessToken,
      filePath,
      title: metadata.title,
      description: metadata.description,
      privacyStatus: metadata.privacyStatus,
    });
    this.uploadUri = uploadUri;
    return uploadUri;
  }

  // Upload file with progress callback
  async uploadFile(
    uploadUri: string,
    filePath: string,
    onProgress: (progress: UploadProgress) => void
  ): Promise<string> {
    // Get file info
    const fileInfo = await invoke<{ size: number }>('get_video_file_info', { path: filePath });
    const totalBytes = fileInfo.size;

    // Listen for progress events from Rust
    const unlisten = await listen<UploadProgress>('upload-progress', (event) => {
      onProgress(event.payload);
    });

    try {
      let bytesUploaded = 0;

      // Upload in chunks
      while (bytesUploaded < totalBytes) {
        const result = await invoke<{
          bytes_uploaded: number;
          completed: boolean;
          video_id?: string;
        }>('upload_video_chunk', {
          uploadUri,
          filePath,
          startByte: bytesUploaded,
          chunkSize: CHUNK_SIZE,
        });

        bytesUploaded = result.bytes_uploaded;

        if (result.completed && result.video_id) {
          return result.video_id;
        }
      }

      throw new Error('Upload completed but no video ID received');
    } finally {
      unlisten();
    }
  }

  // Resume interrupted upload
  async resumeUpload(uploadUri: string): Promise<number> {
    return invoke<number>('get_upload_status', { uploadUri });
  }

  // Cancel upload
  async cancelUpload(): Promise<void> {
    if (this.uploadUri) {
      await invoke('cancel_upload', { uploadUri: this.uploadUri });
      this.uploadUri = null;
    }
  }

  // Update video privacy after processing
  async publishVideo(videoId: string, accessToken: string): Promise<void> {
    // Use YouTube Data API to update video status
    const response = await fetch(
      `https://www.googleapis.com/youtube/v3/videos?part=status`,
      {
        method: 'PUT',
        headers: {
          Authorization: `Bearer ${accessToken}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          id: videoId,
          status: { privacyStatus: 'public' },
        }),
      }
    );
    if (!response.ok) {
      throw new Error('Failed to publish video');
    }
  }
}

export const youtubeUpload = new YouTubeUploadService();
```

---

### Phase 5: Post-Event Automation Service

**New File:** `src/lib/services/post-event-automation.ts`

```typescript
import { uploadManager } from './upload/upload-manager';
import { uploadSettingsStore } from '$lib/stores/upload-settings-store';
import { eventStore } from '$lib/stores/event-store';
import { youtubeApi } from '$lib/utils/youtube-api';
import type { UploadMetadata } from './upload/upload-service.interface';

class PostEventAutomationService {
  // Main automation workflow
  async runPostEventWorkflow(session: EventSession): Promise<void> {
    // 1. Validate session meets criteria
    if (!this.shouldAutomate(session)) return;

    // 2. Get recording file path
    const recordingPath = session.recordingFilePath;
    if (!recordingPath) throw new Error('No recording file found');

    // 3. Get event details for metadata
    const event = await eventStore.getEvent(session.eventId);
    const settings = await uploadSettingsStore.getSettings();

    // 4. Prepare upload metadata from event
    const metadata: UploadMetadata = {
      title: generateCalculatedTitle(event),
      description: generateYoutubeDescription(event),
      privacy: event.uploads?.youtube?.privacyStatus || 'public',
      tags: [event.speaker, 'sermon', 'church'].filter(Boolean),
    };

    // 5. Upload to all enabled platforms
    const results = await uploadManager.uploadToAllPlatforms(
      recordingPath,
      metadata,
      (platform, progress) => {
        // Update event with per-platform progress
        eventStore.updateEvent(event.id, {
          uploads: {
            ...event.uploads,
            [platform]: {
              ...event.uploads?.[platform],
              uploadState: 'uploading',
              uploadProgress: progress.percentage,
            }
          }
        });
      }
    );

    // 6. Update event with results for each platform
    for (const [platform, result] of results) {
      await eventStore.updateEvent(event.id, {
        uploads: {
          ...event.uploads,
          [platform]: {
            videoId: result.videoId,
            uploadState: 'completed',
            uploadProgress: 100,
          }
        }
      });

      // 7. Platform-specific finalization
      if (platform === 'youtube') {
        // Publish the uploaded video if configured
        const ytConfig = settings.platforms.find(p => p.platform === 'youtube');
        if (ytConfig?.publishAfterUpload) {
          await uploadManager.services.get('youtube')?.finalize(result);
        }

        // End the original broadcast (if not already)
        if (event.youtubeScheduledId && event.youtubeLifeCycleStatus !== 'complete') {
          await youtubeApi.endBroadcast(event.youtubeScheduledId);
        }
      }
    }

    // 8. Notify user of completion
    showToast.success($t('eventSession.automation.completed'));
  }

  // Check if automation should run
  shouldAutomate(session: EventSession): boolean {
    const settings = await uploadSettingsStore.getSettings();
    const duration = (session.recordEndedAt || 0) - (session.recordStartedAt || 0);
    const minDuration = settings.minDurationMinutes * 60 * 1000;

    // Check if any platform has auto-upload enabled
    const hasEnabledPlatform = settings.platforms.some(p => p.enabled && p.autoUpload);

    return (
      hasEnabledPlatform &&
      session.wasOBSConnected &&
      session.wasRecording &&
      duration >= minDuration &&
      !isAnyOutputActive() // Nothing currently active
    );
  }
}

export const postEventAutomation = new PostEventAutomationService();
```

---

### Phase 6: Connection Resilience

**Modify:** `src/lib/utils/obs-websocket.ts`

Add connection state tracking with pause/resume:

```typescript
// In ConnectionClosed handler:
this.obs.on('ConnectionClosed', () => {
  // Notify session store to pause
  eventSessionStore.pause('OBS connection lost');
});

// In ConnectionOpened handler:
this.obs.on('ConnectionOpened', () => {
  // Resume if we were paused
  eventSessionStore.resume();
});
```

**Modify:** `src/lib/stores/refresh-store.ts`

Add reconnection monitoring:

```typescript
async sync() {
  // Existing OBS reconnection logic...

  // Check if session was paused and can resume
  const session = get(eventSessionStore);
  if (session?.state === 'PAUSED' && $systemStore.obs) {
    eventSessionStore.resume();
  }
}
```

---

### Phase 7: UI Components

**New File:** `src/lib/components/event-session-status.svelte`

Display current session state and automation progress:

```svelte
<script>
  import { eventSessionStore } from '$lib/stores/event-session-store';
  import { uploadManager } from '$lib/services/upload/upload-manager';
</script>

{#if $eventSessionStore}
  <div class="session-status">
    <Badge variant={getVariant($eventSessionStore.state)}>
      {$eventSessionStore.state}
    </Badge>

    {#if $eventSessionStore.state === 'FINALIZING'}
      <!-- Show progress for each platform -->
      {#each $eventSessionStore.uploadProgress as platform}
        <div class="platform-upload">
          <span>{platform.name}</span>
          <Progress value={platform.percentage} />
          <span>{platform.status}</span>
        </div>
      {/each}
    {/if}

    {#if $eventSessionStore.state === 'PAUSED'}
      <Alert variant="warning">
        Connection lost: {$eventSessionStore.pauseReason}
      </Alert>
    {/if}
  </div>
{/if}
```

**New File:** `src/lib/components/upload-settings.svelte`

Settings panel for configuring upload platforms:

```svelte
<script lang="ts">
  import { uploadSettingsStore } from '$lib/stores/upload-settings-store';
  import type { UploadSettings, YouTubeUploadConfig } from '$lib/types/upload-config';

  let settings: UploadSettings;

  // Load settings on mount
  onMount(async () => {
    settings = await uploadSettingsStore.getSettings();
  });
</script>

<Card>
  <CardHeader>
    <CardTitle>{$t('settings.upload.title')}</CardTitle>
    <CardDescription>{$t('settings.upload.description')}</CardDescription>
  </CardHeader>
  <CardContent>
    <!-- Global settings -->
    <div class="setting-group">
      <Label>{$t('settings.upload.minDuration')}</Label>
      <Input
        type="number"
        bind:value={settings.minDurationMinutes}
        min="1"
        max="180"
      />
      <span class="hint">{$t('settings.upload.minDurationHint')}</span>
    </div>

    <!-- YouTube settings -->
    <div class="platform-section">
      <div class="platform-header">
        <YoutubeIcon />
        <h3>YouTube</h3>
        <Switch
          checked={getYouTubeConfig()?.enabled}
          onCheckedChange={(v) => togglePlatform('youtube', v)}
        />
      </div>

      {#if getYouTubeConfig()?.enabled}
        <div class="platform-options">
          <Checkbox
            checked={getYouTubeConfig()?.autoUpload}
            label={$t('settings.upload.youtube.autoUpload')}
          />
          <Checkbox
            checked={getYouTubeConfig()?.publishAfterUpload}
            label={$t('settings.upload.youtube.autoPublish')}
          />
          <Select
            value={getYouTubeConfig()?.defaultPrivacy}
            label={$t('settings.upload.youtube.defaultPrivacy')}
          >
            <option value="public">Public</option>
            <option value="unlisted">Unlisted</option>
            <option value="private">Private</option>
          </Select>
        </div>
      {/if}
    </div>

    <!-- Facebook settings (future - show as "coming soon") -->
    <div class="platform-section disabled">
      <div class="platform-header">
        <FacebookIcon />
        <h3>Facebook</h3>
        <Badge variant="secondary">{$t('common.comingSoon')}</Badge>
      </div>
    </div>
  </CardContent>
</Card>
```

**Modify:** `src/lib/components/sidebar.svelte`

Add session status indicator.

---

### Phase 8: Localization

**Modify:** `src/lib/locales/en.json` and `hu.json`

Add new strings:

```json
{
  "eventSession": {
    "states": {
      "idle": "Idle",
      "preparing": "Preparing",
      "active": "Active",
      "finalizing": "Finalizing",
      "completed": "Completed",
      "paused": "Paused"
    },
    "automation": {
      "uploadingRecording": "Uploading recording...",
      "uploadingTo": "Uploading to {platform}...",
      "publishingVideo": "Publishing video...",
      "endingBroadcast": "Ending live broadcast...",
      "completed": "Post-event automation completed",
      "failed": "Automation failed: {error}"
    },
    "connection": {
      "lost": "Connection lost",
      "reconnecting": "Reconnecting...",
      "resumed": "Connection restored"
    }
  },
  "settings": {
    "upload": {
      "title": "Upload Settings",
      "description": "Configure automatic upload to video platforms after events",
      "minDuration": "Minimum duration (minutes)",
      "minDurationHint": "Events shorter than this won't trigger auto-upload",
      "retryAttempts": "Retry attempts",
      "chunkSize": "Chunk size (MB)",
      "youtube": {
        "title": "YouTube",
        "autoUpload": "Auto-upload recording after event",
        "autoPublish": "Publish video automatically when ready",
        "defaultPrivacy": "Default privacy",
        "useEventPrivacy": "Use event-specific privacy setting"
      },
      "facebook": {
        "title": "Facebook",
        "autoUpload": "Auto-upload to Facebook Page",
        "selectPage": "Select Page",
        "crosspost": "Also post to timeline"
      }
    }
  },
  "recording": {
    "selectFile": {
      "title": "Select Recording",
      "description": "Multiple recordings found ({count}). Please select the correct one to upload.",
      "useLatest": "Use Latest",
      "autoSelected": "Recording auto-selected: {filename}",
      "noRecordingFound": "No suitable recording found",
      "multipleLongVideos": "Multiple long recordings found, please select one"
    }
  },
  "common": {
    "comingSoon": "Coming soon",
    "cancel": "Cancel"
  }
}
```

---

## File Summary

### New Files
| File | Purpose |
|------|---------|
| **Types** | |
| `src/lib/types/upload-config.ts` | Multi-platform upload configuration types |
| `src/lib/types/event-session.ts` | Event session state machine types |
| **Stores** | |
| `src/lib/stores/event-session-store.ts` | Event lifecycle state machine |
| `src/lib/stores/upload-settings-store.ts` | Persisted upload platform settings |
| **Utils** | |
| `src/lib/utils/recording-file-selector.ts` | Auto-select recording from multiple files |
| **Services** | |
| `src/lib/services/upload/upload-service.interface.ts` | Common upload interface |
| `src/lib/services/upload/upload-manager.ts` | Multi-platform upload orchestrator |
| `src/lib/services/upload/youtube-upload.service.ts` | YouTube implementation |
| `src/lib/services/upload/facebook-upload.service.ts` | Facebook implementation (stub for future) |
| `src/lib/services/post-event-automation.ts` | Post-event workflow orchestration |
| **Components** | |
| `src/lib/components/event-session-status.svelte` | UI for session/upload status |
| `src/lib/components/upload-settings.svelte` | Settings UI for platform configuration |
| `src/lib/components/recording-file-picker.svelte` | Dialog for manual recording selection |

### Modified Files
| File | Changes |
|------|---------|
| `src/lib/utils/obs-websocket.ts` | Add `getLastRecordedFile()`, session hooks |
| `src/lib/stores/refresh-store.ts` | Add session resume logic |
| `src/lib/components/sidebar.svelte` | Add session status display |
| `src/lib/locales/en.json` | Add localization strings |
| `src/lib/locales/hu.json` | Add localization strings |

### Tauri Backend (Rust)
| File | Purpose |
|------|---------|
| `src-tauri/src/video_upload.rs` | **NEW** - Video file streaming & chunked upload |
| `src-tauri/src/lib.rs` | Register new video upload commands |
| `src-tauri/Cargo.toml` | Add `reqwest` with streaming support |

**Rust Commands:**
- `scan_recording_directory(dir, start, end)` - Find video files in time window with duration
- `get_video_file_info(path)` - Get file size and validate existence
- `init_youtube_upload(token, path, title, desc, privacy)` - Start resumable upload session
- `upload_video_chunk(uri, path, start, size)` - Upload a chunk with progress events
- `get_upload_status(uri)` - Query bytes uploaded (for resume)
- `cancel_upload(uri)` - Cancel in-progress upload

---

## Edge Cases & Error Handling

1. **Recording file not found**: Show error, allow manual retry
2. **Upload interrupted**: Resume from last chunk using resumable upload
3. **YouTube API quota exceeded**: Queue upload for later, notify user
4. **App closed during upload**: Persist upload URI, resume on next launch
5. **Multiple events same day**: Track by event ID, not date
6. **Manual stream stop before 45min**: Don't auto-upload, log reason
7. **OBS crash vs intentional stop**: Use timeout + state to differentiate

---

## Testing Strategy

1. **Unit tests**: State machine transitions, duration calculations
2. **Integration tests**: OBS WebSocket event handling
3. **E2E tests**: Full workflow with mock YouTube API
4. **Manual testing**: Real OBS + YouTube with short duration threshold

---

## Implementation Order

1. Phase 1: OBS recording path detection
2. Phase 2: Event session store (state machine)
3. Phase 3: Rust backend for video upload
4. Phase 4: Frontend upload service (TypeScript wrapper)
5. Phase 5: Post-event automation service
6. Phase 6: Connection resilience
7. Phase 7: UI components
8. Phase 8: Localization

---

## Design Decisions (Confirmed)

1. **Recording path**: Query OBS via WebSocket API (`GetRecordDirectory`)
2. **Upload timing**: Automatic - starts immediately when all conditions are met
3. **Privacy status**: Default public, but user can configure per-event (add `youtubeUploadPrivacyStatus` field to event)
4. **Retry policy**: 3 retries with exponential backoff, then mark as failed and notify user
5. **Notification**: Toast notification when automation completes or fails
