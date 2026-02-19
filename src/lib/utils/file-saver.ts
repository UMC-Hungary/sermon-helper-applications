import { isTauriApp } from './storage-helpers';

export interface SaveResult {
	success: boolean;
	path?: string;
	error?: string;
}

/**
 * Save a file to the file system (Tauri) or trigger browser download
 */
export async function saveFile(
	blob: Blob,
	filename: string,
	outputPath: string | null
): Promise<SaveResult> {
	if (isTauriApp()) {
		return saveFileTauri(blob, filename, outputPath);
	} else {
		return downloadFile(blob, filename);
	}
}

async function saveFileTauri(
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

		const dirExists = await exists(outputPath);
		if (!dirExists) {
			await mkdir(outputPath, { recursive: true });
		}

		const arrayBuffer = await blob.arrayBuffer();
		const contents = new Uint8Array(arrayBuffer);
		const fullPath = await join(outputPath, filename);

		await writeFile(fullPath, contents);

		return { success: true, path: fullPath };
	} catch (error) {
		console.error('Failed to save file:', error);
		const errorMessage = error instanceof Error ? error.message : String(error);

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

async function downloadFile(blob: Blob, filename: string): Promise<SaveResult> {
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
		console.error('Failed to download file:', error);
		return {
			success: false,
			error: error instanceof Error ? error.message : 'Failed to download file',
		};
	}
}

/**
 * Open folder picker dialog (Tauri only)
 */
export async function pickOutputFolder(title = 'Select Output Folder'): Promise<string | null> {
	if (!isTauriApp()) {
		return null;
	}

	try {
		const { open } = await import('@tauri-apps/plugin-dialog');
		const selected = await open({
			directory: true,
			multiple: false,
			title,
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
