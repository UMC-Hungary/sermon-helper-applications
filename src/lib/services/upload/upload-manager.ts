// Upload Manager
// Orchestrates uploads across all configured platforms

import { get } from 'svelte/store';
import { uploadSettingsStore, uploadSettings } from '$lib/stores/upload-settings-store';
import { eventStore, allPendingUploads, type PendingUploadWithEvent } from '$lib/stores/event-store';
import type { EventUploadSession } from '$lib/types/event';
import type { UploadPlatform } from '$lib/types/upload-config';
import type {
	IUploadService,
	UploadMetadata,
	UploadSession,
	UploadProgress,
	UploadResult
} from './upload-service.interface';
import { youtubeUploadService } from './youtube-upload.service';

// Platform upload progress callback
export type PlatformProgressCallback = (platform: UploadPlatform, progress: UploadProgress) => void;

class UploadManager {
	private services: Map<UploadPlatform, IUploadService> = new Map();
	private activeUploads: Map<string, UploadSession> = new Map();

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

	// Upload to all enabled platforms
	async uploadToAllPlatforms(
		eventId: string,
		filePath: string,
		metadata: UploadMetadata,
		onProgress: PlatformProgressCallback
	): Promise<Map<UploadPlatform, UploadResult>> {
		const settings = get(uploadSettings);
		const results = new Map<UploadPlatform, UploadResult>();
		const errors: Array<{ platform: UploadPlatform; error: Error }> = [];

		// Get enabled platforms with auto-upload
		const enabledPlatforms = settings.platforms.filter((p) => p.enabled && p.autoUpload);

		if (enabledPlatforms.length === 0) {
			console.log('[UploadManager] No platforms enabled for auto-upload');
			return results;
		}

		// Upload to each platform sequentially (to avoid bandwidth contention)
		for (const platformConfig of enabledPlatforms) {
			const service = this.services.get(platformConfig.platform);
			if (!service) {
				console.warn(`[UploadManager] No service for platform: ${platformConfig.platform}`);
				continue;
			}

			// Check if configured
			const isConfigured = await service.isConfigured();
			if (!isConfigured) {
				console.warn(`[UploadManager] Platform not configured: ${platformConfig.platform}`);
				continue;
			}

				let session: UploadSession | null = null;
			try {
				console.log(`[UploadManager] Starting upload to ${service.displayName}`);

				// Prepare platform-specific metadata
				const platformMetadata = this.preparePlatformMetadata(
					metadata,
					platformConfig.platform
				);

				// Initialize upload
				session = await service.initializeUpload(filePath, platformMetadata);
				this.activeUploads.set(session.id, session);
				await this.saveUploadToEvent(eventId, session);

				// Upload with progress
				const result = await service.upload(session, (progress) => {
					onProgress(platformConfig.platform, progress);
				});

				// Clean up - update session as completed with result
				this.activeUploads.delete(session.id);
				await this.completeUploadInEvent(eventId, session.id, result);

				results.set(platformConfig.platform, result);
				console.log(`[UploadManager] Upload complete to ${service.displayName}`);
			} catch (error) {
				console.error(`[UploadManager] Failed to upload to ${platformConfig.platform}:`, error);
				// Mark session as failed in the event
				if (session) {
					const failedSession = this.toEventUploadSession(session);
					failedSession.status = 'failed';
					failedSession.error = error instanceof Error ? error.message : String(error);
					await eventStore.saveUploadSession(eventId, failedSession);
					this.activeUploads.delete(session.id);
				}
				errors.push({
					platform: platformConfig.platform,
					error: error instanceof Error ? error : new Error(String(error))
				});
			}
		}

		// If all uploads failed, throw an error
		if (errors.length > 0 && results.size === 0) {
			throw new Error(
				`All uploads failed: ${errors.map((e) => `${e.platform}: ${e.error.message}`).join(', ')}`
			);
		}

		return results;
	}

	// Upload to a specific platform
	async uploadToPlatform(
		eventId: string,
		platform: UploadPlatform,
		filePath: string,
		metadata: UploadMetadata,
		onProgress: (progress: UploadProgress) => void
	): Promise<UploadResult> {
		const service = this.services.get(platform);
		if (!service) {
			throw new Error(`No service registered for platform: ${platform}`);
		}

		const isConfigured = await service.isConfigured();
		if (!isConfigured) {
			throw new Error(`Platform not configured: ${platform}`);
		}

		const platformMetadata = this.preparePlatformMetadata(metadata, platform);
		const session = await service.initializeUpload(filePath, platformMetadata);

		this.activeUploads.set(session.id, session);
		await this.saveUploadToEvent(eventId, session);

		try {
			const result = await service.upload(session, onProgress);
			this.activeUploads.delete(session.id);
			await this.completeUploadInEvent(eventId, session.id, result);
			return result;
		} catch (error) {
			// Keep session for resume - mark as paused
			const pausedSession = this.toEventUploadSession(session);
			pausedSession.status = 'paused';
			pausedSession.error = error instanceof Error ? error.message : String(error);
			await eventStore.saveUploadSession(eventId, pausedSession);
			throw error;
		}
	}

	// Prepare platform-specific metadata
	private preparePlatformMetadata(
		metadata: UploadMetadata,
		platform: UploadPlatform
	): UploadMetadata {
		const config = uploadSettingsStore.getPlatformConfig(platform);

		// Use platform-specific default privacy if not specified
		let privacy = metadata.privacy;
		if (!privacy && config) {
			if ('defaultPrivacy' in config) {
				privacy = config.defaultPrivacy;
			}
		}

		return {
			...metadata,
			privacy: privacy || 'public'
		};
	}

	// Convert UploadSession to EventUploadSession format
	private toEventUploadSession(session: UploadSession): EventUploadSession {
		return {
			id: session.id,
			platform: session.platform,
			filePath: session.filePath,
			fileSize: session.fileSize,
			metadata: session.metadata,
			uploadUri: session.uploadUri,
			bytesUploaded: session.bytesUploaded,
			startedAt: session.startedAt,
			status: session.status,
			error: session.error
		};
	}

	// Convert EventUploadSession back to UploadSession format
	private toUploadSession(eventSession: EventUploadSession): UploadSession {
		return {
			id: eventSession.id,
			platform: eventSession.platform,
			filePath: eventSession.filePath,
			fileSize: eventSession.fileSize,
			metadata: eventSession.metadata,
			uploadUri: eventSession.uploadUri,
			bytesUploaded: eventSession.bytesUploaded,
			startedAt: eventSession.startedAt,
			status: eventSession.status,
			error: eventSession.error
		};
	}

	// Save upload session to the event
	private async saveUploadToEvent(eventId: string, session: UploadSession): Promise<void> {
		const eventUploadSession = this.toEventUploadSession(session);
		await eventStore.saveUploadSession(eventId, eventUploadSession);
	}

	// Complete an upload session and store the result
	private async completeUploadInEvent(
		eventId: string,
		sessionId: string,
		result: UploadResult
	): Promise<void> {
		const found = eventStore.getUploadSession(sessionId);
		if (found) {
			const completedSession: EventUploadSession = {
				...found.session,
				status: 'completed',
				videoId: result.videoId,
				videoUrl: result.videoUrl
			};
			await eventStore.saveUploadSession(eventId, completedSession);
		}
	}

	// Get all pending/paused uploads (from events)
	getPendingUploads(): PendingUploadWithEvent[] {
		return get(allPendingUploads);
	}

	// Resume all pending uploads
	async resumeAllPending(
		onProgress: PlatformProgressCallback
	): Promise<Map<UploadPlatform, UploadResult>> {
		const pending = this.getPendingUploads();
		const results = new Map<UploadPlatform, UploadResult>();

		for (const { event, session } of pending) {
			const service = this.services.get(session.platform);
			if (!service) continue;

			try {
				console.log(`[UploadManager] Resuming upload: ${session.id} for event: ${event.id}`);

				// Convert to UploadSession format for service
				const uploadSession = this.toUploadSession(session);

				// Resume the session
				const resumedSession = await service.resume(uploadSession);

				// Continue upload
				const result = await service.upload(resumedSession, (progress) => {
					onProgress(session.platform, progress);
				});

				// Mark completed in event
				await this.completeUploadInEvent(event.id, session.id, result);
				results.set(session.platform, result);
			} catch (error) {
				console.error(`[UploadManager] Failed to resume upload ${session.id}:`, error);
				// Mark as failed
				const failedSession: EventUploadSession = {
					...session,
					status: 'failed',
					error: error instanceof Error ? error.message : String(error)
				};
				await eventStore.saveUploadSession(event.id, failedSession);
			}
		}

		return results;
	}

	// Resume a specific upload by session ID
	async resumeUpload(
		sessionId: string,
		onProgress: (progress: UploadProgress) => void
	): Promise<UploadResult> {
		const found = eventStore.getUploadSession(sessionId);
		if (!found) {
			throw new Error(`Upload session not found: ${sessionId}`);
		}

		const { event, session } = found;
		const service = this.services.get(session.platform);
		if (!service) {
			throw new Error(`No service for platform: ${session.platform}`);
		}

		console.log(`[UploadManager] Resuming upload: ${sessionId} for event: ${event.id}`);

		// Mark as uploading
		const uploadingSession: EventUploadSession = { ...session, status: 'uploading' };
		await eventStore.saveUploadSession(event.id, uploadingSession);

		try {
			// Convert to UploadSession format for service
			const uploadSession = this.toUploadSession(session);

			// Resume the session
			const resumedSession = await service.resume(uploadSession);

			// Continue upload
			const result = await service.upload(resumedSession, onProgress);

			// Mark completed in event
			await this.completeUploadInEvent(event.id, sessionId, result);
			return result;
		} catch (error) {
			// Mark as failed
			const failedSession: EventUploadSession = {
				...session,
				status: 'failed',
				error: error instanceof Error ? error.message : String(error)
			};
			await eventStore.saveUploadSession(event.id, failedSession);
			throw error;
		}
	}

	// Cancel an active upload
	async cancelUpload(sessionId: string): Promise<void> {
		// Check active uploads first
		const activeSession = this.activeUploads.get(sessionId);
		if (activeSession) {
			const service = this.services.get(activeSession.platform);
			await service?.cancel(activeSession);
			this.activeUploads.delete(sessionId);
		}

		// Remove from event
		const found = eventStore.getUploadSession(sessionId);
		if (found) {
			const { event, session } = found;
			const service = this.services.get(session.platform);
			if (service && !activeSession) {
				// Cancel with service if not already cancelled
				await service.cancel(this.toUploadSession(session));
			}
			await eventStore.removeUploadSession(event.id, sessionId);
		}
	}

	// Finalize upload (publish, etc.)
	async finalizeUpload(platform: UploadPlatform, result: UploadResult): Promise<void> {
		const service = this.services.get(platform);
		if (service) {
			await service.finalize(result);
		}
	}
}

// Export singleton instance
export const uploadManager = new UploadManager();
