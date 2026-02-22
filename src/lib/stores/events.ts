import { writable } from 'svelte/store';
import type { EventSummary } from '$lib/types/event.js';

export const events = writable<EventSummary[]>([]);
export const eventsLoading = writable<boolean>(false);
