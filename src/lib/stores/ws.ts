import { writable } from 'svelte/store';
import type { WsMessage } from '$lib/types/ws-messages.js';

export type WsStatus = 'connecting' | 'connected' | 'disconnected' | 'error';

export const wsStatus = writable<WsStatus>('disconnected');
export const lastWsMessage = writable<WsMessage | null>(null);
