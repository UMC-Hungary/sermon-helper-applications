// Uploader Integration Service
// Integrates event session tracking with OBS WebSocket events and post-event automation
// Simplified to push recordings to events on recording finished

import { get } from 'svelte/store';
import { obsWebSocket } from '$lib/utils/obs-websocket';
import { eventStore, currentEvent, todayEvent } from '$lib/stores/event-store';
import { uploadSettingsStore } from '$lib/stores/upload-settings-store';
import { streamStatus, recordStatus } from '$lib/stores/streaming-store';
import type { OBSOutputState } from '$lib/types/obs-streaming';
import { isSessionActive, createEventRecording, type RecordingFile } from '$lib/types/event';
import { toast } from '$lib/utils/toast';
import { getFileInfo } from '$lib/utils/file-info';

class UploaderIntegrationService {
	private initialized = false;
	private unsubscribers: Array<() => void> = [];

	// Previous states for change detection
	private prevOBSConnected = false;
	private prevStreamActive = false;
	private prevRecordActive = false;
	private prevYouTubeLive = false;

	// Initialize the service
	async init(): Promise<void> {
		if (this.initialized) return;

		// Initialize stores
		await uploadSettingsStore.init();

		// Subscribe to OBS connection status
		const obsStatusUnsub = obsWebSocket.obsStatus.subscribe((status) => {
			this.handleOBSConnectionChange(status.connected);
		});
		this.unsubscribers.push(obsStatusUnsub);

		// Subscribe to stream status
		const streamUnsub = streamStatus.subscribe((status) => {
			if (status.state) {
				this.handleStreamStateChange(status.active, status.state);
			}
		});
		this.unsubscribers.push(streamUnsub);

		// Subscribe to record status
		const recordUnsub = recordStatus.subscribe((status) => {
			if (status.state) {
				this.handleRecordStateChange(status.active, status.state);
			}
		});
		this.unsubscribers.push(recordUnsub);

		// Subscribe to today's event changes
		const eventUnsub = todayEvent.subscribe((event) => {
			this.handleTodayEventChange(event);
		});
		this.unsubscribers.push(eventUnsub);

		this.initialized = true;
		console.log('[UploaderIntegration] Initialized');
	}

	// Cleanup subscriptions
	destroy(): void {
		for (const unsub of this.unsubscribers) {
			unsub();
		}
		this.unsubscribers = [];
		this.initialized = false;
	}

	// Get the current event ID (from active session or today's event)
	private getCurrentEventId(): string | null {
		const current = get(currentEvent);
		if (current) return current.id;

		const today = get(todayEvent);
		return today?.id ?? null;
	}

	// Handle OBS connection changes
	private async handleOBSConnectionChange(connected: boolean): Promise<void> {
		if (connected === this.prevOBSConnected) return;

		console.log(`[UploaderIntegration] OBS connection changed: ${connected}`);

		const eventId = this.getCurrentEventId();

		if (connected) {
			if (eventId) {
				await eventStore.onOBSConnected(eventId);
			}
		} else {
			if (eventId) {
				await eventStore.onOBSDisconnected(eventId);
			}
		}

		this.prevOBSConnected = connected;
	}

	// Handle stream state changes
	private async handleStreamStateChange(active: boolean, state: OBSOutputState): Promise<void> {
		// Only process actual state transitions
		const isStarting = active && !this.prevStreamActive;
		const isStopped = !active && this.prevStreamActive && state === 'OBS_WEBSOCKET_OUTPUT_STOPPED';

		if (!isStarting && !isStopped) return;

		console.log(`[UploaderIntegration] Stream state changed: active=${active}, state=${state}`);

		if (isStarting) {
			// Stream started
			this.prevStreamActive = true;
			const eventId = await this.ensureSessionExists();
			if (eventId) {
				await eventStore.onStreamStarted(eventId);
			}
		} else if (isStopped) {
			// Stream fully stopped
			this.prevStreamActive = false;
			const eventId = this.getCurrentEventId();
			if (eventId) {
				await eventStore.onStreamStopped(eventId);
			}
		}
	}

	// Handle record state changes
	private async handleRecordStateChange(active: boolean, state: OBSOutputState): Promise<void> {
		// Only process actual state transitions
		const isStarting = active && !this.prevRecordActive;
		const isStopped = !active && this.prevRecordActive && state === 'OBS_WEBSOCKET_OUTPUT_STOPPED';

		if (!isStarting && !isStopped) return;

		console.log(`[UploaderIntegration] Record state changed: active=${active}, state=${state}`);

		if (isStarting) {
			// Recording started
			this.prevRecordActive = true;
			const eventId = await this.ensureSessionExists();
			if (eventId) {
				await eventStore.onRecordStarted(eventId);
			}
		} else if (isStopped) {
			// Recording fully stopped - get recording info and push to event
			this.prevRecordActive = false;
			await this.handleRecordingStopped();
		}
	}

	// Handle recording stopped - get file info and add to current event
	private async handleRecordingStopped(): Promise<void> {
		const eventId = this.getCurrentEventId();

		// Get recording directory
		let recordingDir: string | undefined;
		try {
			recordingDir = await obsWebSocket.getRecordDirectory();
		} catch (error) {
			console.warn('[UploaderIntegration] Could not get recording directory:', error);
		}

		if (eventId) {
			await eventStore.onRecordStopped(eventId, recordingDir);
		}

		// Try to get the recording file info
		let recordingPath: string | null = null;
		try {
			recordingPath = await obsWebSocket.getLastRecordedFilePath();
		} catch (error) {
			console.warn('[UploaderIntegration] Could not get recording file path:', error);
		}

		if (!recordingPath) {
			console.log('[UploaderIntegration] No recording path available from OBS');
			// We don't have the file path - user may need to manually select later
			return;
		}

		// Check if we have a current event to add the recording to
		if (!eventId) {
			console.warn('[UploaderIntegration] Recording finished but no current event');
			toast({
				title: 'Recording finished',
				description: 'No event selected - recording was not tracked',
				variant: 'warning'
			});
			return;
		}

		// Get file info
		let fileInfo: RecordingFile;
		try {
			fileInfo = await getFileInfo(recordingPath);
		} catch (error) {
			console.error('[UploaderIntegration] Could not get file info:', error);
			toast({
				title: 'Recording finished',
				description: 'Could not read recording file info',
				variant: 'warning'
			});
			return;
		}

		// Create and add recording to event
		const recording = createEventRecording(fileInfo);
		await eventStore.addRecording(eventId, recording);

		console.log(`[UploaderIntegration] Added recording to event: ${recording.file.name}`);
		toast({
			title: 'Recording added',
			description: `${recording.file.name} added to event`,
			variant: 'success'
		});
	}

	// Handle today's event changes
	private async handleTodayEventChange(
		event: ReturnType<typeof eventStore.getTodayEvent>
	): Promise<void> {
		if (!event) return;

		// Check if YouTube lifecycle status changed to 'live'
		if (event.youtubeLifeCycleStatus === 'live' && !this.prevYouTubeLive) {
			console.log('[UploaderIntegration] YouTube went live');
			await eventStore.onYouTubeLive(event.id);
			this.prevYouTubeLive = true;
		} else if (event.youtubeLifeCycleStatus !== 'live') {
			this.prevYouTubeLive = false;
		}
	}

	// Ensure a session exists for the current event, returns event ID
	private async ensureSessionExists(): Promise<string | null> {
		const current = get(currentEvent);
		if (current && isSessionActive(current)) {
			return current.id;
		}

		const event = get(todayEvent);
		if (!event) {
			console.warn('[UploaderIntegration] No event for today, cannot start session');
			return null;
		}

		console.log(`[UploaderIntegration] Starting session for event: ${event.id}`);
		const started = await eventStore.startSession(event.id);
		return started?.id ?? null;
	}

}

// Export singleton instance
export const uploaderIntegration = new UploaderIntegrationService();
