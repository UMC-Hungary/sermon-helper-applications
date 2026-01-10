import { register, init, getLocaleFromNavigator, locale } from 'svelte-i18n';
import { browser } from '$app/environment';

// Register locales
register('en', () => import('./locales/en.json'));
register('hu', () => import('./locales/hu.json'));

// Check if running in Tauri environment
const isTauriApp = () => {
	return browser &&
		   typeof (window as any).__TAURI_INTERNALS__ !== 'undefined';
};

// Get initial locale synchronously (for SSR compatibility)
function getInitialLocale(): string {
	if (browser) {
		const saved = localStorage.getItem('locale');
		if (saved) return saved;
	}
	return 'en';
}

// Initialize synchronously with default locale
init({
	fallbackLocale: 'en',
	initialLocale: getInitialLocale(),
});

// Load saved locale from Tauri store (called after mount)
export async function loadSavedLocale(): Promise<void> {
	if (!browser) return;

	try {
		if (isTauriApp()) {
			const { load } = await import('@tauri-apps/plugin-store');
			const store = await load('settings.json');
			const savedLocale = await store.get('locale') as string | null;
			if (savedLocale) {
				locale.set(savedLocale);
			}
		}
	} catch (error) {
		console.warn('Failed to load saved locale from Tauri store:', error);
	}
}

// Save locale to storage
export async function saveLocale(newLocale: string): Promise<void> {
	if (!browser) return;

	try {
		// Always save to localStorage for quick access
		localStorage.setItem('locale', newLocale);

		// Also save to Tauri store if available
		if (isTauriApp()) {
			const { load } = await import('@tauri-apps/plugin-store');
			const store = await load('settings.json');
			await store.set('locale', newLocale);
			await store.save();
		}
	} catch (error) {
		console.warn('Failed to save locale:', error);
	}
}

// Set locale and persist
export async function setLocale(newLocale: string): Promise<void> {
	locale.set(newLocale);
	await saveLocale(newLocale);
}

// Available locales for the language switcher
export const availableLocales = [
	{ code: 'en', name: 'English', flag: '🇬🇧' },
	{ code: 'hu', name: 'Magyar', flag: '🇭🇺' },
];
