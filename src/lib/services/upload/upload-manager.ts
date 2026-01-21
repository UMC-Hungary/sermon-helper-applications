// Upload Manager
// Orchestrates uploads across all configured platforms

import { get } from 'svelte/store';
import { uploadSettingsStore, uploadSettings } from '$lib/stores/upload-settings-store';
import { appSettingsStore } from '$lib/utils/app-settings-store';
import type { UploadPlatform } from '$lib/types/upload-config';
import type {
	IUploadService,
	UploadMetadata,
	UploadSession,
	UploadProgress,
	UploadResult
} from './upload-service.interface';
import { youtubeUploadService } from './youtube-upload.service';

// Pending uploads storage key
const PENDING_UPLOADS_KEY = 'pendingUploads';

interface PendingUploadsStorage {
	sessions: UploadSession[];
}

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

			try {
				console.log(`[UploadManager] Starting upload to ${service.displayName}`);

				// Prepare platform-specific metadata
				const platformMetadata = this.preparePlatformMetadata(
					metadata,
					platformConfig.platform
				);

				// Initialize upload
				const session = await service.initializeUpload(filePath, platformMetadata);
				this.activeUploads.set(session.id, session);
				await this.savePendingUpload(session);

				// Upload with progress
				const result = await service.upload(session, (progress) => {
					onProgress(platformConfig.platform, progress);
				});

				// Clean up
				this.activeUploads.delete(session.id);
				await this.removePendingUpload(session.id);

				results.set(platformConfig.platform, result);
				console.log(`[UploadManager] Upload complete to ${service.displayName}`);
			} catch (error) {
				console.error(`[UploadManager] Failed to upload to ${platformConfig.platform}:`, error);
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
		await this.savePendingUpload(session);

		try {
			const result = await service.upload(session, onProgress);
			this.activeUploads.delete(session.id);
			await this.removePendingUpload(session.id);
			return result;
		} catch (error) {
			// Keep session for resume
			session.status = 'paused';
			session.error = error instanceof Error ? error.message : String(error);
			await this.savePendingUpload(session);
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

	// Get all pending/paused uploads
	async getPendingUploads(): Promise<UploadSession[]> {
		try {
			const storage = await appSettingsStore.get(PENDING_UPLOADS_KEY as never);
			const data = (storage as PendingUploadsStorage | null) ?? { sessions: [] };
			return data.sessions.filter((s) => s.status === 'paused' || s.status === 'pending');
		} catch (error) {
			console.error('[UploadManager] Failed to get pending uploads:', error);
			return [];
		}
	}

	// Save a pending upload session
	private async savePendingUpload(session: UploadSession): Promise<void> {
		try {
			const storage = await appSettingsStore.get(PENDING_UPLOADS_KEY as never);
			const data = (storage as PendingUploadsStorage | null) ?? { sessions: [] };

			const existingIndex = data.sessions.findIndex((s) => s.id === session.id);
			if (existingIndex >= 0) {
				data.sessions[existingIndex] = session;
			} else {
				data.sessions.push(session);
			}

			await appSettingsStore.set(PENDING_UPLOADS_KEY as never, data as never);
		} catch (error) {
			console.error('[UploadManager] Failed to save pending upload:', error);
		}
	}

	// Remove a pending upload session
	private async removePendingUpload(sessionId: string): Promise<void> {
		try {
			const storage = await appSettingsStore.get(PENDING_UPLOADS_KEY as never);
			const data = (storage as PendingUploadsStorage | null) ?? { sessions: [] };

			data.sessions = data.sessions.filter((s) => s.id !== sessionId);

			await appSettingsStore.set(PENDING_UPLOADS_KEY as never, data as never);
		} catch (error) {
			console.error('[UploadManager] Failed to remove pending upload:', error);
		}
	}

	// Resume all pending uploads
	async resumeAllPending(
		onProgress: PlatformProgressCallback
	): Promise<Map<UploadPlatform, UploadResult>> {
		const pending = await this.getPendingUploads();
		const results = new Map<UploadPlatform, UploadResult>();

		for (const session of pending) {
			const service = this.services.get(session.platform);
			if (!service) continue;

			try {
				console.log(`[UploadManager] Resuming upload: ${session.id}`);

				// Resume the session
				const resumedSession = await service.resume(session);

				// Continue upload
				const result = await service.upload(resumedSession, (progress) => {
					onProgress(session.platform, progress);
				});

				await this.removePendingUpload(session.id);
				results.set(session.platform, result);
			} catch (error) {
				console.error(`[UploadManager] Failed to resume upload ${session.id}:`, error);
			}
		}

		return results;
	}

	// Cancel an active upload
	async cancelUpload(sessionId: string): Promise<void> {
		const session = this.activeUploads.get(sessionId);
		if (!session) {
			// Try to find in pending
			const pending = await this.getPendingUploads();
			const pendingSession = pending.find((s) => s.id === sessionId);
			if (pendingSession) {
				const service = this.services.get(pendingSession.platform);
				await service?.cancel(pendingSession);
				await this.removePendingUpload(sessionId);
			}
			return;
		}

		const service = this.services.get(session.platform);
		await service?.cancel(session);
		this.activeUploads.delete(sessionId);
		await this.removePendingUpload(sessionId);
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
