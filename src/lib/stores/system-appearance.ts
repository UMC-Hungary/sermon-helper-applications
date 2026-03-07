import { getCurrentWindow } from '@tauri-apps/api/window';
import { writable } from 'svelte/store';

export const systemTheme = writable<'light' | 'dark'>('light');
export const glassSupported = writable(false);
export const reduceTransparency = writable(false);

export async function initSystemAppearance(): Promise<void> {
	const win = getCurrentWindow();

	const theme = await win.theme();
	systemTheme.set(theme === 'dark' ? 'dark' : 'light');
	await win.onThemeChanged(({ payload }) => {
		systemTheme.set(payload === 'dark' ? 'dark' : 'light');
	});

	const mq = window.matchMedia('(prefers-reduced-transparency: reduce)');
	reduceTransparency.set(mq.matches);
	mq.addEventListener('change', (e) => reduceTransparency.set(e.matches));

	try {
		const { isGlassSupported, setLiquidGlassEffect } = await import(/* @vite-ignore */ 'tauri-plugin-liquid-glass-api');
		const supported = await isGlassSupported();
		glassSupported.set(supported);
		if (supported) {
			await setLiquidGlassEffect({});
		}
	} catch {
		// tauri-plugin-liquid-glass is macOS-only; not available on other platforms
	}
}
