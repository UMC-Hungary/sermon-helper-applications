import { createStorageBackend, type StorageBackend } from './storage-helpers';

export interface ObsSettings {
	websocketUrl: string;
	websocketPassword: string;
}

const DEFAULT_SETTINGS: ObsSettings = {
	websocketUrl: 'ws://localhost:4455',
	websocketPassword: '',
};

class ObsSettingsStore {
	private store: StorageBackend | null = null;
	private readonly storeName = 'obs-settings.json';

	async init(): Promise<void> {
		if (!this.store) {
			this.store = await createStorageBackend(this.storeName);
		}
	}

	async getSettings(): Promise<ObsSettings> {
		await this.init();

		if (!this.store) {
			return DEFAULT_SETTINGS;
		}

		try {
			const url = (await this.store.get('websocketUrl')) ?? DEFAULT_SETTINGS.websocketUrl;
			const password =
				(await this.store.get('websocketPassword')) ?? DEFAULT_SETTINGS.websocketPassword;

			return {
				websocketUrl: url,
				websocketPassword: password,
			};
		} catch (error) {
			console.error('Failed to load OBS settings:', error);
			return DEFAULT_SETTINGS;
		}
	}

	async saveSettings(settings: Partial<ObsSettings>): Promise<void> {
		await this.init();

		if (!this.store) {
			throw new Error('Store not initialized');
		}

		try {
			if (settings.websocketUrl !== undefined) {
				await this.store.set('websocketUrl', settings.websocketUrl);
			}
			if (settings.websocketPassword !== undefined) {
				await this.store.set('websocketPassword', settings.websocketPassword);
			}
			await this.store.save();
		} catch (error) {
			console.error('Failed to save OBS settings:', error);
			throw error;
		}
	}

	async resetSettings(): Promise<void> {
		await this.saveSettings(DEFAULT_SETTINGS);
	}
}

// Export singleton instance
export const obsSettingsStore = new ObsSettingsStore();
