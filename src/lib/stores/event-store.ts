import { writable, derived, get, type Readable } from 'svelte/store';
import { appSettings, appSettingsStore } from '$lib/utils/app-settings-store';
import type {
	ServiceEvent,
	EventRecording,
	SessionActivityType
} from '$lib/types/event';
import {
	isEventToday,
	isEventUpcoming,
	sortEventsByDate,
	isSessionActive,
	getSessionDuration,
	deriveSessionState,
	pushActivity,
	isEventUploadable,
	getUploadableRecordings,
	getEventDate,
	getEventTime,
	getLocalToday,
	MIN_RECORDING_DURATION_SECONDS
} from '$lib/types/event';

// ============================================
// Internal Writable Store — Source of Truth
// ============================================

const _eventList = writable<ServiceEvent[]>([]);

// Public read-only export (same subscribe interface as before)
export const eventList: Readable<ServiceEvent[]> = { subscribe: _eventList.subscribe };

// ============================================
// Derived Stores - Event List
// ============================================

// Derived store for today's event (first non-completed event by time)
export const todayEvent = derived(eventList, ($eventList) => {
	const list = $eventList ?? [];
	const todayEvents = sortEventsByDate(list.filter(isEventToday));
	// Pick the first event by time that hasn't completed yet
	return todayEvents.find((e) => deriveSessionState(e.activities) !== 'COMPLETED') ?? null;
});

// Derived store for upcoming events (sorted, excludes completed events)
export const upcomingEvents = derived(eventList, ($eventList) => {
	const list = $eventList ?? [];
	return sortEventsByDate(list.filter((e) => isEventUpcoming(e) && deriveSessionState(e.activities) !== 'COMPLETED'));
});

// Derived store for past events (sorted, most recent first)
// Includes events with past dates OR completed events (even from today)
export const pastEvents = derived(eventList, ($eventList) => {
	const list = $eventList ?? [];
	const today = getLocalToday();
	return sortEventsByDate(list.filter((e) => getEventDate(e) < today || deriveSessionState(e.activities) === 'COMPLETED')).reverse();
});

// ============================================
// Derived Stores - Session State
// ============================================

// Current event — the event that receives OBS activities and recordings.
// Today's event always wins over past events (past active sessions are stale).
// When multiple today events exist, pick the one closest to the current time.
export const currentEvent = derived(eventList, ($eventList) => {
	const list = $eventList ?? [];
	const now = new Date();
	const nowMinutes = now.getHours() * 60 + now.getMinutes();

	const toMinutes = (event: ServiceEvent) => {
		const time = getEventTime(event);
		const [h, m] = time.split(':').map(Number);
		return h * 60 + m;
	};

	// Today's non-completed events, sorted by dateTime
	const todayEvents = sortEventsByDate(list.filter(isEventToday))
		.filter((e) => deriveSessionState(e.activities) !== 'COMPLETED');

	// Prefer the latest event whose time has already passed, otherwise the earliest upcoming
	const started = todayEvents.filter((e) => toMinutes(e) <= nowMinutes);
	const upcoming = todayEvents.filter((e) => toMinutes(e) > nowMinutes);

	return started.at(-1) ?? upcoming.at(0) ?? null;
});

// Current session state (derived from currentEvent)
export const sessionState = derived(currentEvent, ($event) => deriveSessionState($event?.activities));

// Is there an active session in progress?
export const isSessionInProgress = derived(currentEvent, ($event) =>
	$event ? isSessionActive($event) : false
);

// ============================================
// Derived Stores - Upload Queue
// ============================================

// Upload queue item
export interface UploadQueueItem {
	event: ServiceEvent;
	recording: EventRecording;
}

// Events that have uploadable recordings
export const uploadableEvents = derived(eventList, ($events) =>
	($events ?? []).filter(isEventUploadable)
);

// Flat queue of recordings ready to upload
export const uploadQueue = derived(eventList, ($events) =>
	($events ?? [])
		.filter((e) => isEventUploadable(e) && e.autoUploadEnabled)
		.flatMap((event) => {
			const uploadable = getUploadableRecordings(event);

			// Auto-queue: single recording > 10 mins
			const autoQueued: UploadQueueItem[] =
				uploadable.length === 1 &&
				uploadable[0].file.duration >= MIN_RECORDING_DURATION_SECONDS
					? [{ event, recording: uploadable[0] }]
					: [];

			// User-queued: whitelisted recordings (not already in auto)
			const userQueued: UploadQueueItem[] = uploadable
				.filter((rec) => rec.whitelisted)
				.filter((rec) => !autoQueued.some((q) => q.recording.id === rec.id))
				.map((recording) => ({ event, recording }));

			return [...autoQueued, ...userQueued];
		})
);

// Convenience stores for uploads
export const hasUploadsInQueue = derived(uploadQueue, ($queue) => $queue.length > 0);
export const uploadQueueCount = derived(uploadQueue, ($queue) => $queue.length);

// ============================================
// Persistence Effect
// ============================================

function setupPersistence() {
	let isInit = true;
	let timer: ReturnType<typeof setTimeout> | null = null;

	_eventList.subscribe((events) => {
		if (isInit) { isInit = false; return; }
		if (timer) clearTimeout(timer);
		timer = setTimeout(async () => {
			try {
				await appSettingsStore.set('eventList', events);
			} catch (error) {
				console.error('[EventStore] Failed to persist events:', error);
			}
		}, 300);
	});
}

// ============================================
// Initialization
// ============================================

export function initEventStore() {
	const settings = get(appSettings);
	_eventList.set(settings.eventList ?? []);
	setupPersistence();
}

// ============================================
// Event Store Operations
// ============================================

export const eventStore = {
	// ----------------------------------------
	// Event CRUD Operations
	// ----------------------------------------

	// Reload from appSettingsStore (e.g. after import)
	reloadFromStorage() {
		const settings = get(appSettings);
		_eventList.set(settings.eventList ?? []);
	},

	// Add a new event
	addEvent(event: ServiceEvent): void {
		_eventList.update(events => [...events, event]);
	},

	// Update an existing event
	updateEvent(id: string, partial: Partial<ServiceEvent>): void {
		_eventList.update(events => events.map(event =>
			event.id === id ? { ...event, ...partial, updatedAt: new Date().toISOString() } : event
		));
	},

	// Delete an event
	deleteEvent(id: string): void {
		_eventList.update(events => events.filter(event => event.id !== id));
	},

	// Get event by ID
	getEventById(id: string): ServiceEvent | undefined {
		return (get(eventList) ?? []).find((event) => event.id === id);
	},

	// Get today's event
	getTodayEvent(): ServiceEvent | null {
		return get(todayEvent);
	},

	// Get upcoming events
	getUpcomingEvents(): ServiceEvent[] {
		return get(upcomingEvents);
	},

	// Check if there's an event for today
	hasTodayEvent(): boolean {
		return get(todayEvent) !== null;
	},

	// ----------------------------------------
	// Recording Operations
	// ----------------------------------------

	// Add a recording to an event
	addRecording(eventId: string, recording: EventRecording): void {
		_eventList.update(events => events.map(event => {
			if (event.id !== eventId) return event;
			const recordings = event.recordings ?? [];
			if (recordings.some(r => r.file.path === recording.file.path)) {
				console.log(`[EventStore] Recording already exists: ${recording.file.path}`);
				return event;
			}
			console.log(`[EventStore] Added recording ${recording.id} to event ${eventId}`);
			return { ...event, recordings: [...recordings, recording], updatedAt: new Date().toISOString() };
		}));
	},

	// Mark a recording as uploaded
	markRecordingUploaded(
		eventId: string,
		recordingId: string,
		videoId: string,
		videoUrl: string
	): void {
		_eventList.update(events => events.map(event => {
			if (event.id !== eventId) return event;
			const recordings = (event.recordings ?? []).map(rec =>
				rec.id === recordingId
					? { ...rec, uploaded: true, uploadSession: undefined, uploadedAt: Date.now(), videoId, videoUrl }
					: rec
			);
			console.log(`[EventStore] Marked recording ${recordingId} as uploaded (video: ${videoId})`);
			return { ...event, recordings, youtubeUploadedId: videoId, updatedAt: new Date().toISOString() };
		}));
	},

	// Mark a recording as uploading with session data (persisted, survives refresh)
	markRecordingUploading(
		eventId: string,
		recordingId: string,
		uploadSession?: EventRecording['uploadSession']
	): void {
		_eventList.update(events => events.map(event => {
			if (event.id !== eventId) return event;
			const recordings = (event.recordings ?? []).map(rec =>
				rec.id === recordingId
					? { ...rec, ...(uploadSession !== undefined ? { uploadSession } : {}) }
					: rec
			);
			console.log(`[EventStore] Marked recording ${recordingId} as uploading`);
			return { ...event, recordings, updatedAt: new Date().toISOString() };
		}));
	},

	// Update upload progress on a recording
	updateRecordingUploadProgress(
		eventId: string,
		recordingId: string,
		bytesUploaded: number
	): void {
		_eventList.update(events => events.map(event => {
			if (event.id !== eventId) return event;
			const recordings = (event.recordings ?? []).map(rec =>
				rec.id === recordingId && rec.uploadSession
					? { ...rec, uploadSession: { ...rec.uploadSession, bytesUploaded } }
					: rec
			);
			return { ...event, recordings, updatedAt: new Date().toISOString() };
		}));
	},

	// Clear uploading state and session (on failure or cancellation)
	clearRecordingUploading(eventId: string, recordingId: string): void {
		_eventList.update(events => events.map(event => {
			if (event.id !== eventId) return event;
			const recordings = (event.recordings ?? []).map(rec =>
				rec.id === recordingId
					? { ...rec, uploadSession: undefined }
					: rec
			);
			console.log(`[EventStore] Cleared uploading state for recording ${recordingId}`);
			return { ...event, recordings, updatedAt: new Date().toISOString() };
		}));
	},

	// Update custom title for a recording
	updateRecordingTitle(eventId: string, recordingId: string, title: string): void {
		_eventList.update(events => events.map(event => {
			if (event.id !== eventId) return event;
			const recordings = (event.recordings ?? []).map(rec =>
				rec.id === recordingId ? { ...rec, customTitle: title } : rec
			);
			console.log(`[EventStore] Updated title for recording ${recordingId}`);
			return { ...event, recordings, updatedAt: new Date().toISOString() };
		}));
	},

	// Toggle whitelist for a recording
	whitelistRecording(
		eventId: string,
		recordingId: string,
		whitelisted: boolean
	): void {
		_eventList.update(events => events.map(event => {
			if (event.id !== eventId) return event;
			const recordings = (event.recordings ?? []).map(rec =>
				rec.id === recordingId ? { ...rec, whitelisted } : rec
			);
			console.log(`[EventStore] Set whitelist=${whitelisted} for recording ${recordingId}`);
			return { ...event, recordings, updatedAt: new Date().toISOString() };
		}));
	},

	// Get the next item in the upload queue
	getNextUploadQueueItem(): UploadQueueItem | null {
		const queue = get(uploadQueue);
		return queue.length > 0 ? queue[0] : null;
	},

	// ----------------------------------------
	// Session State Operations
	// ----------------------------------------

	// Push a session activity to an event's activity log
	pushSessionActivity(eventId: string, type: SessionActivityType, message?: string): void {
		const event = this.getEventById(eventId);
		if (!event) {
			console.error(`[EventStore] Cannot push activity: event ${eventId} not found`);
			return;
		}
		const activities = pushActivity(event.activities ?? [], type, message);
		this.updateEvent(eventId, { activities });
		console.log(`[EventStore] Activity ${type} pushed for event ${eventId}${message ? `: ${message}` : ''}`);
	},

	// Start a new session for an event
	startSession(eventId: string): ServiceEvent | null {
		const event = this.getEventById(eventId);
		if (!event) {
			console.error(`[EventStore] Cannot start session: event ${eventId} not found`);
			return null;
		}

		// Check if there's already an active session on another event
		const current = get(currentEvent);
		if (current && isSessionActive(current) && current.id !== eventId) {
			console.error(`[EventStore] Cannot start session: another session is already active`);
			return null;
		}

		this.pushSessionActivity(eventId, 'SESSION_STARTED');
		console.log(`[EventStore] Started session for event ${eventId}`);
		return this.getEventById(eventId) ?? null;
	},

	// End the current session (clear session state)
	endSession(eventId: string): void {
		const event = this.getEventById(eventId);
		if (!event) return;

		this.pushSessionActivity(eventId, 'SESSION_ENDED');
		this.updateEvent(eventId, { recordingDirectory: undefined });
		console.log(`[EventStore] Session ended for event ${eventId}`);
	},

	// Called when OBS connects
	onOBSConnected(eventId: string): void {
		this.pushSessionActivity(eventId, 'OBS_CONNECTED');
	},

	// Called when OBS disconnects
	onOBSDisconnected(eventId: string): void {
		this.pushSessionActivity(eventId, 'OBS_DISCONNECTED');
	},

	// Called when streaming starts
	onStreamStarted(eventId: string): void {
		this.pushSessionActivity(eventId, 'STREAM_STARTED');
	},

	// Called when streaming stops
	onStreamStopped(eventId: string): void {
		this.pushSessionActivity(eventId, 'STREAM_STOPPED');
	},

	// Called when recording starts
	onRecordStarted(eventId: string): void {
		this.pushSessionActivity(eventId, 'RECORD_STARTED');
	},

	// Called when recording stops
	onRecordStopped(eventId: string, recordingDirectory?: string): void {
		this.pushSessionActivity(eventId, 'RECORD_STOPPED', recordingDirectory);
		if (recordingDirectory) {
			this.updateEvent(eventId, { recordingDirectory });
		}
	},

	// Called when YouTube goes live
	onYouTubeLive(eventId: string): void {
		this.pushSessionActivity(eventId, 'YOUTUBE_LIVE');
	},

	// Set error on session
	setSessionError(eventId: string, error: string): void {
		this.pushSessionActivity(eventId, 'SESSION_ERROR', error);
	},

	// Transition to FINALIZING state
	setSessionFinalizing(eventId: string): void {
		this.pushSessionActivity(eventId, 'SESSION_FINALIZING');
	},

	// Transition to COMPLETED state
	setSessionCompleted(eventId: string): void {
		this.pushSessionActivity(eventId, 'SESSION_COMPLETED');
	},

	// Get current event with session
	getCurrentEvent(): ServiceEvent | null {
		return get(currentEvent);
	},

	// Get session duration in ms
	getSessionDuration(eventId: string): number {
		const event = this.getEventById(eventId);
		return event ? getSessionDuration(event) : 0;
	},

	// Finalize session (push SESSION_FINALIZED activity)
	setSessionFinalized(eventId: string): void {
		this.pushSessionActivity(eventId, 'SESSION_FINALIZED');
		console.log(`[EventStore] Session finalized for event ${eventId}`);
	},

	// Get all recording file paths across all events (for deduplication)
	getAllRecordingPaths(): Set<string> {
		const events = get(eventList) ?? [];
		const paths = new Set<string>();
		for (const event of events) {
			for (const rec of event.recordings ?? []) {
				paths.add(rec.file.path);
			}
		}
		return paths;
	},

};
