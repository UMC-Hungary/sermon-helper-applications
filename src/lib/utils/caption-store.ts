import { createStorageBackend, type StorageBackend } from './storage-helpers';

export interface CaptionSettings {
	type: 'caption' | 'full';
	title: string;
	boldText: string;
	lightText: string;
	color: string;
	showLogo: boolean;
	svgLogo: string;
}

const DEFAULT_SETTINGS: CaptionSettings = {
	type: 'caption',
	title: '',
	boldText: '',
	lightText: '',
	color: 'black',
	showLogo: true,
	svgLogo: '',
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

			return {
				type,
				title,
				boldText,
				lightText,
				color,
				showLogo,
				svgLogo,
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
