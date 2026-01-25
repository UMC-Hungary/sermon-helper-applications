import { appSettingsStore, type AppSettings } from './app-settings-store';
import { isTauriApp } from './storage-helpers';

const SCHEMA_VERSION = 1;

export interface ExportOptions {
	includeYoutubeTokens?: boolean; // Default: false (sensitive data)
}

export interface ExportedSettings {
	schemaVersion: number;
	exportedAt: string;
	settings: Partial<AppSettings>;
}

/**
 * Export all app settings to a JSON blob
 */
export async function exportSettings(options?: ExportOptions): Promise<Blob> {
	const includeTokens = options?.includeYoutubeTokens ?? false;

	const settings = await appSettingsStore.getAll();

	// Optionally strip sensitive data
	const exportedSettings: Partial<AppSettings> = { ...settings };
	if (!includeTokens) {
		delete exportedSettings.youtubeTokens;
		delete exportedSettings.youtubeOAuthConfig;
	}

	const exportData: ExportedSettings = {
		schemaVersion: SCHEMA_VERSION,
		exportedAt: new Date().toISOString(),
		settings: exportedSettings,
	};

	const json = JSON.stringify(exportData, null, 2);
	return new Blob([json], { type: 'application/json' });
}

/**
 * Import settings from a JSON file or string
 */
export async function importSettings(data: File | string): Promise<void> {
	let jsonString: string;

	if (data instanceof File) {
		jsonString = await data.text();
	} else {
		jsonString = data;
	}

	const parsed = JSON.parse(jsonString) as ExportedSettings;

	// Validate structure
	if (!parsed.schemaVersion || !parsed.settings) {
		throw new Error('Invalid settings file format');
	}

	// Future: handle schema migrations here if schemaVersion < SCHEMA_VERSION

	const settings = parsed.settings;

	// Import each setting key
	const validKeys: (keyof AppSettings)[] = [
		'bibleTranslation',
		'eventList',
		'draftEvent',
		'draftEventOriginalId',
		'draftSaved',
		'youtubeTokens',
		'youtubeOAuthConfig',
		'obsDevicesSettings',
		'pptxOutputPath',
		'discoverySettings',
		'rfIrSettings',
	];

	for (const key of validKeys) {
		if (key in settings) {
			await appSettingsStore.set(key, settings[key] as AppSettings[typeof key]);
		}
	}
}

/**
 * Save settings using Tauri native file dialog (desktop only)
 */
export async function saveSettingsWithDialog(options?: ExportOptions): Promise<boolean> {
	if (!isTauriApp()) {
		return false;
	}

	try {
		const blob = await exportSettings(options);
		const { save } = await import('@tauri-apps/plugin-dialog');
		const { writeTextFile } = await import('@tauri-apps/plugin-fs');

		const filePath = await save({
			filters: [{ name: 'JSON', extensions: ['json'] }],
			defaultPath: `sermon-helper-settings-${new Date().toISOString().split('T')[0]}.json`,
		});

		if (filePath) {
			const text = await blob.text();
			await writeTextFile(filePath, text);
			return true;
		}
		return false;
	} catch (error) {
		console.error('Failed to save settings:', error);
		throw error;
	}
}

/**
 * Load settings using Tauri native file dialog (desktop only)
 */
export async function loadSettingsWithDialog(): Promise<boolean> {
	if (!isTauriApp()) {
		return false;
	}

	try {
		const { open } = await import('@tauri-apps/plugin-dialog');
		const { readTextFile } = await import('@tauri-apps/plugin-fs');

		const filePath = await open({
			filters: [{ name: 'JSON', extensions: ['json'] }],
			multiple: false,
		});

		if (filePath && typeof filePath === 'string') {
			const text = await readTextFile(filePath);
			await importSettings(text);
			return true;
		}
		return false;
	} catch (error) {
		console.error('Failed to load settings:', error);
		throw error;
	}
}

/**
 * Download settings as file (browser mode)
 */
export async function downloadSettings(options?: ExportOptions): Promise<void> {
	const blob = await exportSettings(options);
	const url = URL.createObjectURL(blob);
	const a = document.createElement('a');
	a.href = url;
	a.download = `sermon-helper-settings-${new Date().toISOString().split('T')[0]}.json`;
	document.body.appendChild(a);
	a.click();
	document.body.removeChild(a);
	URL.revokeObjectURL(url);
}
