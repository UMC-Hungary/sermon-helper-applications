import { writable } from 'svelte/store';
import { PresenterStateSchema } from '$lib/schemas/ws-messages.js';
import type { PresenterState, WsClientInfo } from '$lib/schemas/ws-messages.js';

/** Reflects the backend-persisted use_web_presenter setting; updated on every presentation.settings push. */
export const useWebPresenter = writable<boolean>(false);

const emptyState: PresenterState = PresenterStateSchema.parse({
	loaded: false,
	filePath: null,
	currentSlide: 0,
	totalSlides: 0,
	slides: [],
	muted: false,
});

export const presenterState = writable<PresenterState>(emptyState);

export const connectedClients = writable<WsClientInfo[]>([]);
