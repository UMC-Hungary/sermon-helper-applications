import { writable, derived } from 'svelte/store';
import { obsWebSocket } from '../utils/obs-websocket';

const youtubeLogin = writable(false);
const presentationApp = writable<string | null>(null);
const presentationConnected = writable(false);

// Main system store that derives OBS status from obsWebSocket reactive store
export const systemStore = derived(
	[youtubeLogin, obsWebSocket.obsStatus, presentationApp, presentationConnected],
	([$youtubeLoggedIn, $obsStatus, $presentationApp, $presentationConnected]) => ({
		obs: $obsStatus.connected,
		obsLoading: $obsStatus.loading,
		youtubeLoggedIn: $youtubeLoggedIn,
		presentationApp: $presentationApp,
		presentationConnected: $presentationConnected,
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
export const updatePresentationStatus = (app: string | null, connected: boolean) => {
	presentationApp.set(app);
	presentationConnected.set(connected);
};
export const reset = () => {
	youtubeLogin.set(false);
	presentationApp.set(null);
	presentationConnected.set(false);
};
