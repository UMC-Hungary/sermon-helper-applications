import { browser } from '$app/environment';

export type CaptionType = 'caption' | 'preview';
export type Resolution = '1080p' | '4k';

export interface CaptionSettings {
	type: CaptionType;
	title: string;
	boldText: string;
	lightText: string;
	color: string;
	showLogo: boolean;
	svgLogo: string;
	resolution: Resolution;
}

export const RESOLUTION_DIMENSIONS: Record<Resolution, { width: number; height: number }> = {
	'1080p': { width: 1920, height: 1080 },
	'4k': { width: 3840, height: 2160 },
};

export function getCaptionHeight(type: CaptionType, resolution: Resolution): number {
	if (type === 'caption') {
		return resolution === '4k' ? 300 : 150;
	}
	return RESOLUTION_DIMENSIONS[resolution].height;
}

const isTauriApp = (): boolean =>
	browser &&
	typeof (window as unknown as { __TAURI_INTERNALS__?: object }).__TAURI_INTERNALS__ !==
		'undefined';

const STORE_NAME = 'caption-settings.json';

const DEFAULT_SETTINGS: CaptionSettings = {
	type: 'caption',
	title: '',
	boldText: '',
	lightText: '',
	color: 'black',
	showLogo: true,
	svgLogo: '',
	resolution: '1080p',
};

class CaptionSettingsStore {
	getDefaultSettings(): CaptionSettings {
		return { ...DEFAULT_SETTINGS };
	}

	async getSettings(): Promise<CaptionSettings> {
		if (!browser) return this.getDefaultSettings();

		try {
			if (isTauriApp()) {
				const { load } = await import('@tauri-apps/plugin-store');
				const store = await load(STORE_NAME);
				const type = ((await store.get('type')) as CaptionType | null) ?? DEFAULT_SETTINGS.type;
				const title = ((await store.get('title')) as string | null) ?? DEFAULT_SETTINGS.title;
				const boldText =
					((await store.get('boldText')) as string | null) ?? DEFAULT_SETTINGS.boldText;
				const lightText =
					((await store.get('lightText')) as string | null) ?? DEFAULT_SETTINGS.lightText;
				const color = ((await store.get('color')) as string | null) ?? DEFAULT_SETTINGS.color;
				const showLogo =
					((await store.get('showLogo')) as boolean | null) ?? DEFAULT_SETTINGS.showLogo;
				const svgLogo =
					((await store.get('svgLogo')) as string | null) ?? DEFAULT_SETTINGS.svgLogo;
				const resolution =
					((await store.get('resolution')) as Resolution | null) ?? DEFAULT_SETTINGS.resolution;
				return { type, title, boldText, lightText, color, showLogo, svgLogo, resolution };
			}
		} catch (error) {
			console.warn('Failed to load caption settings from Tauri store:', error);
		}

		// Browser localStorage fallback
		try {
			const raw = localStorage.getItem(STORE_NAME);
			if (raw) {
				const parsed = JSON.parse(raw) as Partial<CaptionSettings>;
				return { ...DEFAULT_SETTINGS, ...parsed };
			}
		} catch {
			// ignore parse errors
		}
		return this.getDefaultSettings();
	}

	async saveSettings(settings: Partial<CaptionSettings>): Promise<void> {
		if (!browser) return;

		const merged = { ...(await this.getSettings()), ...settings };

		try {
			if (isTauriApp()) {
				const { load } = await import('@tauri-apps/plugin-store');
				const store = await load(STORE_NAME);
				await store.set('type', merged.type);
				await store.set('title', merged.title);
				await store.set('boldText', merged.boldText);
				await store.set('lightText', merged.lightText);
				await store.set('color', merged.color);
				await store.set('showLogo', merged.showLogo);
				await store.set('svgLogo', merged.svgLogo);
				await store.set('resolution', merged.resolution);
				await store.save();
				return;
			}
		} catch (error) {
			console.warn('Failed to save caption settings to Tauri store:', error);
		}

		// Browser localStorage fallback
		localStorage.setItem(STORE_NAME, JSON.stringify(merged));
	}
}

export const captionSettingsStore = new CaptionSettingsStore();
