import type { ObsSettings } from './obs-store';

export interface OBSConnectionStatus {
	connected: boolean;
	error?: string;
	lastConnected?: Date;
}

export interface OBSScene {
	name: string;
	index: number;
}

export interface OBSSource {
	name: string;
	type: string;
	id: string;
}

export interface OBSStats {
	sceneCount: number;
	sourceCount: number;
	recording: boolean;
	streaming: boolean;
}

class OBSWebSocketConnection {
	private obs: any = null;
	private reconnectAttempts = 0;
	private maxReconnectAttempts = 5;
	private reconnectDelay = 3000;
	private statusCallback?: (status: OBSConnectionStatus) => void;

	// Check if obs-websocket-js is available (browser environment)
	private isOBSSupported = () => {
		return typeof window !== 'undefined' && 
			   typeof (window as any).OBSWebSocket !== 'undefined';
	};

	async connect(url: string, password?: string): Promise<OBSConnectionStatus> {
		try {
			if (this.isOBSSupported()) {
				// Browser environment - use obs-websocket-js
				this.obs = new (window as any).OBSWebSocket({
					address: url,
					password: password || ''
				});

				// Set up event listeners
				this.obs.on('ConnectionOpened', () => {
					console.log('OBS Connection opened');
					this.updateStatus({
						connected: true,
						lastConnected: new Date()
					});
				});

				this.obs.on('ConnectionClosed', (data: any) => {
					console.log('OBS Connection closed:', data);
					this.updateStatus({
						connected: false,
						error: `Connection closed: ${data.code} - ${data.reason}`
					});
				});

				this.obs.on('AuthenticationSuccess', () => {
					console.log('OBS Authentication successful');
				});

				this.obs.on('AuthenticationFailure', (error: string) => {
					console.error('OBS Authentication failed:', error);
					this.updateStatus({
						connected: false,
						error: `Authentication failed: ${error}`
					});
				});

				// Connect to OBS
				await this.obs.connect();
				
				// Reset reconnect attempts on successful connection
				this.reconnectAttempts = 0;

				return {
					connected: true,
					lastConnected: new Date()
				};
			} else {
				// Fallback for non-browser environments
				console.log('Browser not available, using fallback');
				return {
					connected: false,
					error: 'Browser environment required for OBS WebSocket'
				};
			}
		} catch (error) {
			console.error('WebSocket connection failed:', error);
			
			this.updateStatus({
				connected: false,
				error: error instanceof Error ? error.message : 'Unknown connection error'
			});

			return {
				connected: false,
				error: error instanceof Error ? error.message : 'Unknown connection error'
			};
		}
	}

	async disconnect(): Promise<void> {
		if (this.obs) {
			try {
				await this.obs.disconnect();
				this.obs = null;
				
				this.updateStatus({
					connected: false
				});
			} catch (error) {
				console.error('Failed to disconnect WebSocket:', error);
			}
		}
	}

	async getScenes(): Promise<OBSScene[]> {
		if (!this.obs || !this.obs.connected) {
			throw new Error('Not connected to OBS WebSocket');
		}

		try {
			// Mock implementation for now - getting scenes requires specific OBS API calls
			console.log('Getting scenes (using mock implementation)');
			return [
				{ name: 'Default Scene', index: 0 },
				{ name: 'Camera Scene', index: 1 },
				{ name: 'Slides Scene', index: 2 }
			];
		} catch (error) {
			console.error('Failed to get scenes:', error);
			throw error;
		}
	}

	async getStats(): Promise<OBSStats> {
		if (!this.obs || !this.obs.connected) {
			throw new Error('Not connected to OBS WebSocket');
		}

		try {
			// Mock implementation for now
			console.log('Getting OBS stats (using mock implementation)');
			return {
				sceneCount: 3,
				sourceCount: 5,
				recording: false,
				streaming: false
			};
		} catch (error) {
			console.error('Failed to get OBS stats:', error);
			throw error;
		}
	}

	async switchToScene(sceneName: string): Promise<void> {
		if (!this.obs || !this.obs.connected) {
			throw new Error('Not connected to OBS WebSocket');
		}

		try {
			// Mock implementation for now
			console.log('Switching to scene:', sceneName);
		} catch (error) {
			console.error('Failed to switch scene:', error);
			throw error;
		}
	}

	async updateTextSource(sourceName: string, text: string): Promise<void> {
		if (!this.obs || !this.obs.connected) {
			throw new Error('Not connected to OBS WebSocket');
		}

		try {
			// Mock implementation for now
			console.log('Updating text source:', sourceName, 'to:', text);
		} catch (error) {
			console.error('Failed to update text source:', error);
			throw error;
		}
	}

	async startStreaming(): Promise<void> {
		if (!this.obs || !this.obs.connected) {
			throw new Error('Not connected to OBS WebSocket');
		}

		try {
			// Mock implementation for now
			console.log('Starting streaming');
		} catch (error) {
			console.error('Failed to start streaming:', error);
			throw error;
		}
	}

	async stopStreaming(): Promise<void> {
		if (!this.obs || !this.obs.connected) {
			throw new Error('Not connected to OBS WebSocket');
		}

		try {
			// Mock implementation for now
			console.log('Stopping streaming');
		} catch (error) {
			console.error('Failed to stop streaming:', error);
			throw error;
		}
	}

	isConnected(): boolean {
		return this.obs !== null && this.obs.connected;
	}

	onStatusChange(callback: (status: OBSConnectionStatus) => void): void {
		this.statusCallback = callback;
	}

	private updateStatus(status: OBSConnectionStatus): void {
		if (this.statusCallback) {
			this.statusCallback(status);
		}
	}
}

// Export singleton instance
export const obsWebSocket = new OBSWebSocketConnection();