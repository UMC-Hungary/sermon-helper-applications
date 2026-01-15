import { writable, get } from 'svelte/store';
import { eventStore, eventList } from './event-store';
import { youtubeApi } from '$lib/utils/youtube-api';
import { systemStore } from './system-store';
import type { YouTubePrivacyStatus, YouTubeLifeCycleStatus } from '$lib/types/event';

const REFRESH_INTERVAL = 5 * 60 * 1000; // 5 minutes

interface RefreshState {
	lastSync: number | null;
	isSyncing: boolean;
	intervalId: ReturnType<typeof setInterval> | null;
}

function createRefreshStore() {
	const { subscribe, update } = writable<RefreshState>({
		lastSync: null,
		isSyncing: false,
		intervalId: null
	});

	const store = {
		subscribe,

		// Start the global refresh interval
		start() {
			const state = get({ subscribe });
			if (state.intervalId) return; // Already running

			const id = setInterval(() => store.sync(), REFRESH_INTERVAL);
			update((s) => ({ ...s, intervalId: id }));

			// Run immediately on start
			store.sync();
		},

		// Stop the refresh interval
		stop() {
			update((s) => {
				if (s.intervalId) clearInterval(s.intervalId);
				return { ...s, intervalId: null };
			});
		},

		// Sync YouTube broadcasts with local events
		async sync() {
			const $systemStore = get(systemStore);
			if (!$systemStore.youtubeLoggedIn) return;

			update((s) => ({ ...s, isSyncing: true }));

			try {
				const events = get(eventList);
				const scheduledEvents = events.filter((e) => e.youtubeScheduledId);

				for (const event of scheduledEvents) {
					try {
						const broadcast = await youtubeApi.getBroadcast(event.youtubeScheduledId!);

						if (!broadcast) {
							// Broadcast no longer exists - remove ID
							await eventStore.updateEvent(event.id, {
								youtubeScheduledId: undefined,
								youtubeLifeCycleStatus: undefined
							});
						} else {
							// Update status from YouTube
							await eventStore.updateEvent(event.id, {
								youtubePrivacyStatus: broadcast.status
									.privacyStatus as YouTubePrivacyStatus,
								youtubeLifeCycleStatus: broadcast.status
									.lifeCycleStatus as YouTubeLifeCycleStatus
							});
						}
					} catch (error) {
						console.error(`Failed to sync event ${event.id}:`, error);
					}
				}

				update((s) => ({ ...s, lastSync: Date.now() }));
			} finally {
				update((s) => ({ ...s, isSyncing: false }));
			}
		},

		// Manual sync trigger
		async triggerSync() {
			await store.sync();
		}
	};

	return store;
}

export const refreshStore = createRefreshStore();
