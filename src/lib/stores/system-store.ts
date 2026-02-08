import { writable, derived } from 'svelte/store';
import { obsWebSocket } from '../utils/obs-websocket';

const youtubeLogin = writable(false);

// Main system store that derives OBS status from obsWebSocket reactive store
export const systemStore = derived(
	[youtubeLogin, obsWebSocket.obsStatus],
	([$youtubeLoggedIn, $obsStatus]) => ({
		obs: $obsStatus.connected,
		obsLoading: $obsStatus.loading,
		youtubeLoggedIn: $youtubeLoggedIn,
	})
);

// Derived stores for convenience
export const obsStatus = derived(obsWebSocket.obsStatus, $obsStatus => ({
	connected: $obsStatus.connected,
	loading: $obsStatus.loading,
	reconnecting: $obsStatus.reconnecting,
	lastConnected: $obsStatus.lastConnected,
	error: $obsStatus.error
}));

export const isSystemReady = derived(systemStore, $system => {
	// System is ready when critical components are working
	return $system.obs; // TODO: extend
});

export const updateYoutubeLogin = (status: boolean) => youtubeLogin.set(status);
export const reset = () => youtubeLogin.set(false);
