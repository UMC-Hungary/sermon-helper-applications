import { getCurrentWindow } from '@tauri-apps/api/window';
import { isGlassSupported, setLiquidGlassEffect } from 'tauri-plugin-liquid-glass-api';
import { writable } from 'svelte/store';

export const systemTheme = writable<'light' | 'dark'>('light');
export const glassSupported = writable(false);
export const reduceTransparency = writable(false);

export async function initSystemAppearance(): Promise<void> {
	// Tauri APIs are only available in the main webview, not in iframes or plain
	// browsers.  Guard before touching any Tauri API so that the presenter and
	// caption pages (which are accessed without Tauri context) don't throw.
	if (
		typeof (window as Window & { __TAURI_INTERNALS__?: Record<string, unknown> })
			.__TAURI_INTERNALS__ === 'undefined'
	) {
		return;
	}

	const win = getCurrentWindow();

	const theme = await win.theme();
	systemTheme.set(theme === 'dark' ? 'dark' : 'light');
	await win.onThemeChanged(({ payload }) => {
		systemTheme.set(payload === 'dark' ? 'dark' : 'light');
	});

	const mq = window.matchMedia('(prefers-reduced-transparency: reduce)');
	reduceTransparency.set(mq.matches);
	mq.addEventListener('change', (e) => reduceTransparency.set(e.matches));

	const supported = await isGlassSupported();
	glassSupported.set(supported);
	if (supported) {
		await setLiquidGlassEffect({});
	}
}
