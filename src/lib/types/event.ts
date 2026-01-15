import type { BibleVerse, BibleTranslation } from './bible';

export type VideoUploadState = 'pending' | 'uploading' | 'completed' | 'failed';
export type YouTubePrivacyStatus = 'public' | 'private' | 'unlisted';
export type YouTubeLifeCycleStatus = 'created' | 'ready' | 'testing' | 'live' | 'complete';

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
