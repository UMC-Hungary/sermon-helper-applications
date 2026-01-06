// Real OBS WebSocket service for sermon app
import OBSWebSocket, { OBSWebSocketError } from "obs-websocket-js";

export interface OBSConnectionStatus {
	connected: boolean;
	error?: string;
	lastConnected?: Date;
}

export class LocalOBSWebSocket {
	private obs: OBSWebSocket | null = null;
	private statusCallback?: (status: OBSConnectionStatus) => void;

	// Initialize connection status
	private updateStatus(connected: boolean, error?: string): void {
		if (this.statusCallback) {
			this.statusCallback({
				connected,
				lastConnected: connected ? new Date() : undefined,
				error
			});
		}
	}

	async connect(url: string, password?: string): Promise<OBSConnectionStatus> {
		try {
			// Initialize OBS WebSocket connection
			this.obs = new OBSWebSocket();

			this.obs.connect(url, password || undefined, { rpcVersion: 1 })
				.then(() => {
					console.log('OBS WebSocket connected');
					this.updateStatus(true);
					this.obs?.call('GetVersion').then(version => console.log('OBS Version:', version));
				})
				.catch((error: OBSWebSocketError) => {
					try { this.obs?.disconnect(); } catch {}
					console.error('Failed to connect', error?.code, error?.message || error);
				});

			// Set up event listeners
			this.obs.on('ConnectionOpened', () => {
				console.log('OBS WebSocket connected');
				this.updateStatus(true);
			});

			this.obs.on('ConnectionClosed', (error: OBSWebSocketError) => {
				console.log('OBS WebSocket closed:', error);
				this.updateStatus(false, `Connection closed: ${error}`);
			});

			return {
				connected: true,
				lastConnected: new Date()
			};
		} catch (error) {
			console.error('WebSocket connection failed:', error);
			this.updateStatus(false, error instanceof Error ? error.message : 'Unknown connection error');
			
			return {
				connected: false,
				error: error instanceof Error ? error.message : 'Unknown connection error'
			};
		}
	}

	onStatusChange(callback: (status: OBSConnectionStatus) => void): void {
		this.statusCallback = callback;
	}
}

// Export singleton instance
export const obsWebSocket = new LocalOBSWebSocket();