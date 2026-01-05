// Simple OBS WebSocket service for sermon app
export interface OBSConnectionStatus {
	connected: boolean;
	error?: string;
	lastConnected?: Date;
}

export class SimpleOBSWebSocket {
	private connected = false;
	private statusCallback?: (status: OBSConnectionStatus) => void;

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
			lastConnected: this.connected ? new Date() : undefined
		};
	}

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

	async getScenes(): Promise<any[]> {
		console.log('Getting scenes');
		return [
			{ name: 'Default Scene', index: 0 },
			{ name: 'Camera Scene', index: 1 },
			{ name: 'Slides Scene', index: 2 }
		];
	}

	async getStats(): Promise<any> {
		console.log('Getting OBS stats');
		return {
			sceneCount: 3,
			sourceCount: 5,
			recording: false,
			streaming: false
		};
	}

	async switchToScene(sceneName: string): Promise<void> {
		console.log('Switching to scene:', sceneName);
	}

	async updateSermonText(text: string): Promise<void> {
		console.log('Updating sermon text:', text);
	}

	async startStreaming(): Promise<void> {
		console.log('Starting streaming');
	}

	async stopStreaming(): Promise<void> {
		console.log('Stopping streaming');
	}

	isConnected(): boolean {
		return this.connected;
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
export const obsWebSocket = new SimpleOBSWebSocket();