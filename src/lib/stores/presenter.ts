import { writable } from 'svelte/store';
import { browser } from '$app/environment';
import { PresenterStateSchema } from '$lib/schemas/ws-messages.js';
import type { PresenterState, WsClientInfo } from '$lib/schemas/ws-messages.js';

const USE_WEB_PRESENTER_KEY = 'useWebPresenter';

function createUseWebPresenter() {
	const initial = browser ? localStorage.getItem(USE_WEB_PRESENTER_KEY) === 'true' : false;
	const store = writable<boolean>(initial);
	if (browser) {
		store.subscribe((val) => {
			localStorage.setItem(USE_WEB_PRESENTER_KEY, String(val));
		});
	}
	return store;
}

export const useWebPresenter = createUseWebPresenter();

const emptyState: PresenterState = PresenterStateSchema.parse({
	loaded: false,
	filePath: null,
	currentSlide: 0,
	totalSlides: 0,
	slides: [] as { index: number; paragraphs: { text: string; align: string }[] }[],
});

export const presenterState = writable<PresenterState>(emptyState);

export const connectedClients = writable<WsClientInfo[]>([]);
