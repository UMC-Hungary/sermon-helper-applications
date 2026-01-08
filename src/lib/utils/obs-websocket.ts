// Real OBS WebSocket service for sermon app
import OBSWebSocket, { OBSWebSocketError } from "obs-websocket-js";
import { writable, derived } from 'svelte/store';
import { obsSettingsStore } from "./obs-store";

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
}

// Export singleton instance
export const obsWebSocket = new LocalOBSWebSocket();