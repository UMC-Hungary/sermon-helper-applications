// Simple OBS WebSocket service for sermon app
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

export class OBSWebSocket {
	private connected = false;
	private ws: WebSocket | null = null;
	private statusCallback?: (status: OBSConnectionStatus) => void;

	// Browser WebSocket for development
	private createWebSocket(): WebSocket | null {
		if (typeof window !== 'undefined') {
			const ws = new WebSocket('ws://mock-obs-websocket');
			ws.binaryType = 'arraybuffer';
			return ws;
		}
		return null;
	}

	// Connect to OBS WebSocket
	async connect(url: string, password?: string): Promise<OBSConnectionStatus> {
		console.log('OBS WebSocket connection requested:', { url, password });
		
		// Simple mock connection logic
		await new Promise(resolve => setTimeout(resolve, 1000));
		
		this.connected = url.includes('localhost');
		
		if (this.statusCallback) {
			this.statusCallback({
				connected: this.connected,
				lastConnected: new Date()
			});
		}
		
		return {
			connected: this.connected,
			lastConnected: new Date()
		};
	}

	// Disconnect from OBS WebSocket
	async disconnect(): Promise<void> {
		console.log('OBS WebSocket disconnect requested');
		this.connected = false;
		
		if (this.statusCallback) {
			this.statusCallback({
				connected: false,
				lastConnected: undefined
			});
		}
	}

	// Get available scenes (mock)
	async getScenes(): Promise<OBSScene[]> {
		console.log('Getting scenes (mock implementation)');
		return [
			{ name: 'Default Scene', index: 0 },
			{ name: 'Camera Scene', index: 1 },
			{ name: 'Slides Scene', index: 2 }
		];
	}

	// Get OBS stats (mock)
	async getStats(): Promise<OBSStats> {
		console.log('Getting OBS stats (mock implementation)');
		return {
			sceneCount: 3,
			sourceCount: 5,
			recording: false,
			streaming: false
		};
	}

	// Switch to scene (mock)
	async switchToScene(sceneName: string): Promise<void> {
		console.log('Switching to scene (mock):', sceneName);
	}

	// Update sermon text (mock)
	async updateSermonText(text: string): Promise<void> {
		console.log('Updating sermon text (mock):', text);
	}

	// Start streaming (mock)
	async startStreaming(): Promise<void> {
		console.log('Starting streaming (mock)');
	}

	// Stop streaming (mock)
	async stopStreaming(): Promise<void> {
		console.log('Stopping streaming (mock)');
	}

	// Get current connection status
	async getConnectionInfo(): Promise<{ url: string; connected: boolean }> {
		return {
			url: this.ws?.url || 'Not connected',
			connected: this.isConnected()
		};
	}

	// Check if connected
	isConnected(): boolean {
		return this.ws !== null && this.ws?.readyState === WebSocket.OPEN;
	}

	// Handle status changes
	onStatusChange(callback: (status: OBSConnectionStatus) => void): void {
		this.statusCallback = callback;
	}

	// Update internal status
	private updateStatus(status: OBSConnectionStatus): void {
		if (this.statusCallback) {
			this.statusCallback(status);
		}
	}
}

// Export singleton instance
export const obsWebSocket = new OBSWebSocket();