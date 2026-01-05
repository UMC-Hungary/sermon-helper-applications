import WebSocket from '@tauri-apps/plugin-websocket';
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
	private ws: WebSocket | null = null;
	private reconnectAttempts = 0;
	private maxReconnectAttempts = 5;
	private reconnectDelay = 3000;
	private statusCallback?: (status: OBSConnectionStatus) => void;

	// Check if running in Tauri environment
	private isTauriApp = () => {
		return typeof window !== 'undefined' && 
			   typeof (window as any).__TAURI_INTERNALS__ !== 'undefined';
	};

	// Mock WebSocket for browser environment
	private createMockWebSocket(): any {
		return {
			connect: async (url: string, password?: string) => {
				console.log('Mock WebSocket connection:', { url, password });
				// Simulate connection delay
				await new Promise(resolve => setTimeout(resolve, 1500));
				return { connected: url.includes('localhost') };
			},
			
			disconnect: () => {
				console.log('Mock WebSocket disconnect');
			},
			
			getScenes: async () => {
				return [
					{ name: 'Default Scene', index: 0 },
					{ name: 'Camera Scene', index: 1 },
					{ name: 'Slides Scene', index: 2 }
				];
			},
			
			getStats: async () => {
				return {
					sceneCount: 3,
					sourceCount: 5,
					recording: false,
					streaming: false
				};
			},
			
			send: async (method: string, params?: any) => {
				console.log('Mock WebSocket send:', method, params);
				return { success: true };
			}
		};
	}

	async connect(url: string, password?: string): Promise<OBSConnectionStatus> {
		if (!this.isTauriApp()) {
			// Browser environment - use mock
			const mock = this.createMockWebSocket();
			const result = await mock.connect(url, password);
			
			this.updateStatus({
				connected: result.connected,
				lastConnected: new Date()
			});
			
			return {
				connected: result.connected,
				lastConnected: new Date()
			};
		}

		try {
			// Tauri environment - use real WebSocket
			this.ws = await WebSocket.connect(url);
			
			this.updateStatus({
				connected: true,
				lastConnected: new Date()
			});

			// Reset reconnect attempts on successful connection
			this.reconnectAttempts = 0;

			return {
				connected: true,
				lastConnected: new Date()
			};
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
		if (this.ws) {
			try {
				await this.ws.disconnect();
				this.ws = null;
				
				this.updateStatus({
					connected: false
				});
			} catch (error) {
				console.error('Failed to disconnect WebSocket:', error);
			}
		}
	}

	async getScenes(): Promise<OBSScene[]> {
		if (!this.isTauriApp()) {
			const mock = this.createMockWebSocket();
			return mock.getScenes();
		}

		if (!this.ws) {
			throw new Error('Not connected to OBS WebSocket');
		}

		try {
			// For now, return mock data since OBS WebSocket API needs specific implementation
			console.log('Getting scenes (mock implementation)');
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
		if (!this.isTauriApp()) {
			const mock = this.createMockWebSocket();
			return mock.getStats();
		}

		if (!this.ws) {
			throw new Error('Not connected to OBS WebSocket');
		}

		try {
			// For now, return mock data since OBS WebSocket API needs specific implementation
			console.log('Getting OBS stats (mock implementation)');
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
		if (!this.isTauriApp()) {
			const mock = this.createMockWebSocket();
			await mock.send('SetCurrentScene', { 'scene-name': sceneName });
			return;
		}

		if (!this.ws) {
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
		if (!this.isTauriApp()) {
			const mock = this.createMockWebSocket();
			await mock.send('SetTextSettings', { source: sourceName, text });
			return;
		}

		if (!this.ws) {
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
		if (!this.isTauriApp()) {
			console.log('Mock: Start streaming');
			return;
		}

		if (!this.ws) {
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
		if (!this.isTauriApp()) {
			console.log('Mock: Stop streaming');
			return;
		}

		if (!this.ws) {
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
		return this.ws !== null;
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