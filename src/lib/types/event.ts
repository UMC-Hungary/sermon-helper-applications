import type { BibleVerse, BibleTranslation } from './bible';

export type VideoUploadState = 'pending' | 'uploading' | 'completed' | 'failed';
export type YouTubePrivacyStatus = 'public' | 'private' | 'unlisted';
export type YouTubeLifeCycleStatus = 'created' | 'ready' | 'testing' | 'live' | 'complete';

// ============================================
// Constants
// ============================================

export const MIN_RECORDING_DURATION_SECONDS = 2 * 60; // 2 minutes
export const CURRENT_EVENT_VERSION = 'v0.7.0';

// ============================================
// Session State Types
// ============================================

// Event session lifecycle states
export type EventSessionState =
	| 'IDLE' // Initial state, nothing active
	| 'PREPARING' // OBS connected, waiting for stream/record
	| 'ACTIVE' // Streaming and/or recording in progress
	| 'FINALIZING' // Post-event automation in progress
	| 'COMPLETED'; // Event fully processed

// Session activity types — append-only log entries
export type SessionActivityType =
	| 'SESSION_STARTED'
	| 'OBS_CONNECTED'
	| 'OBS_DISCONNECTED'
	| 'STREAM_STARTED'
	| 'STREAM_STOPPED'
	| 'RECORD_STARTED'
	| 'RECORD_STOPPED'
	| 'YOUTUBE_LIVE'
	| 'SESSION_FINALIZING'
	| 'SESSION_COMPLETED'
	| 'SESSION_ERROR'
	| 'SESSION_ENDED';

export interface SessionActivity {
	type: SessionActivityType;
	timestamp: number;
	message?: string;
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

// Recording entry stored within an event
export interface EventRecording {
	id: string;
	file: RecordingFile;
	detectedAt: number; // When OBS reported this recording

	// User selection
	whitelisted: boolean; // User manually selected for upload (overrides 10 min rule)

	// Upload state
	uploaded: boolean;
	uploadSession?: {
		uploadUri: string; // YouTube resumable upload URI
		fileSize: number;
		bytesUploaded: number;
		platform: string;
		startedAt: number;
	};
	uploadedAt?: number;
	videoId?: string; // YouTube video ID after upload
	videoUrl?: string;

	// User customization (for multi-recording events)
	customTitle?: string; // Override title for this specific recording
}

export interface ServiceEvent {
	id: string;
	title: string;
	date: string; // YYYY-MM-DD format
	time: string; // HH:MM format
	speaker: string;
	description: string;

	// Bible references
	textus: string;
	leckio: string;
	textusTranslation: BibleTranslation;
	leckioTranslation: BibleTranslation;
	textusVerses: BibleVerse[];
	leckioVerses: BibleVerse[];

	// YouTube integration (optional)
	youtubeScheduledId?: string;
	youtubeUploadedId?: string;
	youtubePrivacyStatus: YouTubePrivacyStatus;
	youtubeLifeCycleStatus?: YouTubeLifeCycleStatus;
	videoUploadState?: VideoUploadState;

	// Transient UI state (not persisted)
	isBroadcastScheduling: boolean;

	// Per-event upload settings
	autoUploadEnabled: boolean; // Default true for new events
	uploadPrivacyStatus: YouTubePrivacyStatus; // Privacy for uploaded recording (separate from live broadcast)

	// PPTX generation timestamps
	textusGeneratedAt?: string;
	leckioGeneratedAt?: string;

	// Metadata
	createdAt: string;
	updatedAt: string;

	// Schema version
	version: string;

	// Session activity log (append-only)
	activities?: SessionActivity[];

	// Recording files
	recordings?: EventRecording[];
	recordingDirectory?: string; // Informational only
}

// Generate a unique ID
export function generateEventId(): string {
	return crypto.randomUUID();
}

// Get next Sunday date
// If today is Sunday and time < 10:45, return today
// Otherwise, find next Sunday
export function getNextSundayDate(): string {
	const now = new Date();
	const dayOfWeek = now.getDay(); // 0 = Sunday
	const hours = now.getHours();
	const minutes = now.getMinutes();
	const currentTimeInMinutes = hours * 60 + minutes;
	const cutoffTimeInMinutes = 10 * 60 + 45; // 10:45

	let daysUntilSunday: number;

	if (dayOfWeek === 0 && currentTimeInMinutes < cutoffTimeInMinutes) {
		// Today is Sunday and before 10:45
		daysUntilSunday = 0;
	} else {
		// Find next Sunday
		daysUntilSunday = (7 - dayOfWeek) % 7;
		if (daysUntilSunday === 0) {
			// Today is Sunday but after 10:45, go to next Sunday
			daysUntilSunday = 7;
		}
	}

	const nextSunday = new Date(now);
	nextSunday.setDate(now.getDate() + daysUntilSunday);

	// Format as YYYY-MM-DD
	const year = nextSunday.getFullYear();
	const month = String(nextSunday.getMonth() + 1).padStart(2, '0');
	const day = String(nextSunday.getDate()).padStart(2, '0');

	return `${year}-${month}-${day}`;
}

// Generate calculated event title for YouTube
// Format: YYYY.MM.DD. Title | Textus: ... Lekció: ... | Speaker name
// Each section after the first is optional
// Max 100 characters
export function generateCalculatedTitle(event: ServiceEvent): string {
	if (!event.date) return '';

	// Format date as YYYY.MM.DD.
	const [year, month, day] = event.date.split('-');
	const dateStr = `${year}.${month}.${day}.`;

	// Start with date and title
	let title = `${dateStr} ${event.title}`;

	// Add textus and leckio section if either is present
	const bibleParts: string[] = [];
	if (event.textus) {
		bibleParts.push(`Textus: ${event.textus}`);
	}
	if (event.leckio) {
		bibleParts.push(`Lekció: ${event.leckio}`);
	}

	if (bibleParts.length > 0) {
		title += ` | ${bibleParts.join(' ')}`;
	}

	// Add speaker if present
	if (event.speaker) {
		title += ` | ${event.speaker}`;
	}

	return title;
}

// Get calculated title length (for character counter)
export function getCalculatedTitleLength(event: ServiceEvent): number {
	return generateCalculatedTitle(event).length;
}

// Create an empty event with defaults
export function createEmptyEvent(): ServiceEvent {
	const now = new Date().toISOString();
	return {
		id: generateEventId(),
		title: '(vasárnapi) istentisztelet',
		date: getNextSundayDate(),
		time: '10:00',
		speaker: '',
		description: '',
		textus: '',
		leckio: '',
		textusTranslation: 'UF_v2',
		leckioTranslation: 'UF_v2',
		textusVerses: [],
		leckioVerses: [],
		youtubePrivacyStatus: 'public',
		autoUploadEnabled: true,
		uploadPrivacyStatus: 'public',
		createdAt: now,
		updatedAt: now,
		isBroadcastScheduling: false,
		version: CURRENT_EVENT_VERSION,
		recordings: []
	};
}

// Get today's date as YYYY-MM-DD in local timezone
export function getLocalToday(): string {
	const now = new Date();
	const year = now.getFullYear();
	const month = String(now.getMonth() + 1).padStart(2, '0');
	const day = String(now.getDate()).padStart(2, '0');
	return `${year}-${month}-${day}`;
}

// Check if an event is scheduled for today
export function isEventToday(event: ServiceEvent): boolean {
	return event.date === getLocalToday();
}

// Check if an event is in the future (including today)
export function isEventUpcoming(event: ServiceEvent): boolean {
	return event.date >= getLocalToday();
}

// Sort events by date and time (ascending)
export function sortEventsByDate(events: ServiceEvent[]): ServiceEvent[] {
	return [...events].sort((a, b) => {
		const dateCompare = a.date.localeCompare(b.date);
		if (dateCompare !== 0) return dateCompare;
		return a.time.localeCompare(b.time);
	});
}

// Format event date for display
export function formatEventDate(date: string): string {
	if (!date) return '';
	const d = new Date(date);
	return d.toLocaleDateString(undefined, {
		weekday: 'long',
		year: 'numeric',
		month: 'long',
		day: 'numeric',
	});
}

// Format event time for display
export function formatEventTime(time: string): string {
	if (!time) return '';
	return time;
}

// ============================================
// Upload Eligibility Helpers
// ============================================

// Check if an event is "uploadable"
// An event is uploadable if:
// - Has at least one recording > MIN_RECORDING_DURATION or whitelisted
// - AND session has been manually finished or completed
export function isEventUploadable(event: ServiceEvent): boolean {
	const recordings = event.recordings ?? [];
	if (recordings.length === 0) return false;

	// Check if there are any uploadable recordings (not yet uploaded)
	const uploadableRecordings = getUploadableRecordings(event);
	if (uploadableRecordings.length === 0) return false;

	return deriveSessionState(event.activities) === 'COMPLETED';
}

// Get recordings that are eligible for upload
// Recordings > MIN_RECORDING_DURATION OR whitelisted, not yet uploaded
export function getUploadableRecordings(event: ServiceEvent): EventRecording[] {
	const recordings = event.recordings ?? [];
	return recordings.filter((rec) => {
		if (rec.uploaded) return false;
		const meetsMinDuration = rec.file.duration >= MIN_RECORDING_DURATION_SECONDS;
		return meetsMinDuration || rec.whitelisted;
	});
}

// Check if event has multiple uploadable recordings (needs user selection)
export function hasMultipleUploadableRecordings(event: ServiceEvent): boolean {
	return getUploadableRecordings(event).length > 1;
}

// Check if event has an uploaded recording
export function hasUploadedRecording(event: ServiceEvent): boolean {
	return !!event.youtubeUploadedId;
}

// Recording status type for display
export type EventRecordingStatus = 'none' | 'pending' | 'uploading' | 'uploaded' | 'failed';

// Get the recording/upload status for an event (simplified)
export function getRecordingStatus(event: ServiceEvent): EventRecordingStatus {
	// Check if we have an uploaded recording
	if (event.youtubeUploadedId) {
		return 'uploaded';
	}

	// Check recordings array
	const recordings = event.recordings ?? [];
	if (recordings.length === 0) {
		return 'none';
	}

	// Check if any recording is uploaded
	const anyUploaded = recordings.some((rec) => rec.uploaded);
	if (anyUploaded) {
		return 'uploaded';
	}

	// Check if there are uploadable recordings
	const uploadable = getUploadableRecordings(event);
	if (uploadable.length > 0) {
		return 'pending';
	}

	// Has recordings but none are uploadable
	return 'none';
}

// Get upload progress percentage (simplified - returns 0 or 100)
export function getUploadProgress(event: ServiceEvent): number {
	if (event.youtubeUploadedId) return 100;
	const recordings = event.recordings ?? [];
	const anyUploaded = recordings.some((rec) => rec.uploaded);
	return anyUploaded ? 100 : 0;
}

// Get the YouTube video URL for the uploaded recording
export function getYouTubeVideoUrl(event: ServiceEvent): string | null {
	if (!event.youtubeUploadedId) return null;
	return `https://www.youtube.com/watch?v=${event.youtubeUploadedId}`;
}

// Get the YouTube broadcast URL (for live broadcast)
export function getYouTubeBroadcastUrl(event: ServiceEvent): string | null {
	if (!event.youtubeScheduledId) return null;
	return `https://www.youtube.com/watch?v=${event.youtubeScheduledId}`;
}

// Get YouTube Studio URL for the uploaded video
export function getYouTubeStudioUploadUrl(event: ServiceEvent): string | null {
	if (!event.youtubeUploadedId) return null;
	return `https://studio.youtube.com/video/${event.youtubeUploadedId}/edit`;
}

// ============================================
// Session Activity Helpers
// ============================================

// Push a new activity to the log
export function pushActivity(
	activities: SessionActivity[],
	type: SessionActivityType,
	message?: string
): SessionActivity[] {
	return [...activities, { type, timestamp: Date.now(), message }];
}

// Map activity types to the session state they produce
const STATE_PRODUCING_ACTIVITIES: Record<string, EventSessionState> = {
	SESSION_ENDED: 'IDLE',
	SESSION_COMPLETED: 'COMPLETED',
	SESSION_FINALIZING: 'FINALIZING',
	STREAM_STARTED: 'ACTIVE',
	RECORD_STARTED: 'ACTIVE',
	SESSION_ERROR: 'ACTIVE',
	SESSION_STARTED: 'PREPARING'
};

// Derive current session state from activities
export function deriveSessionState(activities?: SessionActivity[]): EventSessionState {
	if (!activities?.length) return 'IDLE';
	const lastStateActivity = activities.findLast((a) => a.type in STATE_PRODUCING_ACTIVITIES);
	return lastStateActivity ? STATE_PRODUCING_ACTIVITIES[lastStateActivity.type] : 'IDLE';
}

// Check if event has an active session
export function isSessionActive(event: ServiceEvent): boolean {
	const state = deriveSessionState(event.activities);
	return state === 'ACTIVE' || state === 'PREPARING';
}

// Check if session is complete
export function isSessionFinished(event: ServiceEvent): boolean {
	return deriveSessionState(event.activities) === 'COMPLETED';
}

// Check if event has any session state (session was started at some point)
export function hasSessionState(event: ServiceEvent): boolean {
	return deriveSessionState(event.activities) !== 'IDLE';
}

// Find last activity of a given type
export function getLastActivity(
	activities: SessionActivity[] | undefined,
	type: SessionActivityType
): SessionActivity | undefined {
	return activities?.findLast((a) => a.type === type);
}

// Check if any activity of a given type exists
export function hasActivity(
	activities: SessionActivity[] | undefined,
	type: SessionActivityType
): boolean {
	return activities?.some((a) => a.type === type) ?? false;
}

// Get last SESSION_ERROR message
export function getSessionError(activities?: SessionActivity[]): string | undefined {
	return getLastActivity(activities, 'SESSION_ERROR')?.message;
}

// Get session duration in milliseconds (from first stream/record start to last stop)
export function getSessionDuration(event: ServiceEvent): number {
	const activities = event.activities;
	if (!activities?.length) return 0;

	const startTypes: SessionActivityType[] = ['RECORD_STARTED', 'STREAM_STARTED'];
	const stopTypes: SessionActivityType[] = ['RECORD_STOPPED', 'STREAM_STOPPED'];

	const firstStart = activities.find((a) => startTypes.includes(a.type));
	const lastStop = activities.findLast((a) => stopTypes.includes(a.type));

	if (!firstStart) return 0;
	const endTime = lastStop?.timestamp ?? Date.now();
	return endTime - firstStart.timestamp;
}

// Check if session meets minimum duration for upload
export function meetsMinimumDuration(event: ServiceEvent, minMinutes: number): boolean {
	const durationMs = getSessionDuration(event);
	const minMs = minMinutes * 60 * 1000;
	return durationMs >= minMs;
}

// Create an EventRecording from a RecordingFile
export function createEventRecording(file: RecordingFile): EventRecording {
	return {
		id: crypto.randomUUID(),
		file,
		detectedAt: Date.now(),
		whitelisted: false,
		uploaded: false
	};
}

// Format duration in seconds to HH:MM:SS or MM:SS
export function formatDuration(seconds: number): string {
	const hrs = Math.floor(seconds / 3600);
	const mins = Math.floor((seconds % 3600) / 60);
	const secs = Math.floor(seconds % 60);

	if (hrs > 0) {
		return `${hrs}:${String(mins).padStart(2, '0')}:${String(secs).padStart(2, '0')}`;
	}
	return `${mins}:${String(secs).padStart(2, '0')}`;
}
