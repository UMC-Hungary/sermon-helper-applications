import { writable, get } from 'svelte/store';
import { createStorageBackend, type StorageBackend } from './storage-helpers';
import type { ServiceEvent } from '$lib/types/event';
import type { YouTubeTokens, YouTubeOAuthConfig } from '$lib/types/youtube';
import type { ObsDevicesSettings } from '$lib/types/obs-devices';
import { DEFAULT_OBS_DEVICES_SETTINGS } from '$lib/types/obs-devices';

export interface AppSettings {
	bibleTranslation: string;
	eventList: ServiceEvent[];
	draftEvent: ServiceEvent | null;
	draftEventOriginalId: string | null; // If editing, stores the original event ID
	draftSaved: boolean; // True if the draft was saved (form should not reopen)
	// YouTube OAuth settings
	youtubeTokens: YouTubeTokens | null;
	youtubeOAuthConfig: YouTubeOAuthConfig | null;
	// OBS device and source configurations
	obsDevicesSettings: ObsDevicesSettings;
	// PPTX output folder path (Tauri only)
	pptxOutputPath: string | null;
}

const DEFAULT_SETTINGS: AppSettings = {
	bibleTranslation: 'RUF_v2',
	eventList: [],
	draftEvent: null,
	draftEventOriginalId: null,
	draftSaved: false,
	youtubeTokens: null,
	youtubeOAuthConfig: null,
	obsDevicesSettings: DEFAULT_OBS_DEVICES_SETTINGS,
	pptxOutputPath: null,
};

// Reactive store for app settings - can be subscribed to by components
export const appSettings = writable<AppSettings>({ ...DEFAULT_SETTINGS });

// Loading state
export const appSettingsLoaded = writable<boolean>(false);

class AppSettingsStore {
	private store: StorageBackend | null = null;
	private readonly storeName = 'app-settings.json';
	private initialized = false;

	async init(): Promise<void> {
		if (!this.store) {
			this.store = await createStorageBackend(this.storeName);
		}
	}

	// Load all settings and populate the reactive store
	// Call this once in the layout before rendering
	async load(): Promise<AppSettings> {
		if (this.initialized) {
			return get(appSettings);
		}

		await this.init();
		const settings = await this.getAll();
		appSettings.set(settings);
		appSettingsLoaded.set(true);
		this.initialized = true;
		return settings;
	}

	async get<K extends keyof AppSettings>(key: K): Promise<AppSettings[K]> {
		await this.init();

		if (!this.store) {
			return DEFAULT_SETTINGS[key];
		}

		try {
			const value = await this.store.get(key);
			return value ?? DEFAULT_SETTINGS[key];
		} catch (error) {
			console.error(`Failed to get app setting '${key}':`, error);
			return DEFAULT_SETTINGS[key];
		}
	}

	async set<K extends keyof AppSettings>(key: K, value: AppSettings[K]): Promise<void> {
		await this.init();

		if (!this.store) {
			throw new Error('Store not initialized');
		}

		try {
			await this.store.set(key, value);
			await this.store.save();

			// Update the reactive store
			appSettings.update((current) => ({
				...current,
				[key]: value,
			}));
		} catch (error) {
			console.error(`Failed to set app setting '${key}':`, error);
			throw error;
		}
	}

	async getAll(): Promise<AppSettings> {
		await this.init();

		if (!this.store) {
			return { ...DEFAULT_SETTINGS };
		}

		try {
			const settings: AppSettings = { ...DEFAULT_SETTINGS };

			for (const key of Object.keys(DEFAULT_SETTINGS) as (keyof AppSettings)[]) {
				const value = await this.store.get(key);
				if (value !== null) {
					(settings as unknown as Record<string, unknown>)[key] = value;
				}
			}

			return settings;
		} catch (error) {
			console.error('Failed to get all app settings:', error);
			return { ...DEFAULT_SETTINGS };
		}
	}

	async reset(): Promise<void> {
		await this.init();

		if (!this.store) {
			throw new Error('Store not initialized');
		}

		try {
			for (const key of Object.keys(DEFAULT_SETTINGS) as (keyof AppSettings)[]) {
				await this.store.set(key, DEFAULT_SETTINGS[key]);
			}
			await this.store.save();

			// Reset the reactive store
			appSettings.set({ ...DEFAULT_SETTINGS });
		} catch (error) {
			console.error('Failed to reset app settings:', error);
			throw error;
		}
	}
}

// Export singleton instance
export const appSettingsStore = new AppSettingsStore();
