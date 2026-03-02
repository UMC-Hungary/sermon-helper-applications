import { writable } from 'svelte/store';
import { browser } from '$app/environment';

const STORAGE_KEY = 'streamPreviewEnabled';

function createEnabledStore() {
	const initial = browser ? localStorage.getItem(STORAGE_KEY) === 'true' : false;
	const { subscribe, set } = writable(initial);
	return {
		subscribe,
		set: (value: boolean) => {
			if (browser) localStorage.setItem(STORAGE_KEY, String(value));
			set(value);
		}
	};
}

export const streamPreviewEnabled = createEnabledStore();
