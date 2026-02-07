// Post-Event Automation Service
// Simplified: detects when session is "done" and triggers upload queue processing

import { get } from 'svelte/store';
import { eventStore, currentEvent } from '$lib/stores/event-store';
import { uploadSettingsStore } from '$lib/stores/upload-settings-store';
import { youtubeApi } from '$lib/utils/youtube-api';
import { uploadManager } from './upload/upload-manager';
import { toast } from '$lib/utils/toast';
import { getUploadableRecordings, MIN_RECORDING_DURATION_SECONDS } from '$lib/types/event';
import type { EventRecording } from '$lib/types/event';

// Automation state
interface AutomationState {
	isRunning: boolean;
	currentStep: AutomationStep | null;
	error: string | null;
}

type AutomationStep = 'processing_queue' | 'completing_broadcast' | 'completed';

class PostEventAutomationService {
	private state: AutomationState = {
		isRunning: false,
		currentStep: null,
		error: null
	};

	// Check if uploads should be processed for current event
	shouldProcessUploads(): boolean {
		const event = get(currentEvent);
		if (!event) return false;

		// Check if any platform has auto-upload enabled globally
		if (!uploadSettingsStore.hasAutoUploadEnabled()) {
			console.log('[PostEventAutomation] No platforms enabled for auto-upload');
			return false;
		}

		// Check per-event auto-upload setting
		if (event.autoUploadEnabled === false) {
			console.log('[PostEventAutomation] Per-event auto-upload disabled for event:', event.id);
			return false;
		}

		return true;
	}

	// Run the post-event automation workflow
	// Now simplified: just process upload queue and complete broadcast
	async runWorkflow(): Promise<void> {
		if (this.state.isRunning) {
			console.log('[PostEventAutomation] Workflow already running');
			return;
		}

		const event = get(currentEvent);
		if (!event) {
			console.log('[PostEventAutomation] No event to process');
			return;
		}

		this.state.isRunning = true;
		this.state.error = null;

		try {
			// Update event session to FINALIZING
			await eventStore.updateSessionState(event.id, 'FINALIZING');

			// Step 1: Process upload queue (only if auto-upload is enabled)
			let uploadedCount = 0;
			if (this.shouldProcessUploads()) {
				this.state.currentStep = 'processing_queue';
				uploadedCount = await this.processUploadQueue();
			}

			// Step 2: Complete the live broadcast
			this.state.currentStep = 'completing_broadcast';
			await this.completeLiveBroadcast(event);

			// Step 3: Mark session as completed
			this.state.currentStep = 'completed';
			await eventStore.updateSessionState(event.id, 'COMPLETED');

			console.log('[PostEventAutomation] Workflow completed successfully');
			if (uploadedCount > 0) {
				toast({
					title: 'Post-event automation complete',
					description: `Uploaded ${uploadedCount} recording(s)`,
					variant: 'success'
				});
			}
		} catch (error) {
			const message = error instanceof Error ? error.message : String(error);
			this.state.error = message;
			console.error('[PostEventAutomation] Workflow failed:', error);

			const currentEventNow = get(currentEvent);
			if (currentEventNow) {
				await eventStore.setSessionError(currentEventNow.id, message);
				// Revert to ACTIVE so user can retry or continue working
				await eventStore.updateSessionState(currentEventNow.id, 'ACTIVE');
			}

			toast({
				title: 'Post-event automation failed',
				description: message,
				variant: 'error'
			});
		} finally {
			this.state.isRunning = false;
		}
	}

	// Process uploads for the current event directly (not from the uploadQueue derived store,
	// which requires COMPLETED state - but we're in FINALIZING here)
	private async processUploadQueue(): Promise<number> {
		const event = get(currentEvent);
		if (!event) {
			console.log('[PostEventAutomation] No current event');
			return 0;
		}

		// Get recordings eligible for upload from the current event only
		const uploadable = getUploadableRecordings(event);
		if (uploadable.length === 0) {
			console.log('[PostEventAutomation] No uploadable recordings for current event');
			return 0;
		}

		// Apply same auto-queue logic as uploadQueue derived store:
		// - Single recording >= MIN_RECORDING_DURATION is auto-queued
		// - Whitelisted recordings are always queued
		const autoQueued: EventRecording[] =
			uploadable.length === 1 && uploadable[0].file.duration >= MIN_RECORDING_DURATION_SECONDS
				? [uploadable[0]]
				: [];

		const userQueued: EventRecording[] = uploadable
			.filter((rec) => rec.whitelisted)
			.filter((rec) => !autoQueued.some((q) => q.id === rec.id));

		const toUpload = [...autoQueued, ...userQueued];
		if (toUpload.length === 0) {
			console.log('[PostEventAutomation] No recordings qualify for auto-upload');
			return 0;
		}

		let uploadedCount = 0;
		for (const recording of toUpload) {
			try {
				console.log(`[PostEventAutomation] Uploading recording: ${recording.file.name}`);
				await uploadManager.uploadRecording(event.id, recording);
				uploadedCount++;
			} catch (error) {
				console.error(`[PostEventAutomation] Failed to upload recording:`, error);
				// Continue with next recording
			}
		}

		return uploadedCount;
	}

	// Complete the live broadcast
	private async completeLiveBroadcast(event: { youtubeScheduledId?: string; youtubeLifeCycleStatus?: string; id: string }): Promise<void> {
		if (!event.youtubeScheduledId) {
			console.log('[PostEventAutomation] No live broadcast to complete');
			return;
		}

		// Check if already complete
		if (event.youtubeLifeCycleStatus === 'complete') {
			console.log('[PostEventAutomation] Live broadcast already complete');
			return;
		}

		try {
			await youtubeApi.endBroadcast(event.youtubeScheduledId);
			console.log('[PostEventAutomation] Live broadcast completed');

			// Update event
			await eventStore.updateEvent(event.id, {
				youtubeLifeCycleStatus: 'complete'
			});
		} catch (error) {
			console.error('[PostEventAutomation] Failed to complete live broadcast:', error);
			// Don't throw - this is not critical
		}
	}

	// Get current state
	getState(): AutomationState {
		return { ...this.state };
	}

	// Check if currently running
	isRunning(): boolean {
		return this.state.isRunning;
	}
}

// Export singleton instance
export const postEventAutomation = new PostEventAutomationService();
