import { createStorageBackend, type StorageBackend } from './storage-helpers';
import type { APSSettings } from './aps-api-client';

const DEFAULT_SETTINGS: APSSettings = {
	host: '127.0.0.1',
	port: 31600,
	autoConnect: true,
	timeout: 5000
};

class APSSettingsStore {
	private store: StorageBackend | null = null;
	private readonly storeName = 'aps-settings.json';

	async init(): Promise<void> {
		if (!this.store) {
			this.store = await createStorageBackend(this.storeName);
		}
	}

	async getSettings(): Promise<APSSettings> {
		await this.init();

		if (!this.store) {
			return DEFAULT_SETTINGS;
		}

		try {
			const host = (await this.store.get('host')) ?? DEFAULT_SETTINGS.host;
			const port = (await this.store.get('port')) ?? DEFAULT_SETTINGS.port;
			const autoConnect = (await this.store.get('autoConnect')) ?? DEFAULT_SETTINGS.autoConnect;
			const timeout = (await this.store.get('timeout')) ?? DEFAULT_SETTINGS.timeout;

			return {
				host,
				port,
				autoConnect,
				timeout
			};
		} catch (error) {
			console.error('Failed to load APS settings:', error);
			return DEFAULT_SETTINGS;
		}
	}

	async saveSettings(settings: Partial<APSSettings>): Promise<void> {
		await this.init();

		if (!this.store) {
			throw new Error('Store not initialized');
		}

		try {
			if (settings.host !== undefined) {
				await this.store.set('host', settings.host);
			}
			if (settings.port !== undefined) {
				await this.store.set('port', settings.port);
			}
			if (settings.autoConnect !== undefined) {
				await this.store.set('autoConnect', settings.autoConnect);
			}
			if (settings.timeout !== undefined) {
				await this.store.set('timeout', settings.timeout);
			}
			await this.store.save();
		} catch (error) {
			console.error('Failed to save APS settings:', error);
			throw error;
		}
	}

	async resetSettings(): Promise<void> {
		await this.saveSettings(DEFAULT_SETTINGS);
	}
}

// Export singleton instance
export const apsSettingsStore = new APSSettingsStore();
