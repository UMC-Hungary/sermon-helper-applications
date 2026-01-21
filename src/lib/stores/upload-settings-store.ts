// Upload Settings Store
// Manages multi-platform upload configuration

import { writable, derived, get } from 'svelte/store';
import { appSettingsStore, type AppSettings } from '$lib/utils/app-settings-store';
import type {
	UploadSettings,
	UploadPlatform,
	PlatformUploadConfig,
	YouTubeUploadConfig
} from '$lib/types/upload-config';
import { DEFAULT_UPLOAD_SETTINGS, getPlatformConfig, isPlatformEnabled } from '$lib/types/upload-config';

// Internal writable store
const settingsStore = writable<UploadSettings>(DEFAULT_UPLOAD_SETTINGS);

// Loading state
const settingsLoaded = writable<boolean>(false);

// Derived stores for UI
export const uploadSettings = derived(settingsStore, ($settings) => $settings);
export const enabledPlatforms = derived(settingsStore, ($settings) =>
	$settings.platforms.filter((p) => p.enabled).map((p) => p.platform)
);
export const youtubeConfig = derived(settingsStore, ($settings) =>
	getPlatformConfig<YouTubeUploadConfig>($settings, 'youtube')
);

// Upload settings store operations
class UploadSettingsStore {
	private initialized = false;

	// Initialize and load persisted settings
	async init(): Promise<void> {
		if (this.initialized) return;

		try {
			const stored = await appSettingsStore.get('uploadSettings' as keyof AppSettings);
			const settings = (stored as UploadSettings | null) ?? DEFAULT_UPLOAD_SETTINGS;

			// Merge with defaults to ensure all fields exist
			const merged = this.mergeWithDefaults(settings);
			settingsStore.set(merged);

			settingsLoaded.set(true);
			this.initialized = true;
		} catch (error) {
			console.error('[UploadSettingsStore] Failed to initialize:', error);
			settingsStore.set(DEFAULT_UPLOAD_SETTINGS);
			settingsLoaded.set(true);
			this.initialized = true;
		}
	}

	// Merge stored settings with defaults (handles version upgrades)
	private mergeWithDefaults(stored: Partial<UploadSettings>): UploadSettings {
		return {
			...DEFAULT_UPLOAD_SETTINGS,
			...stored,
			platforms: stored.platforms ?? DEFAULT_UPLOAD_SETTINGS.platforms
		};
	}

	// Persist settings
	private async persist(): Promise<void> {
		const settings = get(settingsStore);
		try {
			await appSettingsStore.set('uploadSettings' as keyof AppSettings, settings as never);
		} catch (error) {
			console.error('[UploadSettingsStore] Failed to persist settings:', error);
		}
	}

	// Get current settings
	getSettings(): UploadSettings {
		return get(settingsStore);
	}

	// Update settings
	async updateSettings(partial: Partial<UploadSettings>): Promise<void> {
		settingsStore.update((current) => ({
			...current,
			...partial
		}));
		await this.persist();
	}

	// Get config for a specific platform
	getPlatformConfig<T extends PlatformUploadConfig>(platform: UploadPlatform): T | undefined {
		const settings = get(settingsStore);
		return getPlatformConfig<T>(settings, platform);
	}

	// Update config for a specific platform
	async setPlatformConfig(config: PlatformUploadConfig): Promise<void> {
		settingsStore.update((current) => {
			const existingIndex = current.platforms.findIndex((p) => p.platform === config.platform);

			const platforms =
				existingIndex >= 0
					? current.platforms.map((p, i) => (i === existingIndex ? config : p))
					: [...current.platforms, config];

			return { ...current, platforms };
		});
		await this.persist();
	}

	// Enable or disable a platform
	async enablePlatform(platform: UploadPlatform, enabled: boolean): Promise<void> {
		settingsStore.update((current) => {
			const platforms = current.platforms.map((p) =>
				p.platform === platform ? { ...p, enabled } : p
			);
			return { ...current, platforms };
		});
		await this.persist();
	}

	// Check if a platform is enabled
	isPlatformEnabled(platform: UploadPlatform): boolean {
		const settings = get(settingsStore);
		return isPlatformEnabled(settings, platform);
	}

	// Check if any platform has auto-upload enabled
	hasAutoUploadEnabled(): boolean {
		const settings = get(settingsStore);
		return settings.platforms.some((p) => p.enabled && p.autoUpload);
	}

	// Get minimum duration in milliseconds
	getMinDurationMs(): number {
		const settings = get(settingsStore);
		return settings.minDurationMinutes * 60 * 1000;
	}

	// Get short video threshold in seconds
	getShortVideoThresholdSeconds(): number {
		const settings = get(settingsStore);
		return settings.shortVideoThresholdMinutes * 60;
	}

	// Get chunk size in bytes
	getChunkSizeBytes(): number {
		const settings = get(settingsStore);
		return settings.chunkSizeMB * 1024 * 1024;
	}

	// Reset to defaults
	async reset(): Promise<void> {
		settingsStore.set(DEFAULT_UPLOAD_SETTINGS);
		await this.persist();
	}
}

export const uploadSettingsStore = new UploadSettingsStore();
