// Real OBS WebSocket service for sermon app
import OBSWebSocket, { OBSWebSocketError } from "obs-websocket-js";
import { writable, derived, get } from 'svelte/store';
import { obsSettingsStore } from "./obs-store";
import type { ObsDevice, ObsInputInfo } from "$lib/types/obs-devices";
import type { OBSMediaStatus, OBSOutputState } from "$lib/types/obs-streaming";
import { DEFAULT_MEDIA_STATUS, formatTimecode } from "$lib/types/obs-streaming";

// Lazy import to avoid circular dependencies
let refreshStoreModule: typeof import('$lib/stores/refresh-store') | null = null;
async function triggerYoutubeSync() {
	if (!refreshStoreModule) {
		refreshStoreModule = await import('$lib/stores/refresh-store');
	}
	refreshStoreModule.refreshStore.triggerSync();
}

export interface OBSConnectionStatus {
	connected: boolean;
	error?: string;
	lastConnected?: Date;
	loading: boolean;
	reconnecting: boolean;
}

export class LocalOBSWebSocket {
	private obs: OBSWebSocket | null = null;

	// Client-side timer for timecode updates (no polling)
	private timecodeInterval: ReturnType<typeof setInterval> | null = null;
	private streamStartTime: number | null = null;  // Timestamp when stream started
	private recordStartTime: number | null = null;  // Timestamp when record started
	private lastRecordedFilePath: string | null = null;  // Path from last RecordStateChanged event

	private status = writable<OBSConnectionStatus>({
		connected: false,
		loading: true,
		reconnecting: false,
		lastConnected: undefined,
		error: undefined
	});

	private mediaStatus = writable<OBSMediaStatus>(DEFAULT_MEDIA_STATUS);

	public readonly obsStatus = derived(this.status, $status => $status);
	public readonly obsMediaStatus = derived(this.mediaStatus, $media => $media);

	private updateStatus(connected: boolean, loading: boolean = false, reconnecting: boolean = false, error?: string): void {
		this.status.set({
			connected,
			loading,
			reconnecting,
			lastConnected: connected ? new Date() : undefined,
			error
		});
	}

	async connect(url: string, password?: string): Promise<OBSConnectionStatus> {
		this.updateStatus(false, true, false);
		
		try {
			this.obs = new OBSWebSocket();
			this.obs.connect(url, password || undefined, { rpcVersion: 1 })
				.then(async () => {
					console.log('OBS WebSocket connected');
					this.updateStatus(true, false, false);
					this.obs?.call('GetVersion').then(version => console.log('OBS Version:', version));
					// Fetch initial stream/record status
					await this.fetchInitialMediaStatus();
				})
				.catch((error: OBSWebSocketError) => {
					try { this.obs?.disconnect(); } catch {}
					console.error('Failed to connect', error?.code, error?.message || error);
					this.updateStatus(false, false, false, error?.message || 'Connection failed');
				});

			// Set up event listeners
			this.obs.on('ConnectionOpened', () => {
				console.log('OBS WebSocket connected');
				this.updateStatus(true, false, false);
			});

			this.obs.on('ConnectionClosed', (error: OBSWebSocketError) => {
				console.log('OBS WebSocket closed:', error);
				this.updateStatus(false, false, false, `Connection closed: ${error}`);
				// Stop timecode timer and reset media status on disconnect
				this.stopTimecodeTimer();
				this.streamStartTime = null;
				this.recordStartTime = null;
				this.mediaStatus.set(DEFAULT_MEDIA_STATUS);
			});

			// Stream state change events
			this.obs.on('StreamStateChanged', (data) => {
				console.log('Stream state changed:', data);
				this.handleStreamStateChange(data.outputActive, data.outputState as OBSOutputState);
			});

			// Record state change events
			this.obs.on('RecordStateChanged', (data) => {
				console.log('Record state changed:', data);
				const paused = 'outputPaused' in data ? (data as { outputPaused?: boolean }).outputPaused ?? false : false;
				// Capture outputPath when recording stops (OBS includes the file path in the event)
				const outputPath = 'outputPath' in data ? (data as { outputPath?: string }).outputPath : undefined;
				if (outputPath) {
					this.lastRecordedFilePath = outputPath;
				}
				this.handleRecordStateChange(data.outputActive, data.outputState as OBSOutputState, paused);
			});

			return {
				connected: false,
				loading: true,
				reconnecting: false
			};
		} catch (error) {
			console.error('WebSocket connection failed:', error);
			this.updateStatus(false, false, false, error instanceof Error ? error.message : 'Unknown connection error');
			
			return {
				connected: false,
				loading: false,
				reconnecting: false,
				error: error instanceof Error ? error.message : 'Unknown connection error'
			};
		}
	}

	async autoconnect(): Promise<OBSConnectionStatus> {
		this.updateStatus(false, true, false, 'Connecting...');
		
		try {
			const settings = await obsSettingsStore.getSettings();

			if (!settings.websocketUrl) {
				this.updateStatus(false, false, false, 'No WebSocket URL configured');
				return {
					connected: false,
					loading: false,
					reconnecting: false,
					error: 'No WebSocket URL configured'
				};
			}

			return await this.connect(settings.websocketUrl, settings.websocketPassword);
		} catch (error) {
			console.error('Autoconnect failed:', error);
			return {
				connected: false,
				loading: false,
				reconnecting: false,
				error: error instanceof Error ? error.message : 'Autoconnect failed'
			};
		}
	}

	async disconnect(): Promise<void> {
		// Stop timecode timer before disconnecting
		this.stopTimecodeTimer();
		this.streamStartTime = null;
		this.recordStartTime = null;

		if (this.obs) {
			try {
				await this.obs.disconnect();
				console.log('OBS WebSocket disconnected');
			} catch (error) {
				console.warn('Error disconnecting OBS WebSocket:', error);
			}
			this.obs = null;
		}
		this.updateStatus(false, false, false);
		this.mediaStatus.set(DEFAULT_MEDIA_STATUS);
	}

	/**
	 * Check if OBS is connected
	 */
	isConnected(): boolean {
		return get(this.status).connected;
	}

	/**
	 * Get list of available property items for an input source
	 * Used to get available displays (display_uuid) or audio devices (device_id)
	 */
	async getInputPropertyItems(
		inputName: string,
		propertyName: string
	): Promise<ObsDevice[]> {
		if (!this.obs || !this.isConnected()) {
			throw new Error('OBS not connected');
		}

		try {
			const result = await this.obs.call('GetInputPropertiesListPropertyItems', {
				inputName,
				propertyName
			});

			const items = result.propertyItems as Array<{ itemName: string; itemValue: string }> || [];
			return items.map((item) => ({
				itemName: item.itemName,
				itemValue: item.itemValue
			}));
		} catch (error) {
			console.error('Failed to get input property items:', error);
			throw error;
		}
	}

	/**
	 * Get current settings for an input source
	 */
	async getInputSettings(inputName: string): Promise<Record<string, unknown>> {
		if (!this.obs || !this.isConnected()) {
			throw new Error('OBS not connected');
		}

		try {
			const result = await this.obs.call('GetInputSettings', { inputName });
			return (result.inputSettings || {}) as Record<string, unknown>;
		} catch (error) {
			console.error('Failed to get input settings:', error);
			throw error;
		}
	}

	/**
	 * Set settings for an input source
	 * Used to auto-assign devices or update browser source URLs
	 */
	async setInputSettings(
		inputName: string,
		inputSettings: Record<string, unknown>
	): Promise<void> {
		if (!this.obs || !this.isConnected()) {
			throw new Error('OBS not connected');
		}

		try {
			await this.obs.call('SetInputSettings', {
				inputName,
				inputSettings: inputSettings as Record<string, string | number | boolean>
			});
			console.log('SetInputSettings success:', inputName, inputSettings);
		} catch (error) {
			console.error('Failed to set input settings:', error);
			throw error;
		}
	}

	/**
	 * Get list of all inputs, optionally filtered by kind
	 */
	async getInputList(inputKind?: string): Promise<ObsInputInfo[]> {
		if (!this.obs || !this.isConnected()) {
			throw new Error('OBS not connected');
		}

		try {
			const result = await this.obs.call('GetInputList', inputKind ? { inputKind } : undefined);
			const inputs = result.inputs as Array<{ inputName: string; inputKind: string; inputUuid?: string }> || [];
			return inputs.map((input) => ({
				inputName: input.inputName,
				inputKind: input.inputKind,
				inputUuid: input.inputUuid
			}));
		} catch (error) {
			console.error('Failed to get input list:', error);
			throw error;
		}
	}

	/**
	 * Refresh a browser source by pressing the "Refresh" button
	 * This triggers a page reload in the browser source
	 */
	async refreshBrowserSource(inputName: string): Promise<void> {
		if (!this.obs || !this.isConnected()) {
			throw new Error('OBS not connected');
		}

		try {
			await this.obs.call('PressInputPropertiesButton', {
				inputName,
				propertyName: 'refreshnocache'
			});
			console.log('Browser source refreshed:', inputName);
		} catch (error) {
			console.error('Failed to refresh browser source:', error);
			throw error;
		}
	}

	/**
	 * Fetch initial stream and record status from OBS
	 * Called after connection is established
	 */
	private async fetchInitialMediaStatus(): Promise<void> {
		if (!this.obs || !this.isConnected()) {
			return;
		}

		try {
			// Fetch stream status
			const streamStatus = await this.obs.call('GetStreamStatus');
			const streamDuration = streamStatus.outputDuration || 0;

			// If streaming is active, calculate start time from current duration
			if (streamStatus.outputActive) {
				this.streamStartTime = Date.now() - streamDuration;
			}

			this.mediaStatus.update((current) => ({
				...current,
				stream: {
					active: streamStatus.outputActive,
					reconnecting: streamStatus.outputReconnecting,
					timecode: streamStatus.outputTimecode || '00:00:00',
					duration: streamDuration,
					state: streamStatus.outputActive ? 'OBS_WEBSOCKET_OUTPUT_STARTED' : 'OBS_WEBSOCKET_OUTPUT_STOPPED'
				}
			}));
			console.log('Initial stream status:', streamStatus);
		} catch (error) {
			console.warn('Failed to get stream status (streaming may not be configured):', error);
		}

		try {
			// Fetch record status
			const recordStatus = await this.obs.call('GetRecordStatus');
			const recordDuration = recordStatus.outputDuration || 0;

			// If recording is active, calculate start time from current duration
			if (recordStatus.outputActive) {
				this.recordStartTime = Date.now() - recordDuration;
			}

			this.mediaStatus.update((current) => ({
				...current,
				record: {
					active: recordStatus.outputActive,
					paused: recordStatus.outputPaused,
					timecode: recordStatus.outputTimecode || '00:00:00',
					duration: recordDuration,
					state: recordStatus.outputActive ? 'OBS_WEBSOCKET_OUTPUT_STARTED' : 'OBS_WEBSOCKET_OUTPUT_STOPPED'
				}
			}));
			console.log('Initial record status:', recordStatus);
		} catch (error) {
			console.warn('Failed to get record status:', error);
		}

		// Start timecode timer if either is active
		this.updateTimecodeTimer();
	}

	/**
	 * Handle stream state change from WebSocket event
	 */
	private async handleStreamStateChange(active: boolean, state: OBSOutputState): Promise<void> {
		if (active && state === 'OBS_WEBSOCKET_OUTPUT_STARTED') {
			// Stream just started - fetch initial duration and set start time
			try {
				const streamStatus = await this.obs?.call('GetStreamStatus');
				const initialDuration = streamStatus?.outputDuration || 0;
				this.streamStartTime = Date.now() - initialDuration;
				console.log('Stream started, initial duration:', initialDuration);
			} catch {
				// Fallback: assume stream just started now
				this.streamStartTime = Date.now();
			}
		} else if (!active) {
			// Stream stopped
			this.streamStartTime = null;
		}

		this.mediaStatus.update((current) => ({
			...current,
			stream: {
				...current.stream,
				active,
				state,
				// Reset timecode when stopped
				timecode: active ? current.stream.timecode : '00:00:00',
				duration: active ? current.stream.duration : 0
			}
		}));

		this.updateTimecodeTimer();

		// Trigger YouTube sync when stream starts or stops
		// This ensures YouTube broadcast status is updated promptly
		if (state === 'OBS_WEBSOCKET_OUTPUT_STARTED' || state === 'OBS_WEBSOCKET_OUTPUT_STOPPED') {
			triggerYoutubeSync();
		}
	}

	/**
	 * Handle record state change from WebSocket event
	 */
	private async handleRecordStateChange(active: boolean, state: OBSOutputState, paused: boolean): Promise<void> {
		if (active && state === 'OBS_WEBSOCKET_OUTPUT_STARTED') {
			// Recording just started - fetch initial duration and set start time
			try {
				const recordStatus = await this.obs?.call('GetRecordStatus');
				const initialDuration = recordStatus?.outputDuration || 0;
				this.recordStartTime = Date.now() - initialDuration;
				console.log('Recording started, initial duration:', initialDuration);
			} catch {
				// Fallback: assume recording just started now
				this.recordStartTime = Date.now();
			}
		} else if (!active) {
			// Recording stopped
			this.recordStartTime = null;
		}

		this.mediaStatus.update((current) => ({
			...current,
			record: {
				...current.record,
				active,
				state,
				paused,
				// Reset timecode when stopped
				timecode: active ? current.record.timecode : '00:00:00',
				duration: active ? current.record.duration : 0
			}
		}));

		this.updateTimecodeTimer();
	}

	/**
	 * Update timecode timer based on current stream/record activity
	 * Starts timer when streaming or recording, stops when neither
	 */
	private updateTimecodeTimer(): void {
		const status = get(this.mediaStatus);
		const needsTimer = status.stream.active || status.record.active;

		if (needsTimer && !this.timecodeInterval) {
			this.startTimecodeTimer();
		} else if (!needsTimer && this.timecodeInterval) {
			this.stopTimecodeTimer();
		}
	}

	/**
	 * Start the client-side timecode timer
	 * Updates timecode every second based on elapsed time (no API calls)
	 */
	private startTimecodeTimer(): void {
		if (this.timecodeInterval) {
			return; // Already running
		}

		console.log('Starting client-side timecode timer');
		this.timecodeInterval = setInterval(() => {
			this.updateTimecodes();
		}, 1000);
	}

	/**
	 * Stop the client-side timecode timer
	 */
	private stopTimecodeTimer(): void {
		if (this.timecodeInterval) {
			console.log('Stopping client-side timecode timer');
			clearInterval(this.timecodeInterval);
			this.timecodeInterval = null;
		}
	}

	/**
	 * Update timecodes based on elapsed time since start
	 * Called every second by the timer (no API calls)
	 */
	private updateTimecodes(): void {
		const now = Date.now();

		this.mediaStatus.update((current) => {
			const updates: Partial<OBSMediaStatus> = {};

			// Update stream timecode if active
			if (current.stream.active && this.streamStartTime !== null) {
				const elapsed = now - this.streamStartTime;
				updates.stream = {
					...current.stream,
					duration: elapsed,
					timecode: formatTimecode(elapsed)
				};
			}

			// Update record timecode if active (and not paused)
			if (current.record.active && this.recordStartTime !== null && !current.record.paused) {
				const elapsed = now - this.recordStartTime;
				updates.record = {
					...current.record,
					duration: elapsed,
					timecode: formatTimecode(elapsed)
				};
			}

			return { ...current, ...updates };
		});
	}

	/**
	 * Start streaming
	 */
	async startStream(): Promise<void> {
		if (!this.obs || !this.isConnected()) {
			throw new Error('OBS not connected');
		}

		try {
			await this.obs.call('StartStream');
			console.log('Stream start requested');
		} catch (error) {
			console.error('Failed to start stream:', error);
			throw error;
		}

		// Give OBS time to connect, then verify it actually started
		await new Promise((r) => setTimeout(r, 5000));
		try {
			const status = await this.obs?.call('GetStreamStatus');
			if (status && !status.outputActive) {
				// OBS failed to start — force reset UI state if still transitioning
				this.handleStreamStateChange(false, 'OBS_WEBSOCKET_OUTPUT_STOPPED');
				throw new Error('Stream failed to start — check OBS for details');
			}
		} catch (error) {
			if (error instanceof Error && error.message.includes('Stream failed')) {
				throw error;
			}
			// GetStreamStatus call failed (connection lost etc.) — ignore
		}
	}

	/**
	 * Stop streaming
	 */
	async stopStream(): Promise<void> {
		if (!this.obs || !this.isConnected()) {
			throw new Error('OBS not connected');
		}

		try {
			await this.obs.call('StopStream');
			console.log('Stream stop requested');
		} catch (error) {
			console.error('Failed to stop stream:', error);
			throw error;
		}
	}

	/**
	 * Start recording
	 */
	async startRecord(): Promise<void> {
		if (!this.obs || !this.isConnected()) {
			throw new Error('OBS not connected');
		}

		try {
			await this.obs.call('StartRecord');
			console.log('Recording started');
		} catch (error) {
			console.error('Failed to start recording:', error);
			throw error;
		}
	}

	/**
	 * Stop recording
	 */
	async stopRecord(): Promise<void> {
		if (!this.obs || !this.isConnected()) {
			throw new Error('OBS not connected');
		}

		try {
			await this.obs.call('StopRecord');
			console.log('Recording stopped');
		} catch (error) {
			console.error('Failed to stop recording:', error);
			throw error;
		}
	}

	/**
	 * Toggle stream state (start if stopped, stop if started)
	 */
	async toggleStream(): Promise<void> {
		const status = get(this.mediaStatus);
		if (status.stream.active) {
			await this.stopStream();
		} else {
			await this.startStream();
		}
	}

	/**
	 * Toggle record state (start if stopped, stop if started)
	 */
	async toggleRecord(): Promise<void> {
		const status = get(this.mediaStatus);
		if (status.record.active) {
			await this.stopRecord();
		} else {
			await this.startRecord();
		}
	}

	/**
	 * Get the recording output directory from OBS
	 */
	async getRecordDirectory(): Promise<string> {
		if (!this.obs || !this.isConnected()) {
			throw new Error('OBS not connected');
		}

		try {
			const result = await this.obs.call('GetRecordDirectory');
			return result.recordDirectory;
		} catch (error) {
			console.error('Failed to get record directory:', error);
			throw error;
		}
	}

	/**
	 * Get the last recorded file path from OBS (if available)
	 * Returns the path captured from the RecordStateChanged event
	 */
	async getLastRecordedFilePath(): Promise<string | null> {
		return this.lastRecordedFilePath;
	}
}

// Export singleton instance
export const obsWebSocket = new LocalOBSWebSocket();