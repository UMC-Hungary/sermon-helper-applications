import { writable } from 'svelte/store';
import type { EventSummary } from '$lib/schemas/event.js';
import type { UntrackedRecording } from '$lib/schemas/untracked-recording.js';

export const events = writable<EventSummary[]>([]);
export const eventsLoading = writable<boolean>(false);
export const untrackedRecordings = writable<UntrackedRecording[]>([]);
