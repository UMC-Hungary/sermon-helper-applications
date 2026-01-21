// Session Integration Service
// Integrates event session tracking with OBS WebSocket events and post-event automation

import { get } from 'svelte/store';
import { obsWebSocket } from '$lib/utils/obs-websocket';
import { eventSessionStore, currentSession } from '$lib/stores/event-session-store';
import { uploadSettingsStore } from '$lib/stores/upload-settings-store';
import { todayEvent } from '$lib/stores/event-store';
import { systemStore } from '$lib/stores/system-store';
import { postEventAutomation } from './post-event-automation';
import { streamStatus, recordStatus } from '$lib/stores/streaming-store';
import { refreshStore } from '$lib/stores/refresh-store';
import type { OBSOutputState } from '$lib/types/obs-streaming';

class SessionIntegrationService {
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
		await eventSessionStore.init();
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

		// Subscribe to system store for YouTube status
		const systemUnsub = systemStore.subscribe((system) => {
			// We'd need to track YouTube live status from refresh-store sync
			// For now, this is handled via the refresh cycle
		});
		this.unsubscribers.push(systemUnsub);

		this.initialized = true;
		console.log('[SessionIntegration] Initialized');

		// Check for existing paused session to resume
		await this.checkForPausedSession();
	}

	// Cleanup subscriptions
	destroy(): void {
		for (const unsub of this.unsubscribers) {
			unsub();
		}
		this.unsubscribers = [];
		this.initialized = false;
	}

	// Handle OBS connection changes
	private async handleOBSConnectionChange(connected: boolean): Promise<void> {
		if (connected === this.prevOBSConnected) return;

		console.log(`[SessionIntegration] OBS connection changed: ${connected}`);

		if (connected) {
			await eventSessionStore.onOBSConnected();

			// Check if we have a paused session to resume
			const session = get(currentSession);
			if (session?.state === 'PAUSED') {
				console.log('[SessionIntegration] Resuming paused session');
				await eventSessionStore.resume();
			}
		} else {
			await eventSessionStore.onOBSDisconnected();
		}

		this.prevOBSConnected = connected;
	}

	// Handle stream state changes
	private async handleStreamStateChange(active: boolean, state: OBSOutputState): Promise<void> {
		if (active === this.prevStreamActive) return;

		console.log(`[SessionIntegration] Stream state changed: active=${active}, state=${state}`);

		if (active) {
			// Stream started
			await this.ensureSessionExists();
			await eventSessionStore.onStreamStarted();
		} else if (this.prevStreamActive) {
			// Stream stopped
			await eventSessionStore.onStreamStopped();
			await this.checkForPostEventTrigger();
		}

		this.prevStreamActive = active;
	}

	// Handle record state changes
	private async handleRecordStateChange(active: boolean, state: OBSOutputState): Promise<void> {
		if (active === this.prevRecordActive) return;

		console.log(`[SessionIntegration] Record state changed: active=${active}, state=${state}`);

		if (active) {
			// Recording started
			await this.ensureSessionExists();
			await eventSessionStore.onRecordStarted();
		} else if (this.prevRecordActive) {
			// Recording stopped - get the recording directory
			let recordingDir: string | undefined;
			try {
				recordingDir = await obsWebSocket.getRecordDirectory();
			} catch (error) {
				console.warn('[SessionIntegration] Could not get recording directory:', error);
			}

			await eventSessionStore.onRecordStopped(recordingDir);
			await this.checkForPostEventTrigger();
		}

		this.prevRecordActive = active;
	}

	// Handle today's event changes
	private async handleTodayEventChange(event: typeof todayEvent extends { subscribe: (fn: (v: infer T) => void) => void } ? T : never): Promise<void> {
		if (!event) return;

		// Check if YouTube lifecycle status changed to 'live'
		if (event.youtubeLifeCycleStatus === 'live' && !this.prevYouTubeLive) {
			console.log('[SessionIntegration] YouTube went live');
			await eventSessionStore.onYouTubeLive();
			this.prevYouTubeLive = true;
		} else if (event.youtubeLifeCycleStatus !== 'live') {
			this.prevYouTubeLive = false;
		}
	}

	// Ensure a session exists for the current event
	private async ensureSessionExists(): Promise<void> {
		const session = get(currentSession);
		if (session) return;

		const event = get(todayEvent);
		if (!event) {
			console.warn('[SessionIntegration] No event for today, cannot start session');
			return;
		}

		console.log(`[SessionIntegration] Starting session for event: ${event.id}`);
		await eventSessionStore.startSession(event.id);
	}

	// Check if post-event automation should be triggered
	private async checkForPostEventTrigger(): Promise<void> {
		// Wait a moment for state to settle
		await new Promise((resolve) => setTimeout(resolve, 1000));

		const shouldRun = await postEventAutomation.shouldRunAutomation();
		if (shouldRun) {
			console.log('[SessionIntegration] Triggering post-event automation');
			await postEventAutomation.runWorkflow();
		}
	}

	// Check for paused session on startup
	private async checkForPausedSession(): Promise<void> {
		const session = get(currentSession);
		if (!session) return;

		if (session.state === 'PAUSED') {
			console.log('[SessionIntegration] Found paused session on startup');

			// Check if OBS is connected
			const obsStatus = get(obsWebSocket.obsStatus);
			if (obsStatus.connected) {
				console.log('[SessionIntegration] OBS is connected, resuming session');
				await eventSessionStore.resume();
			}
		}
	}

	// Manually trigger post-event automation (for testing or manual trigger)
	async triggerPostEventAutomation(): Promise<void> {
		await postEventAutomation.runWorkflow();
	}

	// Get current automation state
	getAutomationState() {
		return postEventAutomation.getState();
	}

	// Check if automation is running
	isAutomationRunning(): boolean {
		return postEventAutomation.isRunning();
	}
}

// Export singleton instance
export const sessionIntegration = new SessionIntegrationService();
