import { createStorageBackend, type StorageBackend } from './storage-helpers';

export type CaptionType = 'caption' | 'full';
export type Resolution = '1080p' | '4k';
export type AspectRatio = '16:9' | '4:3' | '1:1' | '9:16';

export interface CaptionSettings {
	type: CaptionType;
	title: string;
	boldText: string;
	lightText: string;
	color: string;
	showLogo: boolean;
	svgLogo: string;
	resolution: Resolution;
	aspectRatio: AspectRatio;
}

// Resolution dimensions
export const RESOLUTION_DIMENSIONS: Record<Resolution, { width: number; height: number }> = {
	'1080p': { width: 1920, height: 1080 },
	'4k': { width: 3840, height: 2160 },
};

// Aspect ratio multipliers (width / height)
export const ASPECT_RATIOS: Record<AspectRatio, number> = {
	'16:9': 16 / 9,
	'4:3': 4 / 3,
	'1:1': 1,
	'9:16': 9 / 16,
};

// Get export dimensions based on resolution and aspect ratio
export function getExportDimensions(resolution: Resolution, aspectRatio: AspectRatio): { width: number; height: number } {
	const base = RESOLUTION_DIMENSIONS[resolution];
	const ratio = ASPECT_RATIOS[aspectRatio];

	// Keep the height fixed, adjust width based on aspect ratio
	const height = base.height;
	const width = Math.round(height * ratio);

	return { width, height };
}

// Get caption height based on type and resolution
export function getCaptionHeight(type: CaptionType, resolution: Resolution): number {
	const base = RESOLUTION_DIMENSIONS[resolution];
	if (type === 'caption') {
		// Caption bar is ~14% of screen height
		return Math.round(base.height * 0.14);
	}
	return base.height;
}

const DEFAULT_SETTINGS: CaptionSettings = {
	type: 'caption',
	title: '',
	boldText: '',
	lightText: '',
	color: 'black',
	showLogo: true,
	svgLogo: '',
	resolution: '1080p',
	aspectRatio: '16:9',
};

class CaptionSettingsStore {
	private store: StorageBackend | null = null;
	private readonly storeName = 'caption-settings.json';

	async init(): Promise<void> {
		if (!this.store) {
			this.store = await createStorageBackend(this.storeName);
		}
	}

	async getSettings(): Promise<CaptionSettings> {
		await this.init();

		if (!this.store) {
			return DEFAULT_SETTINGS;
		}

		try {
			const type = (await this.store.get('type')) ?? DEFAULT_SETTINGS.type;
			const title = (await this.store.get('title')) ?? DEFAULT_SETTINGS.title;
			const boldText = (await this.store.get('boldText')) ?? DEFAULT_SETTINGS.boldText;
			const lightText = (await this.store.get('lightText')) ?? DEFAULT_SETTINGS.lightText;
			const color = (await this.store.get('color')) ?? DEFAULT_SETTINGS.color;
			const showLogo = (await this.store.get('showLogo')) ?? DEFAULT_SETTINGS.showLogo;
			const svgLogo = (await this.store.get('svgLogo')) ?? DEFAULT_SETTINGS.svgLogo;
			const resolution = (await this.store.get('resolution')) ?? DEFAULT_SETTINGS.resolution;
			const aspectRatio = (await this.store.get('aspectRatio')) ?? DEFAULT_SETTINGS.aspectRatio;

			return {
				type,
				title,
				boldText,
				lightText,
				color,
				showLogo,
				svgLogo,
				resolution,
				aspectRatio,
			};
		} catch (error) {
			console.error('Failed to load caption settings:', error);
			return DEFAULT_SETTINGS;
		}
	}

	async saveSettings(settings: Partial<CaptionSettings>): Promise<void> {
		await this.init();

		if (!this.store) {
			throw new Error('Store not initialized');
		}

		try {
			if (settings.type !== undefined) {
				await this.store.set('type', settings.type);
			}
			if (settings.title !== undefined) {
				await this.store.set('title', settings.title);
			}
			if (settings.boldText !== undefined) {
				await this.store.set('boldText', settings.boldText);
			}
			if (settings.lightText !== undefined) {
				await this.store.set('lightText', settings.lightText);
			}
			if (settings.color !== undefined) {
				await this.store.set('color', settings.color);
			}
			if (settings.showLogo !== undefined) {
				await this.store.set('showLogo', settings.showLogo);
			}
			if (settings.svgLogo !== undefined) {
				await this.store.set('svgLogo', settings.svgLogo);
			}
			if (settings.resolution !== undefined) {
				await this.store.set('resolution', settings.resolution);
			}
			if (settings.aspectRatio !== undefined) {
				await this.store.set('aspectRatio', settings.aspectRatio);
			}
			await this.store.save();
		} catch (error) {
			console.error('Failed to save caption settings:', error);
			throw error;
		}
	}

	async resetSettings(): Promise<void> {
		await this.saveSettings(DEFAULT_SETTINGS);
	}

	getDefaultSettings(): CaptionSettings {
		return { ...DEFAULT_SETTINGS };
	}
}

// Export singleton instance
export const captionSettingsStore = new CaptionSettingsStore();
