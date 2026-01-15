import { writable, get } from 'svelte/store';
import { browser } from '$app/environment';

export type Theme = 'system' | 'dark' | 'light';

// Check if running in Tauri environment
const isTauriApp = () => {
	return browser && typeof (window as any).__TAURI_INTERNALS__ !== 'undefined';
};

// Writable store for current theme
export const theme = writable<Theme>('system');

// Media query for system preference
let systemPreferenceQuery: MediaQueryList | null = null;

// Apply theme to document
function applyTheme(themeValue: Theme): void {
	if (!browser) return;

	let isDark = false;

	if (themeValue === 'system') {
		// Check system preference
		isDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
	} else {
		isDark = themeValue === 'dark';
	}

	if (isDark) {
		document.documentElement.classList.add('dark');
	} else {
		document.documentElement.classList.remove('dark');
	}
}

// Handle system preference change
function handleSystemPreferenceChange(e: MediaQueryListEvent): void {
	const currentTheme = get(theme);
	if (currentTheme === 'system') {
		applyTheme('system');
	}
}

// Initialize theme from storage
export async function initTheme(): Promise<void> {
	if (!browser) return;

	// First, check localStorage for quick initial load
	let savedTheme: Theme = 'system';
	const localTheme = localStorage.getItem('theme') as Theme | null;
	if (localTheme && ['system', 'dark', 'light'].includes(localTheme)) {
		savedTheme = localTheme;
	}

	// Try to load from Tauri store if available
	try {
		if (isTauriApp()) {
			const { load } = await import('@tauri-apps/plugin-store');
			const store = await load('settings.json');
			const tauriTheme = (await store.get('theme')) as Theme | null;
			if (tauriTheme && ['system', 'dark', 'light'].includes(tauriTheme)) {
				savedTheme = tauriTheme;
			}
		}
	} catch (error) {
		console.warn('Failed to load theme from Tauri store:', error);
	}

	// Set the store and apply
	theme.set(savedTheme);
	applyTheme(savedTheme);

	// Set up system preference listener
	systemPreferenceQuery = window.matchMedia('(prefers-color-scheme: dark)');
	systemPreferenceQuery.addEventListener('change', handleSystemPreferenceChange);
}

// Save theme to storage
async function saveTheme(themeValue: Theme): Promise<void> {
	if (!browser) return;

	try {
		// Always save to localStorage for quick access
		localStorage.setItem('theme', themeValue);

		// Also save to Tauri store if available
		if (isTauriApp()) {
			const { load } = await import('@tauri-apps/plugin-store');
			const store = await load('settings.json');
			await store.set('theme', themeValue);
			await store.save();
		}
	} catch (error) {
		console.warn('Failed to save theme:', error);
	}
}

// Set theme and persist
export async function setTheme(newTheme: Theme): Promise<void> {
	theme.set(newTheme);
	applyTheme(newTheme);
	await saveTheme(newTheme);
}

// Clean up listener (call on app unmount if needed)
export function cleanupTheme(): void {
	if (systemPreferenceQuery) {
		systemPreferenceQuery.removeEventListener('change', handleSystemPreferenceChange);
	}
}
