// Common interface for all upload platforms

import type { UploadPlatform } from '$lib/types/upload-config';

// Common metadata for all platforms
export interface UploadMetadata {
	title: string;
	description: string;
	privacy: string; // Platform-specific privacy value
	tags?: string[];
	thumbnailPath?: string;
}

// Upload session (persisted for resume capability)
export interface UploadSession {
	id: string;
	platform: UploadPlatform;
	filePath: string;
	fileSize: number;
	metadata: UploadMetadata;
	uploadUri: string; // Platform-specific upload URI
	bytesUploaded: number;
	startedAt: number;
	status: 'pending' | 'uploading' | 'paused' | 'processing' | 'completed' | 'failed';
	error?: string;
}

// Upload progress event
export interface UploadProgress {
	bytesUploaded: number;
	totalBytes: number;
	percentage: number;
}

// Upload result
export interface UploadResult {
	platform: UploadPlatform;
	videoId: string;
	videoUrl: string;
	processingStatus: 'processing' | 'ready' | 'failed';
}

// Common interface for all upload platforms
export interface IUploadService {
	readonly platform: UploadPlatform;
	readonly displayName: string;

	// Check if platform is configured and ready
	isConfigured(): Promise<boolean>;

	// Initialize upload session
	initializeUpload(filePath: string, metadata: UploadMetadata): Promise<UploadSession>;

	// Upload file with progress
	upload(
		session: UploadSession,
		onProgress: (progress: UploadProgress) => void
	): Promise<UploadResult>;

	// Resume interrupted upload
	resume(session: UploadSession): Promise<UploadSession>;

	// Cancel upload
	cancel(session: UploadSession): Promise<void>;

	// Post-upload actions (publish, etc.)
	finalize(result: UploadResult): Promise<void>;
}

// Create a new upload session
export function createUploadSession(
	platform: UploadPlatform,
	filePath: string,
	fileSize: number,
	metadata: UploadMetadata,
	uploadUri: string
): UploadSession {
	return {
		id: crypto.randomUUID(),
		platform,
		filePath,
		fileSize,
		metadata,
		uploadUri,
		bytesUploaded: 0,
		startedAt: Date.now(),
		status: 'pending'
	};
}
