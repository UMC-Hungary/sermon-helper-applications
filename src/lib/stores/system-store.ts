import { writable, derived } from 'svelte/store';
import type { SystemStatus } from './types';
import { obsWebSocket } from '../utils/obs-websocket';

// Create writable store for non-OBS system components only
function createNonObsSystemStore() {
	const { subscribe, set, update } = writable({
		rodeInterface: true,
		mainDisplay: true,
		secondaryDisplay: true,
		airplayDisplay: false,
		displayAlignment: false,
		youtubeLoggedIn: false,
	});

	return {
		subscribe,
		set,
		update,
		// Update methods for non-OBS components
		updateRodeInterface: (status: boolean) => {
			update(current => ({ ...current, rodeInterface: status }));
		},
		updateMainDisplay: (status: boolean) => {
			update(current => ({ ...current, mainDisplay: status }));
		},
		updateSecondaryDisplay: (status: boolean) => {
			update(current => ({ ...current, secondaryDisplay: status }));
		},
		updateAirplayDisplay: (status: boolean) => {
			update(current => ({ ...current, airplayDisplay: status }));
		},
		updateDisplayAlignment: (status: boolean) => {
			update(current => ({ ...current, displayAlignment: status }));
		},
		updateYoutubeLogin: (status: boolean) => {
			update(current => ({ ...current, youtubeLoggedIn: status }));
		},
		// Reset to defaults
		reset: () => {
			set({
				rodeInterface: true,
				mainDisplay: true,
				secondaryDisplay: true,
				airplayDisplay: false,
				displayAlignment: false,
				youtubeLoggedIn: false,
			});
		}
	};
}

const nonObsSystemStore = createNonObsSystemStore();

// Main system store that derives OBS status from obsWebSocket reactive store
export const systemStore = derived(
	[nonObsSystemStore, obsWebSocket.obsStatus],
	([$nonObs, $obsStatus]) => ({
		obs: $obsStatus.connected,
		obsLoading: $obsStatus.loading,
		rodeInterface: $nonObs.rodeInterface,
		mainDisplay: $nonObs.mainDisplay,
		secondaryDisplay: $nonObs.secondaryDisplay,
		airplayDisplay: $nonObs.airplayDisplay,
		displayAlignment: $nonObs.displayAlignment,
		youtubeLoggedIn: $nonObs.youtubeLoggedIn,
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
	return $system.obs && $system.rodeInterface && $system.mainDisplay;
});

// Export individual update methods for non-OBS components
export const { 
	updateRodeInterface,
	updateMainDisplay,
	updateSecondaryDisplay,
	updateAirplayDisplay,
	updateDisplayAlignment,
	updateYoutubeLogin,
	reset
} = nonObsSystemStore;