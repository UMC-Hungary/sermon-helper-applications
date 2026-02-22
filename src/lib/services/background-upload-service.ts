// Background Upload Service
// Autonomous service that processes the upload queue independently of the finalize flow.
// Pauses when OBS is streaming, resumes when streaming stops.

import { get } from 'svelte/store';
import { uploadQueue, type UploadQueueItem } from '$lib/stores/event-store';
import { isStreaming } from '$lib/stores/streaming-store';
import { isYouTubeConnected } from '$lib/stores/youtube-store';
import { uploadManager } from './upload/upload-manager';

class BackgroundUploadService {
	private initialized = false;
	private unsubscribers: Array<() => void> = [];
	private currentUpload: Promise<void> | null = null;
	private paused = false;

	init(): void {
		if (this.initialized) return;

		// Subscribe to streaming state to pause/resume
		const streamUnsub = isStreaming.subscribe((streaming) => {
			this.paused = streaming;
			if (!streaming) {
				this.processNext();
			}
		});
		this.unsubscribers.push(streamUnsub);

		// Subscribe to upload queue changes
		const queueUnsub = uploadQueue.subscribe(() => {
			if (!this.paused) {
				this.processNext();
			}
		});
		this.unsubscribers.push(queueUnsub);

		// Subscribe to YouTube connection state — retry uploads after login
		const ytUnsub = isYouTubeConnected.subscribe((connected) => {
			if (connected && !this.paused) {
				this.processNext();
			}
		});
		this.unsubscribers.push(ytUnsub);

		this.initialized = true;
		console.log('[BackgroundUploadService] Initialized');
	}

	private processNext(): void {
		// Don't start if already uploading or paused
		if (this.currentUpload || this.paused) return;

		const queue = get(uploadQueue);
		if (queue.length === 0) return;

		// Sort by event date descending (most recent first)
		const sorted = [...queue].sort((a, b) => b.event.dateTime.localeCompare(a.event.dateTime));
		const next = sorted[0];

		this.currentUpload = this.upload(next);
	}

	private async upload(item: UploadQueueItem): Promise<void> {
		try {
			console.log(`[BackgroundUploadService] Uploading: ${item.recording.file.name}`);
			await uploadManager.uploadRecording(item.event.id, item.recording);
			console.log(`[BackgroundUploadService] Upload complete: ${item.recording.file.name}`);
		} catch (error) {
			console.error(`[BackgroundUploadService] Upload failed:`, error);
		} finally {
			this.currentUpload = null;
			// Check for next item after a short delay
			setTimeout(() => this.processNext(), 1000);
		}
	}

	destroy(): void {
		for (const unsub of this.unsubscribers) {
			unsub();
		}
		this.unsubscribers = [];
		this.initialized = false;
	}
}

export const backgroundUploadService = new BackgroundUploadService();
