// Post-Event Automation Service
// Orchestrates the post-event workflow: recording selection, upload, publish

import { get } from 'svelte/store';
import { eventStore, todayEvent } from '$lib/stores/event-store';
import { eventSessionStore, currentSession } from '$lib/stores/event-session-store';
import { uploadSettingsStore } from '$lib/stores/upload-settings-store';
import { obsWebSocket } from '$lib/utils/obs-websocket';
import { youtubeApi } from '$lib/utils/youtube-api';
import { generateYoutubeDescription } from '$lib/utils/youtube-helpers';
import { generateCalculatedTitle, type ServiceEvent } from '$lib/types/event';
import { selectRecording } from '$lib/utils/recording-file-selector';
import { uploadManager } from './upload/upload-manager';
import type { UploadMetadata, UploadResult } from './upload/upload-service.interface';
import type { UploadPlatform } from '$lib/types/upload-config';
import type { RecordingFile, PlatformUploadProgress } from '$lib/types/event-session';
import { toast } from '$lib/utils/toast';

// Automation state
interface AutomationState {
	isRunning: boolean;
	currentStep: AutomationStep | null;
	error: string | null;
}

type AutomationStep =
	| 'detecting_recording'
	| 'selecting_recording'
	| 'uploading'
	| 'publishing'
	| 'completing_broadcast'
	| 'completed';

class PostEventAutomationService {
	private state: AutomationState = {
		isRunning: false,
		currentStep: null,
		error: null
	};

	// Callbacks for UI updates
	private onRecordingSelectionRequired:
		| ((candidates: RecordingFile[]) => Promise<RecordingFile | null>)
		| null = null;

	// Set callback for when user needs to select a recording
	setRecordingSelectionCallback(
		callback: (candidates: RecordingFile[]) => Promise<RecordingFile | null>
	): void {
		this.onRecordingSelectionRequired = callback;
	}

	// Check if automation should run for current session
	async shouldRunAutomation(): Promise<boolean> {
		const session = get(currentSession);
		if (!session) return false;

		const settings = uploadSettingsStore.getSettings();

		// Check if any platform has auto-upload enabled
		if (!uploadSettingsStore.hasAutoUploadEnabled()) {
			console.log('[PostEventAutomation] No platforms enabled for auto-upload');
			return false;
		}

		// Use the session store's check
		return eventSessionStore.shouldTriggerPostEvent(settings.minDurationMinutes);
	}

	// Run the post-event automation workflow
	async runWorkflow(): Promise<void> {
		if (this.state.isRunning) {
			console.log('[PostEventAutomation] Workflow already running');
			return;
		}

		const session = get(currentSession);
		if (!session) {
			console.log('[PostEventAutomation] No session to process');
			return;
		}

		// Get the event
		const event = eventStore.getEventById(session.eventId);
		if (!event) {
			console.error('[PostEventAutomation] Event not found:', session.eventId);
			return;
		}

		this.state.isRunning = true;
		this.state.error = null;

		try {
			// Update session to FINALIZING
			await eventSessionStore.updateState('FINALIZING');

			// Step 1: Detect and select recording
			this.state.currentStep = 'detecting_recording';
			const recordingFile = await this.selectRecordingFile(session, event);

			if (!recordingFile) {
				throw new Error('No suitable recording file found');
			}

			// Step 2: Upload to all platforms
			this.state.currentStep = 'uploading';
			const uploadResults = await this.uploadToAllPlatforms(recordingFile, event);

			// Step 3: Finalize (publish videos)
			this.state.currentStep = 'publishing';
			await this.finalizeUploads(uploadResults, event);

			// Step 4: Complete the live broadcast
			this.state.currentStep = 'completing_broadcast';
			await this.completeLiveBroadcast(event);

			// Step 5: Mark session as completed
			this.state.currentStep = 'completed';
			await eventSessionStore.updateState('COMPLETED');

			console.log('[PostEventAutomation] Workflow completed successfully');
			toast({
				title: 'Post-event automation complete',
				description: `Recording uploaded to ${uploadResults.size} platform(s)`,
				variant: 'success'
			});
		} catch (error) {
			const message = error instanceof Error ? error.message : String(error);
			this.state.error = message;
			console.error('[PostEventAutomation] Workflow failed:', error);

			await eventSessionStore.setError(message);

			toast({
				title: 'Post-event automation failed',
				description: message,
				variant: 'error'
			});
		} finally {
			this.state.isRunning = false;
		}
	}

	// Select the recording file (auto or manual)
	private async selectRecordingFile(
		session: typeof currentSession extends { subscribe: (fn: (v: infer T) => void) => void }
			? NonNullable<T>
			: never,
		event: ServiceEvent
	): Promise<RecordingFile | null> {
		// Get recording directory from session or query OBS
		let recordingDir = session.recordingDirectory;

		if (!recordingDir) {
			try {
				recordingDir = await obsWebSocket.getRecordDirectory();
			} catch (error) {
				console.error('[PostEventAutomation] Failed to get recording directory:', error);
				throw new Error('Could not determine recording directory');
			}
		}

		// Get settings
		const settings = uploadSettingsStore.getSettings();

		// Select recording
		const result = await selectRecording(
			recordingDir,
			session.sessionStartedAt,
			session.recordEndedAt || Date.now(),
			{
				shortVideoThresholdSeconds: settings.shortVideoThresholdMinutes * 60,
				minUploadDurationSeconds: settings.minDurationMinutes * 60
			}
		);

		// Save selection to session
		await eventSessionStore.setRecordingSelection(result);

		if (result.autoSelected && result.selectedFile) {
			console.log(`[PostEventAutomation] Auto-selected recording: ${result.selectedFile.name}`);
			return result.selectedFile;
		}

		if (result.reason === 'multiple_long' && result.candidates.length > 0) {
			// Need user selection
			this.state.currentStep = 'selecting_recording';

			if (this.onRecordingSelectionRequired) {
				const selected = await this.onRecordingSelectionRequired(result.candidates);
				if (selected) {
					// Update session with user selection
					await eventSessionStore.setRecordingSelection({
						...result,
						autoSelected: false,
						selectedFile: selected,
						reason: 'user_selected'
					});
					return selected;
				}
			}

			// No callback or user cancelled - use the most recent
			console.log('[PostEventAutomation] Using most recent recording (no user selection)');
			return result.candidates[0];
		}

		if (result.reason === 'no_long') {
			console.log('[PostEventAutomation] No long recordings found');
			return null;
		}

		return result.selectedFile;
	}

	// Upload to all enabled platforms
	private async uploadToAllPlatforms(
		recordingFile: RecordingFile,
		event: ServiceEvent
	): Promise<Map<UploadPlatform, UploadResult>> {
		// Prepare metadata
		const metadata: UploadMetadata = {
			title: generateCalculatedTitle(event),
			description: generateYoutubeDescription(event),
			privacy: event.youtubePrivacyStatus || 'public',
			tags: this.generateTags(event)
		};

		// Upload with progress tracking
		const results = await uploadManager.uploadToAllPlatforms(
			event.id,
			recordingFile.path,
			metadata,
			(platform, progress) => {
				// Update session with progress
				const platformProgress: PlatformUploadProgress = {
					platform,
					name: this.getPlatformDisplayName(platform),
					status: 'uploading',
					bytesUploaded: progress.bytesUploaded,
					totalBytes: progress.totalBytes,
					percentage: progress.percentage
				};
				eventSessionStore.updateUploadProgress(platformProgress);
			}
		);

		// Update event with upload results
		for (const [platform, result] of results) {
			if (platform === 'youtube') {
				await eventStore.updateEvent(event.id, {
					youtubeUploadedId: result.videoId,
					videoUploadState: 'completed'
				});
			}
			// Future: handle other platforms
		}

		return results;
	}

	// Finalize uploads (publish)
	private async finalizeUploads(
		results: Map<UploadPlatform, UploadResult>,
		event: ServiceEvent
	): Promise<void> {
		for (const [platform, result] of results) {
			try {
				await uploadManager.finalizeUpload(platform, result);

				// Update progress
				const platformProgress: PlatformUploadProgress = {
					platform,
					name: this.getPlatformDisplayName(platform),
					status: 'completed',
					bytesUploaded: 0,
					totalBytes: 0,
					percentage: 100,
					videoId: result.videoId,
					videoUrl: result.videoUrl
				};
				await eventSessionStore.updateUploadProgress(platformProgress);
			} catch (error) {
				console.error(`[PostEventAutomation] Failed to finalize ${platform}:`, error);
				// Don't throw - continue with other platforms
			}
		}
	}

	// Complete the live broadcast
	private async completeLiveBroadcast(event: ServiceEvent): Promise<void> {
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

	// Generate tags for the video
	private generateTags(event: ServiceEvent): string[] {
		const tags: string[] = [];

		if (event.speaker) tags.push(event.speaker);
		if (event.title) tags.push(event.title);

		// Add some default tags
		tags.push('sermon', 'church', 'worship');

		return tags.filter(Boolean);
	}

	// Get display name for platform
	private getPlatformDisplayName(platform: UploadPlatform): string {
		switch (platform) {
			case 'youtube':
				return 'YouTube';
			case 'facebook':
				return 'Facebook';
			case 'custom':
				return 'Custom';
			default:
				return platform;
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
