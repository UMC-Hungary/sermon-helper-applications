// OBS Streaming and Recording Status Types

/**
 * OBS WebSocket output state values
 * These are the actual state strings returned by OBS WebSocket events
 */
export type OBSOutputState =
	| 'OBS_WEBSOCKET_OUTPUT_STARTING'
	| 'OBS_WEBSOCKET_OUTPUT_STARTED'
	| 'OBS_WEBSOCKET_OUTPUT_STOPPING'
	| 'OBS_WEBSOCKET_OUTPUT_STOPPED';

/**
 * Stream status information
 */
export interface OBSStreamStatus {
	active: boolean;
	reconnecting: boolean;
	timecode: string;
	duration: number; // in milliseconds
	state: OBSOutputState | null;
}

/**
 * Record status information
 */
export interface OBSRecordStatus {
	active: boolean;
	paused: boolean;
	timecode: string;
	duration: number; // in milliseconds
	state: OBSOutputState | null;
}

/**
 * Combined media status for stores
 */
export interface OBSMediaStatus {
	stream: OBSStreamStatus;
	record: OBSRecordStatus;
}

/**
 * Default/initial stream status
 */
export const DEFAULT_STREAM_STATUS: OBSStreamStatus = {
	active: false,
	reconnecting: false,
	timecode: '00:00:00',
	duration: 0,
	state: null
};

/**
 * Default/initial record status
 */
export const DEFAULT_RECORD_STATUS: OBSRecordStatus = {
	active: false,
	paused: false,
	timecode: '00:00:00',
	duration: 0,
	state: null
};

/**
 * Default/initial media status
 */
export const DEFAULT_MEDIA_STATUS: OBSMediaStatus = {
	stream: DEFAULT_STREAM_STATUS,
	record: DEFAULT_RECORD_STATUS
};

/**
 * Check if a stream state indicates the stream is transitioning
 */
export function isStreamTransitioning(state: OBSOutputState | null): boolean {
	return state === 'OBS_WEBSOCKET_OUTPUT_STARTING' || state === 'OBS_WEBSOCKET_OUTPUT_STOPPING';
}

/**
 * Check if a stream state indicates the stream is active
 */
export function isStreamActive(state: OBSOutputState | null): boolean {
	return state === 'OBS_WEBSOCKET_OUTPUT_STARTED';
}

/**
 * Format duration in milliseconds to HH:MM:SS timecode
 */
export function formatTimecode(durationMs: number): string {
	const totalSeconds = Math.floor(durationMs / 1000);
	const hours = Math.floor(totalSeconds / 3600);
	const minutes = Math.floor((totalSeconds % 3600) / 60);
	const seconds = totalSeconds % 60;

	return [hours, minutes, seconds]
		.map((n) => n.toString().padStart(2, '0'))
		.join(':');
}
