import { writable } from 'svelte/store';
import type { QueueSummary } from '$lib/schemas/queue.js';

/// Live depth-by-status per queue, pushed by the `queue.stats` WS message.
export const queues = writable<QueueSummary[]>([]);
