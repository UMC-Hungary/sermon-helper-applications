// Event Session Store
// Manages the event lifecycle state machine for post-event automation

import { writable, derived, get } from 'svelte/store';
import { appSettingsStore, type AppSettings } from '$lib/utils/app-settings-store';
import type {
	EventSession,
	EventSessionState,
	PlatformUploadProgress,
	RecordingSelectionResult
} from '$lib/types/event-session';
import { createEventSession, isSessionActive, getSessionDuration } from '$lib/types/event-session';

// Session stored in app settings for persistence
interface SessionStorage {
	currentSession: EventSession | null;
	sessionHistory: EventSession[]; // Past sessions for reference
}

const DEFAULT_SESSION_STORAGE: SessionStorage = {
	currentSession: null,
	sessionHistory: []
};

// Internal writable store for current session
const sessionStore = writable<EventSession | null>(null);

// Loading state
const sessionLoaded = writable<boolean>(false);

// Derived stores for UI
export const currentSession = derived(sessionStore, ($session) => $session);
export const sessionState = derived(sessionStore, ($session) => $session?.state ?? 'IDLE');
export const isSessionInProgress = derived(sessionStore, ($session) =>
	$session ? isSessionActive($session) : false
);
export const uploadProgress = derived(sessionStore, ($session) => $session?.uploadProgress ?? []);

// Session store operations
class EventSessionStore {
	private initialized = false;

	// Initialize and load persisted session
	async init(): Promise<void> {
		if (this.initialized) return;

		try {
			const storage = await appSettingsStore.get('eventSession' as keyof AppSettings);
			const sessionData = (storage as SessionStorage | null) ?? DEFAULT_SESSION_STORAGE;

			if (sessionData.currentSession) {
				// Restore session, but if it was ACTIVE/PREPARING before crash, mark as PAUSED
				const session = sessionData.currentSession;
				if (session.state === 'ACTIVE' || session.state === 'PREPARING') {
					session.state = 'PAUSED';
					session.pausedAt = Date.now();
					session.pauseReason = 'App restarted during session';
				}
				sessionStore.set(session);
			}

			sessionLoaded.set(true);
			this.initialized = true;
		} catch (error) {
			console.error('[EventSessionStore] Failed to initialize:', error);
			sessionLoaded.set(true);
			this.initialized = true;
		}
	}

	// Persist current session state
	private async persist(): Promise<void> {
		const session = get(sessionStore);
		try {
			const storage = (await appSettingsStore.get(
				'eventSession' as keyof AppSettings
			)) as SessionStorage | null;
			const current = storage ?? DEFAULT_SESSION_STORAGE;

			await appSettingsStore.set('eventSession' as keyof AppSettings, {
				...current,
				currentSession: session
			} as never);
		} catch (error) {
			console.error('[EventSessionStore] Failed to persist session:', error);
		}
	}

	// Start a new session for an event
	async startSession(eventId: string): Promise<EventSession> {
		const existing = get(sessionStore);
		if (existing && isSessionActive(existing)) {
			throw new Error('A session is already in progress');
		}

		const session = createEventSession(eventId);
		session.state = 'PREPARING';

		sessionStore.set(session);
		await this.persist();

		console.log(`[EventSessionStore] Started session for event ${eventId}`);
		return session;
	}

	// End the current session
	async endSession(): Promise<void> {
		const session = get(sessionStore);
		if (!session) return;

		// Archive to history
		try {
			const storage = (await appSettingsStore.get(
				'eventSession' as keyof AppSettings
			)) as SessionStorage | null;
			const current = storage ?? DEFAULT_SESSION_STORAGE;

			// Keep only last 10 sessions
			const history = [session, ...current.sessionHistory].slice(0, 10);

			await appSettingsStore.set('eventSession' as keyof AppSettings, {
				currentSession: null,
				sessionHistory: history
			} as never);
		} catch (error) {
			console.error('[EventSessionStore] Failed to archive session:', error);
		}

		sessionStore.set(null);
		console.log('[EventSessionStore] Session ended');
	}

	// Update session state
	async updateState(state: EventSessionState): Promise<void> {
		sessionStore.update((session) => {
			if (!session) return null;

			const updated = { ...session, state };

			if (state === 'COMPLETED') {
				updated.completedAt = Date.now();
			} else if (state === 'PAUSED') {
				updated.pausedAt = Date.now();
			}

			return updated;
		});

		await this.persist();
		console.log(`[EventSessionStore] State changed to: ${state}`);
	}

	// Called when OBS connects
	async onOBSConnected(): Promise<void> {
		sessionStore.update((session) => {
			if (!session) return null;
			return { ...session, wasOBSConnected: true };
		});
		await this.persist();
	}

	// Called when OBS disconnects
	async onOBSDisconnected(): Promise<void> {
		const session = get(sessionStore);
		if (!session) return;

		// If we were active, pause the session
		if (session.state === 'ACTIVE' || session.state === 'FINALIZING') {
			await this.pause('OBS connection lost');
		}
	}

	// Called when streaming starts
	async onStreamStarted(): Promise<void> {
		sessionStore.update((session) => {
			if (!session) return null;
			return {
				...session,
				wasStreaming: true,
				streamStartedAt: session.streamStartedAt ?? Date.now(),
				state: 'ACTIVE' as EventSessionState
			};
		});
		await this.persist();
		console.log('[EventSessionStore] Stream started');
	}

	// Called when streaming stops
	async onStreamStopped(): Promise<void> {
		sessionStore.update((session) => {
			if (!session) return null;
			return {
				...session,
				streamEndedAt: Date.now()
			};
		});
		await this.persist();
		console.log('[EventSessionStore] Stream stopped');
	}

	// Called when recording starts
	async onRecordStarted(): Promise<void> {
		sessionStore.update((session) => {
			if (!session) return null;
			return {
				...session,
				wasRecording: true,
				recordStartedAt: session.recordStartedAt ?? Date.now(),
				state: 'ACTIVE' as EventSessionState
			};
		});
		await this.persist();
		console.log('[EventSessionStore] Recording started');
	}

	// Called when recording stops
	async onRecordStopped(recordingDirectory?: string): Promise<void> {
		sessionStore.update((session) => {
			if (!session) return null;
			return {
				...session,
				recordEndedAt: Date.now(),
				recordingDirectory: recordingDirectory ?? session.recordingDirectory
			};
		});
		await this.persist();
		console.log('[EventSessionStore] Recording stopped');
	}

	// Called when YouTube goes live
	async onYouTubeLive(): Promise<void> {
		sessionStore.update((session) => {
			if (!session) return null;
			return { ...session, wasYouTubeLive: true };
		});
		await this.persist();
		console.log('[EventSessionStore] YouTube went live');
	}

	// Called when YouTube broadcast completes
	async onYouTubeComplete(): Promise<void> {
		// Just log for now, this might trigger finalizing
		console.log('[EventSessionStore] YouTube broadcast complete');
	}

	// Set the selected recording file
	async setRecordingSelection(selection: RecordingSelectionResult): Promise<void> {
		sessionStore.update((session) => {
			if (!session) return null;
			return {
				...session,
				recordingSelection: selection,
				recordingFilePath: selection.selectedFile?.path
			};
		});
		await this.persist();
	}

	// Update upload progress for a platform
	async updateUploadProgress(platformProgress: PlatformUploadProgress): Promise<void> {
		sessionStore.update((session) => {
			if (!session) return null;

			const existingIndex = session.uploadProgress.findIndex(
				(p) => p.platform === platformProgress.platform
			);

			const uploadProgress =
				existingIndex >= 0
					? session.uploadProgress.map((p, i) => (i === existingIndex ? platformProgress : p))
					: [...session.uploadProgress, platformProgress];

			return { ...session, uploadProgress };
		});
		await this.persist();
	}

	// Pause the session (connection lost, etc.)
	async pause(reason: string): Promise<void> {
		sessionStore.update((session) => {
			if (!session) return null;
			return {
				...session,
				state: 'PAUSED' as EventSessionState,
				pausedAt: Date.now(),
				pauseReason: reason
			};
		});
		await this.persist();
		console.log(`[EventSessionStore] Session paused: ${reason}`);
	}

	// Resume the session after pause
	async resume(): Promise<void> {
		const session = get(sessionStore);
		if (!session || session.state !== 'PAUSED') return;

		// Determine what state to resume to
		const resumeState: EventSessionState =
			session.uploadProgress.some((p) => p.status === 'uploading') ? 'FINALIZING' : 'ACTIVE';

		sessionStore.update((s) => {
			if (!s) return null;
			return {
				...s,
				state: resumeState,
				pausedAt: undefined,
				pauseReason: undefined
			};
		});
		await this.persist();
		console.log(`[EventSessionStore] Session resumed to: ${resumeState}`);
	}

	// Set error on session
	async setError(error: string): Promise<void> {
		sessionStore.update((session) => {
			if (!session) return null;
			return { ...session, completionError: error };
		});
		await this.persist();
	}

	// Get current session
	getSession(): EventSession | null {
		return get(sessionStore);
	}

	// Get session duration in ms
	getDuration(): number {
		const session = get(sessionStore);
		return session ? getSessionDuration(session) : 0;
	}

	// Check if all outputs have stopped (for triggering post-event)
	isAllOutputsStopped(): boolean {
		const session = get(sessionStore);
		if (!session) return false;

		// We consider outputs stopped if:
		// - Recording was active but now has an end time
		// - Streaming was active but now has an end time
		const recordingStopped = session.wasRecording && !!session.recordEndedAt;
		const streamingStopped = !session.wasStreaming || !!session.streamEndedAt;

		return recordingStopped && streamingStopped;
	}

	// Check if session should trigger post-event automation
	shouldTriggerPostEvent(minDurationMinutes: number): boolean {
		const session = get(sessionStore);
		if (!session) return false;

		const minDurationMs = minDurationMinutes * 60 * 1000;
		const duration = getSessionDuration(session);

		return (
			session.wasOBSConnected &&
			session.wasRecording &&
			duration >= minDurationMs &&
			this.isAllOutputsStopped() &&
			session.state === 'ACTIVE' // Only trigger from ACTIVE state
		);
	}
}

export const eventSessionStore = new EventSessionStore();
