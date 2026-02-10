import { writable, derived, get, type Readable } from 'svelte/store';
import { appSettings, appSettingsStore } from '$lib/utils/app-settings-store';
import type {
	ServiceEvent,
	EventRecording,
	EventSessionState
} from '$lib/types/event';
import {
	isEventToday,
	isEventUpcoming,
	sortEventsByDate,
	isSessionActive,
	getSessionDuration,
	initSessionState,
	clearSessionState,
	isEventUploadable,
	getUploadableRecordings,
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
	return todayEvents.find((e) => e.sessionState !== 'COMPLETED') ?? null;
});

// Derived store for upcoming events (sorted, excludes completed events)
export const upcomingEvents = derived(eventList, ($eventList) => {
	const list = $eventList ?? [];
	return sortEventsByDate(list.filter((e) => isEventUpcoming(e) && e.sessionState !== 'COMPLETED'));
});

// Derived store for past events (sorted, most recent first)
// Includes events with past dates OR completed events (even from today)
export const pastEvents = derived(eventList, ($eventList) => {
	const list = $eventList ?? [];
	const today = getLocalToday();
	return sortEventsByDate(list.filter((e) => e.date < today || e.sessionState === 'COMPLETED')).reverse();
});

// ============================================
// Derived Stores - Session State
// ============================================

// Current event with active session (today's event with session OR any event with active session)
export const currentEvent = derived(eventList, ($eventList) => {
	const list = $eventList ?? [];

	// First, check for any event with an active session
	const activeSessionEvent = list.find((e) => isSessionActive(e));
	if (activeSessionEvent) return activeSessionEvent;

	// Then check for a finalizing session
	const finalizingEvent = list.find((e) => e.sessionState === 'FINALIZING');
	if (finalizingEvent) return finalizingEvent;

	// Fall back to today's first non-completed event (by time) if it has a session state
	const todayEvents = sortEventsByDate(list.filter(isEventToday));
	const today = todayEvents.find((e) => e.sessionState && e.sessionState !== 'COMPLETED');
	if (today) return today;

	return null;
});

// Current session state (derived from currentEvent)
export const sessionState = derived(currentEvent, ($event) => $event?.sessionState ?? 'IDLE');

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
		.filter(isEventUploadable)
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

		const sessionFields = initSessionState();
		this.updateEvent(eventId, sessionFields);

		console.log(`[EventStore] Started session for event ${eventId}`);
		return this.getEventById(eventId) ?? null;
	},

	// End the current session (clear session state)
	endSession(eventId: string): void {
		const event = this.getEventById(eventId);
		if (!event) return;

		const clearedFields = clearSessionState(event);
		this.updateEvent(eventId, clearedFields);

		console.log(`[EventStore] Session ended for event ${eventId}`);
	},

	// Update session state
	updateSessionState(eventId: string, state: EventSessionState): void {
		const updates: Partial<ServiceEvent> = { sessionState: state };

		if (state === 'COMPLETED') {
			updates.sessionCompletedAt = Date.now();
		}

		this.updateEvent(eventId, updates);
		console.log(`[EventStore] Session state changed to: ${state} for event ${eventId}`);
	},

	// Called when OBS connects
	onOBSConnected(eventId: string): void {
		this.updateEvent(eventId, { wasOBSConnected: true });
	},

	// Called when OBS disconnects
	onOBSDisconnected(_eventId: string): void {
		// No-op: OBS disconnect no longer pauses the session
	},

	// Called when streaming starts
	onStreamStarted(eventId: string): void {
		const event = this.getEventById(eventId);
		this.updateEvent(eventId, {
			wasStreaming: true,
			streamStartedAt: event?.streamStartedAt ?? Date.now(),
			sessionState: 'ACTIVE'
		});
		console.log(`[EventStore] Stream started for event ${eventId}`);
	},

	// Called when streaming stops
	onStreamStopped(eventId: string): void {
		this.updateEvent(eventId, { streamEndedAt: Date.now() });
		console.log(`[EventStore] Stream stopped for event ${eventId}`);
	},

	// Called when recording starts
	onRecordStarted(eventId: string): void {
		const event = this.getEventById(eventId);
		this.updateEvent(eventId, {
			wasRecording: true,
			recordStartedAt: event?.recordStartedAt ?? Date.now(),
			sessionState: 'ACTIVE'
		});
		console.log(`[EventStore] Recording started for event ${eventId}`);
	},

	// Called when recording stops
	onRecordStopped(eventId: string, recordingDirectory?: string): void {
		const event = this.getEventById(eventId);
		this.updateEvent(eventId, {
			recordEndedAt: Date.now(),
			recordingDirectory: recordingDirectory ?? event?.recordingDirectory
		});
		console.log(`[EventStore] Recording stopped for event ${eventId}`);
	},

	// Called when YouTube goes live
	onYouTubeLive(eventId: string): void {
		this.updateEvent(eventId, { wasYouTubeLive: true });
		console.log(`[EventStore] YouTube went live for event ${eventId}`);
	},

	// Set error on session
	setSessionError(eventId: string, error: string): void {
		this.updateEvent(eventId, { sessionCompletionError: error });
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

};
