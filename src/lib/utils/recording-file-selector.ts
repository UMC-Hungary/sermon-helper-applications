// Recording file selection utility
// Handles automatic selection of the correct recording file when multiple exist

import type { RecordingFile, RecordingSelectionResult } from '$lib/types/event-session';

// Check if running in Tauri
function isTauri(): boolean {
	return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

// Dynamic import for Tauri API (only when in Tauri environment)
async function invokeCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
	if (!isTauri()) {
		throw new Error('Recording file selection is only available in the desktop app');
	}
	const { invoke } = await import('@tauri-apps/api/core');
	return invoke<T>(command, args);
}

// Default thresholds
const DEFAULT_SHORT_VIDEO_THRESHOLD_SECONDS = 10 * 60; // 10 minutes
const DEFAULT_MIN_UPLOAD_DURATION_SECONDS = 45 * 60; // 45 minutes

export interface RecordingSelectionOptions {
	shortVideoThresholdSeconds?: number; // Videos shorter than this are "short" (default: 10min)
	minUploadDurationSeconds?: number; // Minimum duration to upload (default: 45min)
}

/**
 * Find and select the appropriate recording file from OBS output directory
 *
 * Logic:
 * 1. Scan recording directory for video files created during session
 * 2. Filter by session time window (sessionStart - 5min to sessionEnd + 5min)
 * 3. Categorize as "long" (>10min) or "short" (<=10min)
 * 4. Auto-select if exactly 1 long video exists
 * 5. Return candidates if multiple long videos exist (for user selection)
 * 6. Return null if no suitable recordings found
 */
export async function selectRecording(
	recordingDir: string,
	sessionStartTime: number,
	sessionEndTime: number,
	options?: RecordingSelectionOptions
): Promise<RecordingSelectionResult> {
	const shortThreshold = options?.shortVideoThresholdSeconds ?? DEFAULT_SHORT_VIDEO_THRESHOLD_SECONDS;
	const minDuration = options?.minUploadDurationSeconds ?? DEFAULT_MIN_UPLOAD_DURATION_SECONDS;

	// Buffer around session time (5 minutes before and after)
	const timeBuffer = 5 * 60 * 1000;
	const scanStart = sessionStartTime - timeBuffer;
	const scanEnd = sessionEndTime + timeBuffer;

	try {
		// Get all video files from recording directory via Rust
		const allFiles = await invokeCommand<RecordingFile[]>('scan_recording_directory', {
			directory: recordingDir,
			sessionStart: scanStart,
			sessionEnd: scanEnd
		});

		console.log(`[RecordingSelector] Found ${allFiles.length} video files in time window`);

		if (allFiles.length === 0) {
			return {
				autoSelected: false,
				selectedFile: null,
				candidates: [],
				reason: 'no_long'
			};
		}

		// Categorize by duration
		const longVideos = allFiles.filter((f) => f.duration >= shortThreshold);
		const shortVideos = allFiles.filter((f) => f.duration < shortThreshold);

		console.log(
			`[RecordingSelector] Long videos: ${longVideos.length}, Short videos: ${shortVideos.length}`
		);

		// Apply selection logic
		if (longVideos.length === 0) {
			// No long videos found
			console.log('[RecordingSelector] No long videos found');
			return {
				autoSelected: false,
				selectedFile: null,
				candidates: allFiles,
				reason: 'no_long'
			};
		}

		if (longVideos.length === 1) {
			const selected = longVideos[0];

			// Check if it meets minimum upload duration
			if (selected.duration >= minDuration) {
				console.log(`[RecordingSelector] Auto-selected: ${selected.name} (${formatDuration(selected.duration)})`);
				return {
					autoSelected: true,
					selectedFile: selected,
					candidates: longVideos,
					reason: 'single_long'
				};
			} else {
				// Long enough to not be "short", but not long enough to upload
				console.log(
					`[RecordingSelector] Single long video found but below minimum duration: ${selected.name} (${formatDuration(selected.duration)})`
				);
				return {
					autoSelected: false,
					selectedFile: null,
					candidates: longVideos,
					reason: 'no_long'
				};
			}
		}

		// Multiple long videos - need user selection
		console.log(`[RecordingSelector] Multiple long videos found, user selection required`);
		return {
			autoSelected: false,
			selectedFile: null,
			candidates: longVideos,
			reason: 'multiple_long'
		};
	} catch (error) {
		console.error('[RecordingSelector] Error scanning recording directory:', error);
		throw error;
	}
}

/**
 * Format duration in seconds to human-readable string
 */
export function formatDuration(seconds: number): string {
	const hours = Math.floor(seconds / 3600);
	const minutes = Math.floor((seconds % 3600) / 60);
	const secs = Math.floor(seconds % 60);

	if (hours > 0) {
		return `${hours}h ${minutes}m ${secs}s`;
	} else if (minutes > 0) {
		return `${minutes}m ${secs}s`;
	} else {
		return `${secs}s`;
	}
}

/**
 * Format file size to human-readable string
 */
export function formatFileSize(bytes: number): string {
	if (bytes < 1024) return `${bytes} B`;
	if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
	if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
	return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

/**
 * Get the most recent recording file from candidates
 */
export function getMostRecentRecording(candidates: RecordingFile[]): RecordingFile | null {
	if (candidates.length === 0) return null;
	return candidates.reduce((latest, current) =>
		current.modifiedAt > latest.modifiedAt ? current : latest
	);
}

/**
 * Get the longest recording file from candidates
 */
export function getLongestRecording(candidates: RecordingFile[]): RecordingFile | null {
	if (candidates.length === 0) return null;
	return candidates.reduce((longest, current) =>
		current.duration > longest.duration ? current : longest
	);
}
