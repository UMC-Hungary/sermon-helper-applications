import type { BibleVerse, BibleTranslation } from './bible';
import type { UploadPlatform } from './upload-config';

export type VideoUploadState = 'pending' | 'uploading' | 'completed' | 'failed';
export type YouTubePrivacyStatus = 'public' | 'private' | 'unlisted';
export type YouTubeLifeCycleStatus = 'created' | 'ready' | 'testing' | 'live' | 'complete';

// Upload session stored within an event
export interface EventUploadSession {
	id: string;
	platform: UploadPlatform;
	filePath: string;
	fileSize: number;
	metadata: {
		title: string;
		description: string;
		privacy: string;
		tags?: string[];
		thumbnailPath?: string;
	};
	uploadUri: string; // Platform-specific resumable upload URI
	bytesUploaded: number;
	startedAt: number;
	status: 'pending' | 'uploading' | 'paused' | 'processing' | 'completed' | 'failed';
	error?: string;
	// Result when completed
	videoId?: string;
	videoUrl?: string;
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

	// Per-event upload settings
	autoUploadEnabled: boolean; // Default true for new events
	uploadPrivacyStatus: YouTubePrivacyStatus; // Privacy for uploaded recording (separate from live broadcast)

	// Upload sessions for resumable uploads (stored per-event)
	uploadSessions?: EventUploadSession[];

	// PPTX generation timestamps
	textusGeneratedAt?: string;
	leckioGeneratedAt?: string;

	// Metadata
	createdAt: string;
	updatedAt: string;
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
		textusTranslation: 'RUF_v2',
		leckioTranslation: 'RUF_v2',
		textusVerses: [],
		leckioVerses: [],
		youtubePrivacyStatus: 'public',
		autoUploadEnabled: true,
		uploadPrivacyStatus: 'public',
		createdAt: now,
		updatedAt: now,
	};
}

// Check if an event is scheduled for today
export function isEventToday(event: ServiceEvent): boolean {
	const today = new Date().toISOString().split('T')[0];
	return event.date === today;
}

// Check if an event is in the future (including today)
export function isEventUpcoming(event: ServiceEvent): boolean {
	const today = new Date().toISOString().split('T')[0];
	return event.date >= today;
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

// Check if event has pending/paused uploads that need attention
export function hasPendingUploads(event: ServiceEvent): boolean {
	if (!event.uploadSessions) return false;
	return event.uploadSessions.some(
		(s) => s.status === 'pending' || s.status === 'paused' || s.status === 'failed'
	);
}

// Get pending upload sessions from an event
export function getPendingUploadSessions(event: ServiceEvent): EventUploadSession[] {
	if (!event.uploadSessions) return [];
	return event.uploadSessions.filter(
		(s) => s.status === 'pending' || s.status === 'paused' || s.status === 'failed'
	);
}

// Get active (uploading) upload sessions from an event
export function getActiveUploadSessions(event: ServiceEvent): EventUploadSession[] {
	if (!event.uploadSessions) return [];
	return event.uploadSessions.filter((s) => s.status === 'uploading');
}

// Recording status type for display
export type EventRecordingStatus = 'none' | 'pending' | 'uploading' | 'paused' | 'uploaded' | 'failed';

// Get the recording/upload status for an event
export function getRecordingStatus(event: ServiceEvent): EventRecordingStatus {
	// If we have an uploaded video ID, it's uploaded
	if (event.youtubeUploadedId) {
		return 'uploaded';
	}

	// Check upload sessions
	if (event.uploadSessions && event.uploadSessions.length > 0) {
		// Get the most recent YouTube upload session
		const youtubeSession = event.uploadSessions
			.filter((s) => s.platform === 'youtube')
			.sort((a, b) => b.startedAt - a.startedAt)[0];

		if (youtubeSession) {
			switch (youtubeSession.status) {
				case 'uploading':
					return 'uploading';
				case 'paused':
					return 'paused';
				case 'failed':
					return 'failed';
				case 'pending':
					return 'pending';
				case 'completed':
				case 'processing':
					return 'uploaded';
			}
		}
	}

	// Check legacy videoUploadState
	if (event.videoUploadState) {
		switch (event.videoUploadState) {
			case 'uploading':
				return 'uploading';
			case 'completed':
				return 'uploaded';
			case 'failed':
				return 'failed';
			case 'pending':
				return 'pending';
		}
	}

	return 'none';
}

// Check if an event has an uploaded recording
export function hasUploadedRecording(event: ServiceEvent): boolean {
	return !!event.youtubeUploadedId;
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

// Get the upload progress percentage for an event
export function getUploadProgress(event: ServiceEvent): number {
	if (!event.uploadSessions || event.uploadSessions.length === 0) return 0;

	const youtubeSession = event.uploadSessions
		.filter((s) => s.platform === 'youtube')
		.sort((a, b) => b.startedAt - a.startedAt)[0];

	if (!youtubeSession) return 0;

	if (youtubeSession.fileSize === 0) return 0;
	return Math.round((youtubeSession.bytesUploaded / youtubeSession.fileSize) * 100);
}
