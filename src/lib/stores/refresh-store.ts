import { writable, get } from 'svelte/store';
import { eventStore, eventList } from './event-store';
import { youtubeApi } from '$lib/utils/youtube-api';
import { systemStore } from './system-store';
import type { YouTubePrivacyStatus, YouTubeLifeCycleStatus } from '$lib/types/event';
import { checkAllObsDevices } from '$lib/utils/obs-device-checker';
import { obsWebSocket } from '$lib/utils/obs-websocket';
import { eventSessionStore, currentSession } from './event-session-store';
import { uploadManager } from '$lib/services/upload/upload-manager';

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
			if (state.intervalId) {
				console.log('[RefreshStore] Already running, skipping start');
				return;
			}

			console.log(`[RefreshStore] Starting refresh interval (${REFRESH_INTERVAL / 1000}s)`);
			const id = setInterval(() => {
				console.log('[RefreshStore] Interval triggered, running sync...');
				store.sync();
			}, REFRESH_INTERVAL);
			update((s) => ({ ...s, intervalId: id }));

			// Run immediately on start
			console.log('[RefreshStore] Running initial sync...');
			store.sync();
		},

		// Stop the refresh interval
		stop() {
			update((s) => {
				if (s.intervalId) clearInterval(s.intervalId);
				return { ...s, intervalId: null };
			});
		},

		// Sync YouTube broadcasts with local events and check OBS devices
		async sync() {
			const $systemStore = get(systemStore);
			console.log('[RefreshStore] Sync started', { obs: $systemStore.obs, youtubeLoggedIn: $systemStore.youtubeLoggedIn });

			update((s) => ({ ...s, isSyncing: true }));

			try {
				// Check OBS connection - try to reconnect if not connected
				if (!$systemStore.obs) {
					console.log('[RefreshStore] OBS not connected, attempting to reconnect...');
					try {
						await obsWebSocket.autoconnect();
					} catch (error) {
						console.log('[RefreshStore] OBS reconnect failed (OBS may not be running):', error);
					}
				}

				// Check OBS devices if OBS is connected
				const updatedSystemStore = get(systemStore);
				if (updatedSystemStore.obs) {
					try {
						console.log('[RefreshStore] Checking OBS devices...');
						await checkAllObsDevices();
					} catch (error) {
						console.error('[RefreshStore] Failed to check OBS devices:', error);
					}
				}

				// Sync YouTube broadcasts if logged in
				if ($systemStore.youtubeLoggedIn) {
					const events = get(eventList);
					const scheduledEvents = events.filter((e) => e.youtubeScheduledId);
					console.log(`[RefreshStore] Syncing ${scheduledEvents.length} YouTube broadcasts...`);

					for (const event of scheduledEvents) {
						try {
							const broadcast = await youtubeApi.getBroadcast(event.youtubeScheduledId!);

							if (!broadcast) {
								// Broadcast no longer exists - remove ID
								console.log(`[RefreshStore] Broadcast ${event.youtubeScheduledId} no longer exists`);
								await eventStore.updateEvent(event.id, {
									youtubeScheduledId: undefined,
									youtubeLifeCycleStatus: undefined
								});
							} else {
								// Update status from YouTube
								console.log(`[RefreshStore] Broadcast ${event.youtubeScheduledId} status: ${broadcast.status.lifeCycleStatus}`);
								await eventStore.updateEvent(event.id, {
									youtubePrivacyStatus: broadcast.status
										.privacyStatus as YouTubePrivacyStatus,
									youtubeLifeCycleStatus: broadcast.status
										.lifeCycleStatus as YouTubeLifeCycleStatus
								});
							}
						} catch (error) {
							console.error(`[RefreshStore] Failed to sync event ${event.id}:`, error);
						}
					}
				}

				// Check for paused session and resume if OBS is connected
				const session = get(currentSession);
				if (session?.state === 'PAUSED' && updatedSystemStore.obs) {
					console.log('[RefreshStore] Resuming paused session');
					await eventSessionStore.resume();
				}

				// Check for pending uploads to resume
				try {
					const pendingUploads = await uploadManager.getPendingUploads();
					if (pendingUploads.length > 0) {
						console.log(`[RefreshStore] Found ${pendingUploads.length} pending uploads`);
						// Don't auto-resume here - let the user decide via UI
						// But we could notify them
					}
				} catch (error) {
					console.error('[RefreshStore] Failed to check pending uploads:', error);
				}

				update((s) => ({ ...s, lastSync: Date.now() }));
				console.log('[RefreshStore] Sync completed');
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
