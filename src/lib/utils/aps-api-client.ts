// Auto Presentation Switcher (APS) API Client for sermon helper app
// Uses APS API v2 protocol (TCP with length-prefixed JSON messages)

export interface APSCommand {
	command: string;
	parameters?: Record<string, unknown>;
}

export interface APSFeedback {
	action: string;
	data?: Record<string, unknown>;
	index?: number;
}

export interface APSSettings {
	host: string;
	port: number;
	autoConnect: boolean;
	timeout: number;
}

export interface APSConnectionStatus {
	connected: boolean;
	connecting: boolean;
	error?: string;
	lastConnected?: Date;
	apiVersion?: number;
}

export interface PresentationInfo {
	currentFile?: string;
	previousFile?: string;
	nextFile?: string;
	slideNumber?: number;
	slidesCount?: number;
	buildsCount?: number;
	powerPointMediaDuration?: string;
	powerPointMediaPosition?: string;
	powerPointMediaTimeLeft?: string;
	powerPointMediaState?: 'playing' | 'paused' | 'stopped' | 'not_ready';
}

export interface APSSlot {
	filename: string;
	exists: boolean;
	opened: boolean;
}

export const DEFAULT_APS_SETTINGS: APSSettings = {
	host: '127.0.0.1',
	port: 31600,
	autoConnect: true,
	timeout: 5000
};

export const DEFAULT_PRESENTATION_INFO: PresentationInfo = {
	currentFile: undefined,
	previousFile: undefined,
	nextFile: undefined,
	slideNumber: undefined,
	slidesCount: undefined,
	buildsCount: undefined
};

import { writable, derived, get } from 'svelte/store';

class APSClient {
	private socket: WebSocket | null = null;
	private messageBuffer: string = '';
	private expectedLength: number | null = null;
	private feedbackCallbacks: Array<(feedback: APSFeedback) => void> = [];
	private reconnectTimeout: ReturnType<typeof setTimeout> | null = null;

	private connectionStatus = writable<APSConnectionStatus>({
		connected: false,
		connecting: false,
		error: undefined,
		apiVersion: undefined
	});

	private presentationInfo = writable<PresentationInfo>(DEFAULT_PRESENTATION_INFO);
	private slots = writable<APSSlot[]>([]);
	private activeApplication = writable<string | null>(null);

	public readonly apsStatus = derived(this.connectionStatus, $status => $status);
	public readonly currentPresentation = derived(this.presentationInfo, $info => $info);
	public readonly presentationSlots = derived(this.slots, $slots => $slots);
	public readonly currentApp = derived(this.activeApplication, $app => $app);

	private updateStatus(
		connected: boolean,
		connecting: boolean = false,
		error?: string,
		apiVersion?: number
	): void {
		this.connectionStatus.set({
			connected,
			connecting,
			error,
			lastConnected: connected ? new Date() : undefined,
			apiVersion
		});
	}

	async connect(settings: APSSettings = DEFAULT_APS_SETTINGS): Promise<APSConnectionStatus> {
		if (this.socket?.readyState === WebSocket.OPEN) {
			return get(this.connectionStatus);
		}

		this.updateStatus(false, true);

		try {
			// APS uses TCP directly, but for browser/Tauri environment we'll use a bridge
			// The actual TCP connection will be handled by the Tauri backend
			// For now, we'll use WebSocket for communication with the backend
			const wsUrl = `ws://${settings.host}:${settings.port}/aps`;
			
			this.socket = new WebSocket(wsUrl);

			this.socket.onopen = () => {
				console.log('APS connection opened');
				// Send API version handshake
				this.sendCommand({
					command: 'api_version',
					api_version: 2
				});
			};

			this.socket.onmessage = (event) => {
				this.handleMessage(event.data);
			};

			this.socket.onclose = () => {
				console.log('APS connection closed');
				this.updateStatus(false, false, 'Connection closed');
				this.scheduleReconnect(settings);
			};

			this.socket.onerror = (error) => {
				console.error('APS connection error:', error);
				this.updateStatus(false, false, 'Connection error');
			};

			// Wait for connection or timeout
			await new Promise<void>((resolve, reject) => {
				const timeout = setTimeout(() => {
					reject(new Error('Connection timeout'));
				}, settings.timeout);

				const checkConnection = setInterval(() => {
					const status = get(this.connectionStatus);
					if (status.connected) {
						clearTimeout(timeout);
						clearInterval(checkConnection);
						resolve();
					} else if (status.error && !status.connecting) {
						clearTimeout(timeout);
						clearInterval(checkConnection);
						reject(new Error(status.error));
					}
				}, 100);
			});

			return get(this.connectionStatus);
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : 'Connection failed';
			this.updateStatus(false, false, errorMsg);
			return get(this.connectionStatus);
		}
	}

	disconnect(): void {
		if (this.reconnectTimeout) {
			clearTimeout(this.reconnectTimeout);
			this.reconnectTimeout = null;
		}

		if (this.socket) {
			this.socket.close();
			this.socket = null;
		}

		this.updateStatus(false, false);
	}

	private scheduleReconnect(settings: APSSettings): void {
		if (!settings.autoConnect) return;

		this.reconnectTimeout = setTimeout(() => {
			console.log('Attempting to reconnect to APS...');
			this.connect(settings);
		}, 5000);
	}

	private handleMessage(data: string): void {
		// Handle length-prefixed messages
		// APS API v2 uses 4-byte big-endian length prefix followed by JSON
		this.messageBuffer += data;

		while (this.messageBuffer.length > 0) {
			if (this.expectedLength === null) {
				// Need to read length prefix (4 bytes)
				if (this.messageBuffer.length < 4) return;
				
				// For browser environment, assume messages are JSON objects
				// In production, this would use proper binary length parsing
				this.expectedLength = this.messageBuffer.length;
			}

			if (this.messageBuffer.length >= this.expectedLength) {
				const message = this.messageBuffer.substring(0, this.expectedLength);
				this.messageBuffer = this.messageBuffer.substring(this.expectedLength);
				this.expectedLength = null;

				try {
					const feedback = JSON.parse(message) as APSFeedback;
					this.processFeedback(feedback);
				} catch (e) {
					console.error('Failed to parse APS message:', e);
				}
			} else {
				break;
			}
		}
	}

	private processFeedback(feedback: APSFeedback): void {
		console.log('APS Feedback:', feedback.action, feedback);

		switch (feedback.action) {
			case 'api_version':
				const version = feedback.data?.api_version as number;
				this.updateStatus(true, false, undefined, version);
				console.log('APS API version:', version);
				break;

			case 'files':
				this.presentationInfo.set({
					currentFile: feedback.data?.curr as string,
					previousFile: feedback.data?.prev as string,
					nextFile: feedback.data?.next as string,
					slideNumber: parseInt(feedback.data?.slide_number as string) || undefined,
					slidesCount: parseInt(feedback.data?.slides_count as string) || undefined,
					buildsCount: parseInt(feedback.data?.builds_count as string) || undefined,
					powerPointMediaDuration: feedback.data?.PowerPoint_media_duration as string,
					powerPointMediaPosition: feedback.data?.PowerPoint_media_current_position as string,
					powerPointMediaTimeLeft: feedback.data?.PowerPoint_media_time_left as string,
					powerPointMediaState: feedback.data?.PowerPoint_media_state as 'playing' | 'paused' | 'stopped' | 'not_ready'
				});
				break;

			case 'slots':
				const filenames = feedback.data?.filenames as string[];
				const exists = feedback.data?.exists as boolean[];
				const opened = feedback.data?.opened as boolean[];
				
				if (filenames && exists && opened) {
					const slots: APSSlot[] = filenames.map((filename, index) => ({
						filename,
						exists: exists[index] || false,
						opened: opened[index] || false
					}));
					this.slots.set(slots);
				}
				break;

			case 'active_application':
				this.activeApplication.set(feedback.data?.application as string || null);
				break;

			case 'any_presentation_displayed':
				// Handle presentation display status
				break;

			default:
				console.log('Unhandled APS feedback:', feedback.action);
		}

		// Notify all registered callbacks
		this.feedbackCallbacks.forEach(callback => callback(feedback));
	}

	sendCommand(command: Record<string, unknown>): void {
		if (!this.socket || this.socket.readyState !== WebSocket.OPEN) {
			console.error('APS not connected');
			return;
		}

		const message = JSON.stringify(command);
		
		// For APS API v2, prepend length in 4 bytes big-endian
		// In browser, we'll send as a simple JSON message
		// The backend should handle the proper APS protocol
		this.socket.send(message);
	}

	// Presentation Control Commands
	openPresentation(filePath: string, slideNr: number = 1, isFullscreen: boolean = true): void {
		this.sendCommand({
			command: 'OpenStart_Presentation',
			parameters: {
				file_path: filePath,
				slideNr,
				isFullscreen
			}
		});
	}

	openPresentationSlot(slot: number, slideNr: number = 1, isFullscreen: boolean = true): void {
		this.sendCommand({
			command: 'OpenStart_Presentation_Slot',
			parameters: {
				slot,
				slideNr,
				isFullscreen
			}
		});
	}

	nextSlide(): void {
		this.sendCommand({
			command: 'PowerPoint_Next'
		});
	}

	previousSlide(): void {
		this.sendCommand({
			command: 'PowerPoint_Previous'
		});
	}

	goToSlide(slideNr: number): void {
		this.sendCommand({
			command: 'PowerPoint_Go',
			parameters: {
				slideNr
			}
		});
	}

	closePresentation(): void {
		this.sendCommand({
			command: 'Key_Esc'
		});
	}

	// Navigation Commands
	nextPresentation(): void {
		this.sendCommand({
			command: 'Navigation_NextFS',
			parameters: {
				slideNr: 1
			}
		});
	}

	previousPresentation(): void {
		this.sendCommand({
			command: 'Navigation_PrevFS',
			parameters: {
				slideNr: 1
			}
		});
	}

	// Media Control Commands
	playMedia(): void {
		this.sendCommand({
			command: 'Presentation_Media_Control',
			parameters: {
				action: 'play'
			}
		});
	}

	pauseMedia(): void {
		this.sendCommand({
			command: 'Presentation_Media_Control',
			parameters: {
				action: 'pause'
			}
		});
	}

	stopMedia(): void {
		this.sendCommand({
			command: 'Presentation_Media_Control',
			parameters: {
				action: 'stop'
			}
		});
	}

	toggleMedia(): void {
		this.sendCommand({
			command: 'Presentation_Media_Control',
			parameters: {
				action: 'toggle'
			}
		});
	}

	seekMedia(direction: 'forward' | 'backward', milliseconds: number): void {
		this.sendCommand({
			command: 'Presentation_Media_Seek',
			parameters: {
				direction,
				Milliseconds: milliseconds
			}
		});
	}

	// Slot Management
	capturePresentationSlot(slot: number): void {
		this.sendCommand({
			command: 'CapturePresentationSlot',
			parameters: {
				bank_number: slot
			}
		});
	}

	setPresentationSlotPath(slot: number, filePath: string): void {
		this.sendCommand({
			command: 'SetPresentationSlotPath',
			parameters: {
				slot,
				file_path: filePath
			}
		});
	}

	// Folder Management
	setSelectedPresentationFolder(bankNumber: string | number): void {
		this.sendCommand({
			command: 'SetSelected_PresentationFolder',
			parameters: {
				bank_number: bankNumber
			}
		});
	}

	captureFolder(bankNumber: number): void {
		this.sendCommand({
			command: 'CaptureFolder',
			parameters: {
				bank_number: bankNumber
			}
		});
	}

	// Image Commands
	freezeScreen(): void {
		this.sendCommand({
			command: 'Freeze'
		});
	}

	displayTest(): void {
		this.sendCommand({
			command: 'DisplayTest'
		});
	}

	blackout(): void {
		this.sendCommand({
			command: 'Blackout'
		});
	}

	exitImages(): void {
		this.sendCommand({
			command: 'ExitImages'
		});
	}

	// Feedback Registration
	onFeedback(callback: (feedback: APSFeedback) => void): () => void {
		this.feedbackCallbacks.push(callback);
		return () => {
			const index = this.feedbackCallbacks.indexOf(callback);
			if (index > -1) {
				this.feedbackCallbacks.splice(index, 1);
			}
		};
	}

	// Request current states
	requestStates(): void {
		this.sendCommand({
			command: 'states'
		});
	}
}

// Export singleton instance
export const apsClient = new APSClient();

// Helper functions
export async function connectAPS(settings?: APSSettings): Promise<APSConnectionStatus> {
	return apsClient.connect(settings);
}

export function disconnectAPS(): void {
	apsClient.disconnect();
}

export function openPresentation(filePath: string, slideNr?: number, isFullscreen?: boolean): void {
	apsClient.openPresentation(filePath, slideNr, isFullscreen);
}

export function nextSlide(): void {
	apsClient.nextSlide();
}

export function previousSlide(): void {
	apsClient.previousSlide();
}

export function goToSlide(slideNr: number): void {
	apsClient.goToSlide(slideNr);
}
