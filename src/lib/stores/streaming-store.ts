// Streaming and Recording Status Store
// Provides derived stores for reactive UI updates

import { derived } from 'svelte/store';
import { obsWebSocket } from '$lib/utils/obs-websocket';
import { isStreamTransitioning } from '$lib/types/obs-streaming';
import type { OBSStreamStatus, OBSRecordStatus } from '$lib/types/obs-streaming';

/**
 * Stream status derived from OBS WebSocket media status
 */
export const streamStatus = derived<typeof obsWebSocket.obsMediaStatus, OBSStreamStatus>(
	obsWebSocket.obsMediaStatus,
	($media) => $media.stream
);

/**
 * Record status derived from OBS WebSocket media status
 */
export const recordStatus = derived<typeof obsWebSocket.obsMediaStatus, OBSRecordStatus>(
	obsWebSocket.obsMediaStatus,
	($media) => $media.record
);

/**
 * Whether streaming is currently active
 */
export const isStreaming = derived(streamStatus, ($stream) => $stream.active);

/**
 * Whether recording is currently active
 */
export const isRecording = derived(recordStatus, ($record) => $record.active);

/**
 * Whether stream is in a transitioning state (starting/stopping)
 */
export const isStreamTransitioningStore = derived(
	streamStatus,
	($stream) => isStreamTransitioning($stream.state)
);

/**
 * Whether record is in a transitioning state (starting/stopping)
 */
export const isRecordTransitioningStore = derived(
	recordStatus,
	($record) => isStreamTransitioning($record.state)
);

/**
 * Stream timecode (formatted HH:MM:SS)
 */
export const streamTimecode = derived(streamStatus, ($stream) => $stream.timecode);

/**
 * Record timecode (formatted HH:MM:SS)
 */
export const recordTimecode = derived(recordStatus, ($record) => $record.timecode);

/**
 * Whether recording is paused
 */
export const isRecordingPaused = derived(recordStatus, ($record) => $record.paused);

/**
 * Stream control functions
 */
export const streamControls = {
	start: () => obsWebSocket.startStream(),
	stop: () => obsWebSocket.stopStream(),
	toggle: () => obsWebSocket.toggleStream()
};

/**
 * Record control functions
 */
export const recordControls = {
	start: () => obsWebSocket.startRecord(),
	stop: () => obsWebSocket.stopRecord(),
	toggle: () => obsWebSocket.toggleRecord()
};
