// File info utility
// Gets file metadata for recordings

import type { RecordingFile } from '$lib/types/event';
import { invoke } from '@tauri-apps/api/core';

interface TauriFileMetadata {
	path: string;
	name: string;
	size: number;
	duration: number; // seconds
	createdAt: number; // timestamp ms
	modifiedAt: number; // timestamp ms
}

/**
 * Get file info for a recording file
 * Uses Tauri to get file metadata including duration
 */
export async function getFileInfo(filePath: string): Promise<RecordingFile> {
	try {
		// Try to get file info via Tauri command
		const metadata = await invoke<TauriFileMetadata>('get_file_metadata', { path: filePath });
		return {
			path: metadata.path,
			name: metadata.name,
			size: metadata.size,
			duration: metadata.duration,
			createdAt: metadata.createdAt,
			modifiedAt: metadata.modifiedAt
		};
	} catch (error) {
		console.warn('[getFileInfo] Tauri command failed, using fallback:', error);

		// Fallback: extract name from path, set defaults for other fields
		const name = filePath.split(/[/\\]/).pop() || 'recording';
		const now = Date.now();

		return {
			path: filePath,
			name,
			size: 0, // Unknown
			duration: 0, // Unknown - user may need to whitelist
			createdAt: now,
			modifiedAt: now
		};
	}
}
