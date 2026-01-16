import { isTauriApp } from './storage-helpers';

export interface SaveResult {
	success: boolean;
	path?: string;
	error?: string;
}

/**
 * Save a PPTX file to the file system (Tauri) or trigger download (browser)
 */
export async function savePptxFile(
	blob: Blob,
	filename: string,
	outputPath: string | null
): Promise<SaveResult> {
	if (isTauriApp()) {
		return savePptxFileTauri(blob, filename, outputPath);
	} else {
		return downloadPptxFile(blob, filename);
	}
}

/**
 * Save file using Tauri file system plugin
 */
async function savePptxFileTauri(
	blob: Blob,
	filename: string,
	outputPath: string | null
): Promise<SaveResult> {
	if (!outputPath) {
		return { success: false, error: 'Output path not configured' };
	}

	try {
		const { writeFile, mkdir, exists } = await import('@tauri-apps/plugin-fs');
		const { join } = await import('@tauri-apps/api/path');

		// Ensure directory exists
		const dirExists = await exists(outputPath);
		if (!dirExists) {
			await mkdir(outputPath, { recursive: true });
		}

		// Convert blob to Uint8Array
		const arrayBuffer = await blob.arrayBuffer();
		const contents = new Uint8Array(arrayBuffer);

		// Build full path
		const fullPath = await join(outputPath, filename);

		// Write file
		await writeFile(fullPath, contents);

		return { success: true, path: fullPath };
	} catch (error) {
		console.error('Failed to save PPTX file:', error);
		const errorMessage = error instanceof Error ? error.message : String(error);

		// Check for Tauri permission errors
		if (errorMessage.includes('not allowed') || errorMessage.includes('Permissions')) {
			return {
				success: false,
				error: 'Permission denied. Please select a folder in your Documents, Downloads, Desktop, or Home directory.',
			};
		}

		return {
			success: false,
			error: errorMessage || 'Failed to save file',
		};
	}
}

/**
 * Download file in browser mode
 */
async function downloadPptxFile(blob: Blob, filename: string): Promise<SaveResult> {
	try {
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = filename;
		document.body.appendChild(a);
		a.click();
		document.body.removeChild(a);
		URL.revokeObjectURL(url);

		return { success: true, path: filename };
	} catch (error) {
		console.error('Failed to download PPTX file:', error);
		return {
			success: false,
			error: error instanceof Error ? error.message : 'Failed to download file',
		};
	}
}

/**
 * Open folder picker dialog (Tauri only)
 */
export async function pickOutputFolder(): Promise<string | null> {
	if (!isTauriApp()) {
		return null;
	}

	try {
		const { open } = await import('@tauri-apps/plugin-dialog');
		const selected = await open({
			directory: true,
			multiple: false,
			title: 'Select PPTX Output Folder',
		});

		return selected as string | null;
	} catch (error) {
		console.error('Failed to open folder picker:', error);
		return null;
	}
}

/**
 * Open a folder in the system file explorer (Tauri only)
 */
export async function openFolder(path: string): Promise<void> {
	if (!isTauriApp()) {
		return;
	}

	try {
		const { openPath } = await import('@tauri-apps/plugin-opener');
		await openPath(path);
	} catch (error) {
		console.error('Failed to open folder:', error);
	}
}
