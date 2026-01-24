// Pending Uploads Store
// Provides reactive access to pending uploads stored within events

import { derived } from 'svelte/store';
import { uploadManager } from '$lib/services/upload/upload-manager';
import { allPendingUploads, type PendingUploadWithEvent } from '$lib/stores/event-store';
import type { UploadProgress } from '$lib/services/upload/upload-service.interface';
import type { UploadPlatform } from '$lib/types/upload-config';

// Re-export the derived stores from event-store for convenience
export { allPendingUploads } from '$lib/stores/event-store';
export type { PendingUploadWithEvent } from '$lib/stores/event-store';

// Derived stores for easy access
export const pendingUploads = allPendingUploads;
export const hasPendingUploads = derived(allPendingUploads, ($uploads) => $uploads.length > 0);
export const pendingUploadCount = derived(allPendingUploads, ($uploads) => $uploads.length);

// Store operations
export const pendingUploadsStore = {
	subscribe: allPendingUploads.subscribe,

	// Resume a specific upload
	async resumeUpload(
		sessionId: string,
		onProgress?: (progress: UploadProgress) => void
	): Promise<void> {
		await uploadManager.resumeUpload(sessionId, onProgress ?? (() => {}));
	},

	// Cancel an upload
	async cancelUpload(sessionId: string): Promise<void> {
		await uploadManager.cancelUpload(sessionId);
	},

	// Resume all pending uploads
	async resumeAll(
		onProgress?: (platform: UploadPlatform, progress: UploadProgress) => void
	): Promise<void> {
		await uploadManager.resumeAllPending(onProgress ?? (() => {}));
	},

	// Get all pending uploads (synchronous, from reactive store)
	getAll(): PendingUploadWithEvent[] {
		return uploadManager.getPendingUploads();
	}
};
