// Real OBS WebSocket service for sermon app
import OBSWebSocket, { OBSWebSocketError } from "obs-websocket-js";
import { writable, derived, get } from 'svelte/store';
import { obsSettingsStore } from "./obs-store";
import type { ObsDevice, ObsInputInfo } from "$lib/types/obs-devices";

export interface OBSConnectionStatus {
	connected: boolean;
	error?: string;
	lastConnected?: Date;
	loading: boolean;
	reconnecting: boolean;
}

export class LocalOBSWebSocket {
	private obs: OBSWebSocket | null = null;
	
	private status = writable<OBSConnectionStatus>({
		connected: false,
		loading: true,
		reconnecting: false,
		lastConnected: undefined,
		error: undefined
	});

	public readonly obsStatus = derived(this.status, $status => $status);

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
				.then(() => {
					console.log('OBS WebSocket connected');
					this.updateStatus(true, false, false);
					this.obs?.call('GetVersion').then(version => console.log('OBS Version:', version));
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
}

// Export singleton instance
export const obsWebSocket = new LocalOBSWebSocket();