// YouTube Upload Service
// Implements resumable upload to YouTube via Rust backend

import { get } from 'svelte/store';

// Check if running in Tauri
function isTauri(): boolean {
	return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

// Dynamic imports for Tauri API
async function invokeCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
	if (!isTauri()) {
		throw new Error('Video upload is only available in the desktop app');
	}
	const { invoke } = await import('@tauri-apps/api/core');
	return invoke<T>(command, args);
}

async function listenToEvent<T>(
	event: string,
	handler: (event: { payload: T }) => void
): Promise<() => void> {
	if (!isTauri()) {
		return () => {}; // No-op unlisten
	}
	const { listen } = await import('@tauri-apps/api/event');
	return listen<T>(event, handler);
}

type UnlistenFn = () => void;
import { youtubeTokens } from '$lib/stores/youtube-store';
import { youtubeApi } from '$lib/utils/youtube-api';
import { uploadSettingsStore } from '$lib/stores/upload-settings-store';
import type {
	IUploadService,
	UploadMetadata,
	UploadSession,
	UploadProgress,
	UploadResult
} from './upload-service.interface';
import { createUploadSession } from './upload-service.interface';

interface VideoFileInfo {
	path: string;
	size: number;
	exists: boolean;
}

interface UploadChunkResult {
	bytes_uploaded: number;
	total_bytes: number;
	completed: boolean;
	video_id: string | null;
}

interface ProgressEvent {
	bytesUploaded: number;
	totalBytes: number;
	percentage: number;
}

export class YouTubeUploadService implements IUploadService {
	readonly platform = 'youtube' as const;
	readonly displayName = 'YouTube';

	private progressUnlisten: UnlistenFn | null = null;

	// Check if YouTube is configured and user is logged in
	async isConfigured(): Promise<boolean> {
		const tokens = get(youtubeTokens);
		return tokens !== null && !!tokens.accessToken;
	}

	// Initialize a resumable upload session
	async initializeUpload(filePath: string, metadata: UploadMetadata): Promise<UploadSession> {
		// Get valid access token
		const accessToken = await youtubeApi.getValidAccessToken();

		// Get file info
		const fileInfo = await invokeCommand<VideoFileInfo>('get_video_file_info', { path: filePath });
		if (!fileInfo.exists) {
			throw new Error(`Video file does not exist: ${filePath}`);
		}

		// Initialize upload via Rust backend
		const uploadUri = await invokeCommand<string>('init_youtube_upload', {
			accessToken,
			filePath,
			title: metadata.title.substring(0, 100), // YouTube title limit
			description: metadata.description.substring(0, 5000), // YouTube description limit
			privacyStatus: metadata.privacy
		});

		console.log(`[YouTubeUpload] Initialized upload, URI: ${uploadUri}`);

		return createUploadSession(this.platform, filePath, fileInfo.size, metadata, uploadUri);
	}

	// Upload file with progress
	async upload(
		session: UploadSession,
		onProgress: (progress: UploadProgress) => void
	): Promise<UploadResult> {
		const settings = uploadSettingsStore.getSettings();
		const chunkSize = settings.chunkSizeMB * 1024 * 1024;

		// Set up progress listener
		this.progressUnlisten = await listenToEvent<ProgressEvent>('upload-progress', (event) => {
			onProgress({
				bytesUploaded: event.payload.bytesUploaded,
				totalBytes: event.payload.totalBytes,
				percentage: event.payload.percentage
			});
		});

		try {
			let bytesUploaded = session.bytesUploaded;
			const totalBytes = session.fileSize;

			// Upload in chunks
			while (bytesUploaded < totalBytes) {
				const result = await invokeCommand<UploadChunkResult>('upload_video_chunk', {
					uploadUri: session.uploadUri,
					filePath: session.filePath,
					startByte: bytesUploaded,
					chunkSize: chunkSize
				});

				bytesUploaded = result.bytes_uploaded;

				if (result.completed && result.video_id) {
					console.log(`[YouTubeUpload] Upload complete, video ID: ${result.video_id}`);

					return {
						platform: this.platform,
						videoId: result.video_id,
						videoUrl: `https://www.youtube.com/watch?v=${result.video_id}`,
						processingStatus: 'processing', // YouTube needs time to process
						privacy: session.metadata.privacy
					};
				}
			}

			throw new Error('Upload completed but no video ID received');
		} finally {
			// Clean up listener
			if (this.progressUnlisten) {
				this.progressUnlisten();
				this.progressUnlisten = null;
			}
		}
	}

	// Resume an interrupted upload
	async resume(session: UploadSession): Promise<UploadSession> {
		// Query YouTube for how much was uploaded
		const bytesUploaded = await invokeCommand<number>('get_upload_status', {
			uploadUri: session.uploadUri,
			totalSize: session.fileSize
		});

		console.log(`[YouTubeUpload] Resuming from byte ${bytesUploaded}`);

		return {
			...session,
			bytesUploaded,
			status: 'uploading'
		};
	}

	// Cancel an upload
	async cancel(session: UploadSession): Promise<void> {
		try {
			await invokeCommand('cancel_upload', { uploadUri: session.uploadUri });
			console.log('[YouTubeUpload] Upload cancelled');
		} catch (error) {
			console.warn('[YouTubeUpload] Failed to cancel upload:', error);
		}

		// Clean up listener if active
		if (this.progressUnlisten) {
			this.progressUnlisten();
			this.progressUnlisten = null;
		}
	}

	// Finalize - privacy is already set correctly during initializeUpload(),
	// so no additional API call is needed here.
	async finalize(result: UploadResult): Promise<void> {
		console.log(`[YouTubeUpload] Upload finalized: ${result.videoId} (privacy set during initialization)`);
	}
}

// Export singleton instance
export const youtubeUploadService = new YouTubeUploadService();
