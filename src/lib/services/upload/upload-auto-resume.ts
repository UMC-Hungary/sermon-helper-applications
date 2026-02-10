// Auto-resume service for interrupted uploads
// Called once from layout after stores are initialized

import { derived, get } from 'svelte/store';
import { eventList, eventStore } from '$lib/stores/event-store';
import { isYouTubeConnected } from '$lib/stores/youtube-store';
import { uploadManager } from './upload-manager';
import { toast } from '$lib/utils/toast';

// Computed: first interrupted upload across all events
const interruptedUpload = derived(eventList, ($events) =>
	($events ?? [])
		.flatMap((event) => (event.recordings ?? []).map((rec) => ({ event, recording: rec })))
		.find(({ recording }) => recording.uploadSession && !recording.uploaded) ?? null
);

let attempted = false;

/**
 * Attempt to resume any interrupted uploads.
 * Call after stores are initialized (layout onMount).
 */
export function attemptAutoResume(): void {
	if (attempted) return;
	attempted = true;

	const upload = get(interruptedUpload);
	if (!upload || !get(isYouTubeConnected)) return;

	console.log(`[UploadAutoResume] Found interrupted upload: ${upload.recording.id}`);
	resumeInBackground(upload.event.id, upload.recording.id);
}

async function resumeInBackground(eventId: string, recordingId: string): Promise<void> {
	// Re-read from store to get fresh data
	const event = eventStore.getEventById(eventId);
	const recording = event?.recordings?.find((r) => r.id === recordingId);
	if (!event || !recording?.uploadSession) return;

	try {
		const result = await uploadManager.resumeUploadRecording(eventId, recording);
		if (result) {
			toast({
				title: 'Upload resumed',
				description: 'Recording uploaded successfully',
				variant: 'success'
			});
		} else {
			eventStore.clearRecordingUploading(eventId, recordingId);
		}
	} catch (error) {
		console.error('[UploadAutoResume] Resume failed:', error);
		eventStore.clearRecordingUploading(eventId, recordingId);
		toast({
			title: 'Upload resume failed',
			description: error instanceof Error ? error.message : 'Unknown error',
			variant: 'error'
		});
	}
}
