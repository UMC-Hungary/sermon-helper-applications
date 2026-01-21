// Event session state machine types

import type { UploadPlatform } from './upload-config';

// Event session lifecycle states
export type EventSessionState =
	| 'IDLE' // Initial state, nothing active
	| 'PREPARING' // OBS connected, waiting for stream/record
	| 'ACTIVE' // Streaming and/or recording in progress
	| 'FINALIZING' // Post-event automation in progress
	| 'COMPLETED' // Event fully processed
	| 'PAUSED'; // Connection lost during ACTIVE/FINALIZING

// Upload progress for a single platform
export interface PlatformUploadProgress {
	platform: UploadPlatform;
	name: string; // Display name (e.g., "YouTube", "Facebook")
	status: 'pending' | 'initializing' | 'uploading' | 'processing' | 'completed' | 'failed';
	bytesUploaded: number;
	totalBytes: number;
	percentage: number;
	error?: string;
	videoId?: string; // Set when upload completes
	videoUrl?: string;
}

// Recording file information
export interface RecordingFile {
	path: string;
	name: string;
	size: number; // bytes
	duration: number; // seconds (from file metadata)
	createdAt: number; // timestamp
	modifiedAt: number; // timestamp
}

// Recording selection result
export interface RecordingSelectionResult {
	autoSelected: boolean;
	selectedFile: RecordingFile | null;
	candidates: RecordingFile[]; // All files that could be the recording
	reason: 'single_long' | 'multiple_long' | 'no_long' | 'user_selected';
}

// Event session data
export interface EventSession {
	id: string; // Unique session ID
	eventId: string; // Associated event ID
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
	recordingDirectory?: string;
	recordingFilePath?: string;
	recordingSelection?: RecordingSelectionResult;

	// Upload progress (per platform)
	uploadProgress: PlatformUploadProgress[];

	// Connection resilience
	pausedAt?: number;
	pauseReason?: string;

	// Completion
	completedAt?: number;
	completionError?: string;
}

// Create a new session
export function createEventSession(eventId: string): EventSession {
	return {
		id: crypto.randomUUID(),
		eventId,
		state: 'IDLE',
		sessionStartedAt: Date.now(),
		wasOBSConnected: false,
		wasStreaming: false,
		wasRecording: false,
		wasYouTubeLive: false,
		uploadProgress: []
	};
}

// Check if session is in an active state
export function isSessionActive(session: EventSession): boolean {
	return session.state === 'ACTIVE' || session.state === 'PREPARING';
}

// Check if session is complete or failed
export function isSessionFinished(session: EventSession): boolean {
	return session.state === 'COMPLETED';
}

// Get session duration in milliseconds
export function getSessionDuration(session: EventSession): number {
	const endTime = session.recordEndedAt || session.streamEndedAt || Date.now();
	const startTime = session.recordStartedAt || session.streamStartedAt || session.sessionStartedAt;
	return endTime - startTime;
}

// Check if session meets minimum duration for upload
export function meetsMinimumDuration(session: EventSession, minMinutes: number): boolean {
	const durationMs = getSessionDuration(session);
	const minMs = minMinutes * 60 * 1000;
	return durationMs >= minMs;
}
