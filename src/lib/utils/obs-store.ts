import { load } from '@tauri-apps/plugin-store';

export interface ObsSettings {
	websocketUrl: string;
	websocketPassword: string;
}

const DEFAULT_SETTINGS: ObsSettings = {
	websocketUrl: 'ws://localhost:4455',
	websocketPassword: '',
};

// Check if running in Tauri environment
const isTauriApp = () => {
	return typeof window !== 'undefined' && 
		   // @ts-ignore - Tauri internal property
		   typeof (window as any).__TAURI_INTERNALS__ !== 'undefined';
};

// LocalStorage fallback for browser development
class LocalStorageStore {
	constructor(private storeName: string) {}

	async get(key: string): Promise<any> {
		const value = localStorage.getItem(`${this.storeName}_${key}`);
		return value ? JSON.parse(value) : null;
	}
	
	async set(key: string, value: any): Promise<void> {
		localStorage.setItem(`${this.storeName}_${key}`, JSON.stringify(value));
	}
	
	async save(): Promise<void> {
		// localStorage is auto-saving
	}
}

class ObsSettingsStore {
	private store: any = null;
	private readonly storeName = 'obs-settings.json';

	async init(): Promise<void> {
		if (!this.store) {
			if (isTauriApp()) {
				try {
					this.store = await load(this.storeName);
				} catch (error) {
					console.warn('Failed to initialize Tauri store, using localStorage fallback:', error);
					this.store = new LocalStorageStore(this.storeName);
				}
			} else {
				// Browser environment - use localStorage
				this.store = new LocalStorageStore(this.storeName);
			}
		}
	}

	async getSettings(): Promise<ObsSettings> {
		await this.init();
		
		if (!this.store) {
			return DEFAULT_SETTINGS;
		}

		try {
			const url = (await this.store.get('websocketUrl')) || DEFAULT_SETTINGS.websocketUrl;
			const password = (await this.store.get('websocketPassword')) || DEFAULT_SETTINGS.websocketPassword;
			
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