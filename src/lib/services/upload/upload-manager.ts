// Upload Manager
// Simplified: works directly with EventRecording

import { get } from 'svelte/store';
import { uploadSettingsStore, uploadSettings } from '$lib/stores/upload-settings-store';
import { eventStore, uploadQueue } from '$lib/stores/event-store';
import type { EventRecording, ServiceEvent } from '$lib/types/event';
import { generateCalculatedTitle } from '$lib/types/event';
import type { UploadPlatform } from '$lib/types/upload-config';
import type {
	IUploadService,
	UploadMetadata,
	UploadSession,
	UploadProgress,
	UploadResult
} from './upload-service.interface';
import { youtubeUploadService } from './youtube-upload.service';
import { generateYoutubeDescription } from '$lib/utils/youtube-helpers';

class UploadManager {
	private services: Map<UploadPlatform, IUploadService> = new Map();
	private isProcessingQueue = false;

	constructor() {
		// Register available services
		this.registerService(youtubeUploadService);
		// Future: this.registerService(facebookUploadService);
	}

	// Register a platform service
	registerService(service: IUploadService): void {
		this.services.set(service.platform, service);
		console.log(`[UploadManager] Registered service: ${service.displayName}`);
	}

	// Get a service by platform
	getService(platform: UploadPlatform): IUploadService | undefined {
		return this.services.get(platform);
	}

	// Get all registered services
	getAllServices(): IUploadService[] {
		return Array.from(this.services.values());
	}

	// Upload a single recording
	async uploadRecording(
		eventId: string,
		recording: EventRecording,
		onProgress?: (progress: UploadProgress) => void
	): Promise<UploadResult | null> {
		// Prevent duplicate uploads
		if (recording.uploadSession || recording.uploaded) {
			console.log(`[UploadManager] Upload already in progress or completed for recording: ${recording.id}`);
			return null;
		}

		const event = eventStore.getEventById(eventId);
		if (!event) {
			console.error(`[UploadManager] Event not found: ${eventId}`);
			return null;
		}

		const settings = get(uploadSettings);

		// Get enabled platforms with auto-upload
		const enabledPlatforms = settings.platforms.filter((p) => p.enabled && p.autoUpload);
		if (enabledPlatforms.length === 0) {
			console.log('[UploadManager] No platforms enabled for auto-upload');
			return null;
		}

		// For now, just upload to the first enabled platform (usually YouTube)
		const platformConfig = enabledPlatforms[0];
		const service = this.services.get(platformConfig.platform);
		if (!service) {
			console.warn(`[UploadManager] No service for platform: ${platformConfig.platform}`);
			return null;
		}

		// Check if configured
		const isConfigured = await service.isConfigured();
		if (!isConfigured) {
			console.warn(`[UploadManager] Platform not configured: ${platformConfig.platform}`);
			return null;
		}

		// Prepare metadata
		const metadata = this.prepareMetadata(event, recording, platformConfig.platform);

		let session: UploadSession | null = null;
		let lastPersistedBytes = 0;
		try {
			console.log(`[UploadManager] Starting upload to ${service.displayName}: ${recording.file.name}`);

			// Initialize upload
			session = await service.initializeUpload(recording.file.path, metadata);

			// Persist upload session on the recording for resume capability
			await eventStore.markRecordingUploading(eventId, recording.id, {
				uploadUri: session.uploadUri,
				fileSize: session.fileSize,
				bytesUploaded: 0,
				platform: session.platform,
				startedAt: session.startedAt
			});

			// Upload with progress
			const result = await service.upload(session, (progress) => {
				onProgress?.(progress);
				// Persist progress every 5MB to avoid excessive writes
				if (progress.bytesUploaded - lastPersistedBytes > 5 * 1024 * 1024) {
					lastPersistedBytes = progress.bytesUploaded;
					eventStore.updateRecordingUploadProgress(eventId, recording.id, progress.bytesUploaded);
				}
			});

			// Mark as uploaded
			await eventStore.markRecordingUploaded(eventId, recording.id, result.videoId, result.videoUrl);

			// Finalize (publish)
			try {
				await service.finalize(result);
			} catch (error) {
				console.warn(`[UploadManager] Failed to finalize upload:`, error);
				// Don't fail the whole upload for this
			}

			console.log(`[UploadManager] Upload complete: ${result.videoUrl}`);
			return result;
		} catch (error) {
			console.error(`[UploadManager] Failed to upload:`, error);
			eventStore.clearRecordingUploading(eventId, recording.id);
			throw error;
		}
	}

	// Prepare upload metadata from event and recording
	private prepareMetadata(
		event: ServiceEvent,
		recording: EventRecording,
		platform: UploadPlatform
	): UploadMetadata {
		const config = uploadSettingsStore.getPlatformConfig(platform);

		// Use custom title if set, otherwise generate from event
		const title = recording.customTitle || generateCalculatedTitle(event);

		// Get privacy setting
		const privacy = event.uploadPrivacyStatus || event.youtubePrivacyStatus || 'public';

		return {
			title,
			description: generateYoutubeDescription(event),
			privacy: privacy || 'public',
			tags: this.generateTags(event)
		};
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

	// Manually upload a single recording (bypasses autoUpload setting check)
	async uploadRecordingManual(
		eventId: string,
		recording: EventRecording,
		onProgress?: (progress: UploadProgress) => void
	): Promise<UploadResult | null> {
		// Prevent duplicate uploads
		if (recording.uploadSession || recording.uploaded) {
			console.log(`[UploadManager] Upload already in progress or completed for recording: ${recording.id}`);
			return null;
		}

		const event = eventStore.getEventById(eventId);
		if (!event) {
			console.error(`[UploadManager] Event not found: ${eventId}`);
			return null;
		}

		// Find first enabled platform (don't require autoUpload for manual trigger)
		const settings = get(uploadSettings);
		const enabledPlatforms = settings.platforms.filter((p) => p.enabled);
		if (enabledPlatforms.length === 0) {
			console.log('[UploadManager] No platforms enabled');
			return null;
		}

		const platformConfig = enabledPlatforms[0];
		const service = this.services.get(platformConfig.platform);
		if (!service) {
			console.warn(`[UploadManager] No service for platform: ${platformConfig.platform}`);
			return null;
		}

		const isConfigured = await service.isConfigured();
		if (!isConfigured) {
			console.warn(`[UploadManager] Platform not configured: ${platformConfig.platform}`);
			return null;
		}

		const metadata = this.prepareMetadata(event, recording, platformConfig.platform);

		let session: UploadSession | null = null;
		let lastPersistedBytes = 0;
		try {
			console.log(`[UploadManager] Manual upload to ${service.displayName}: ${recording.file.name}`);

			session = await service.initializeUpload(recording.file.path, metadata);

			// Persist upload session on the recording for resume capability
			await eventStore.markRecordingUploading(eventId, recording.id, {
				uploadUri: session.uploadUri,
				fileSize: session.fileSize,
				bytesUploaded: 0,
				platform: session.platform,
				startedAt: session.startedAt
			});

			const result = await service.upload(session, (progress) => {
				onProgress?.(progress);
				// Persist progress every 5MB to avoid excessive writes
				if (progress.bytesUploaded - lastPersistedBytes > 5 * 1024 * 1024) {
					lastPersistedBytes = progress.bytesUploaded;
					eventStore.updateRecordingUploadProgress(eventId, recording.id, progress.bytesUploaded);
				}
			});

			await eventStore.markRecordingUploaded(eventId, recording.id, result.videoId, result.videoUrl);

			try {
				await service.finalize(result);
			} catch (error) {
				console.warn(`[UploadManager] Failed to finalize upload:`, error);
			}

			console.log(`[UploadManager] Manual upload complete: ${result.videoUrl}`);
			return result;
		} catch (error) {
			console.error(`[UploadManager] Manual upload failed:`, error);
			throw error;
		}
	}

	// Resume an interrupted upload from persisted session data
	async resumeUploadRecording(
		eventId: string,
		recording: EventRecording,
		onProgress?: (progress: UploadProgress) => void
	): Promise<UploadResult | null> {
		const savedSession = recording.uploadSession;
		if (!savedSession) {
			console.warn(`[UploadManager] No saved session for recording ${recording.id}`);
			return null;
		}

		const service = this.services.get(savedSession.platform as UploadPlatform);
		if (!service) {
			console.warn(`[UploadManager] No service for platform: ${savedSession.platform}`);
			return null;
		}

		const isConfigured = await service.isConfigured();
		if (!isConfigured) {
			console.warn(`[UploadManager] Platform not configured: ${savedSession.platform}`);
			return null;
		}

		// Reconstruct the UploadSession from persisted data
		// Re-derive metadata from event so privacy is preserved for resumed uploads
		const event = eventStore.getEventById(eventId);
		const resumeMetadata = event
			? this.prepareMetadata(event, recording, savedSession.platform as UploadPlatform)
			: { title: '', description: '', privacy: 'unlisted', tags: [] };

		const session: UploadSession = {
			id: crypto.randomUUID(),
			platform: savedSession.platform as UploadPlatform,
			filePath: recording.file.path,
			fileSize: savedSession.fileSize,
			metadata: resumeMetadata,
			uploadUri: savedSession.uploadUri,
			bytesUploaded: savedSession.bytesUploaded,
			startedAt: savedSession.startedAt,
			status: 'uploading'
		};

		let lastPersistedBytes = savedSession.bytesUploaded;
		try {
			console.log(`[UploadManager] Resuming upload for recording ${recording.id} from ${savedSession.bytesUploaded} bytes`);

			// Ask YouTube how much was actually uploaded
			const resumedSession = await service.resume(session);

			// Update persisted progress with actual YouTube-reported position
			await eventStore.updateRecordingUploadProgress(eventId, recording.id, resumedSession.bytesUploaded);

			const result = await service.upload(resumedSession, (progress) => {
				onProgress?.(progress);
				if (progress.bytesUploaded - lastPersistedBytes > 5 * 1024 * 1024) {
					lastPersistedBytes = progress.bytesUploaded;
					eventStore.updateRecordingUploadProgress(eventId, recording.id, progress.bytesUploaded);
				}
			});

			await eventStore.markRecordingUploaded(eventId, recording.id, result.videoId, result.videoUrl);

			try {
				await service.finalize(result);
			} catch (error) {
				console.warn(`[UploadManager] Failed to finalize resumed upload:`, error);
			}

			console.log(`[UploadManager] Resumed upload complete: ${result.videoUrl}`);
			return result;
		} catch (error) {
			console.error(`[UploadManager] Resume upload failed:`, error);
			throw error;
		}
	}

	// Process the upload queue (called from post-event automation)
	async processUploadQueue(): Promise<number> {
		if (this.isProcessingQueue) {
			console.log('[UploadManager] Already processing queue');
			return 0;
		}

		this.isProcessingQueue = true;
		let uploadedCount = 0;

		try {
			const queue = get(uploadQueue);

			for (const item of queue) {
				try {
					await this.uploadRecording(item.event.id, item.recording);
					uploadedCount++;
				} catch (error) {
					console.error(`[UploadManager] Failed to upload recording ${item.recording.id}:`, error);
					// Continue with next recording
				}
			}
		} finally {
			this.isProcessingQueue = false;
		}

		return uploadedCount;
	}

}

// Export singleton instance
export const uploadManager = new UploadManager();
