// OBS Device Status Store
// Runtime status tracking for device and browser source configurations
// This data is NOT persisted - it's recalculated every 5 minutes

import { writable, derived, get } from 'svelte/store';
import type { ObsDeviceStatus, ObsBrowserSourceStatus } from '$lib/types/obs-devices';
import { deviceConfigs, browserSourceConfigs } from './obs-devices-store';

// Runtime status maps (keyed by config ID)
export const obsDeviceStatuses = writable<Map<string, ObsDeviceStatus>>(new Map());
export const obsBrowserStatuses = writable<Map<string, ObsBrowserSourceStatus>>(new Map());

// Last time devices were checked
export const lastDeviceCheck = writable<number | null>(null);

// Whether a device check is currently in progress
export const isCheckingDevices = writable<boolean>(false);

// Update a single device status
export function updateDeviceStatus(status: ObsDeviceStatus): void {
	obsDeviceStatuses.update((map) => {
		const newMap = new Map(map);
		newMap.set(status.configId, status);
		return newMap;
	});
}

// Update a single browser source status
export function updateBrowserStatus(status: ObsBrowserSourceStatus): void {
	obsBrowserStatuses.update((map) => {
		const newMap = new Map(map);
		newMap.set(status.configId, status);
		return newMap;
	});
}

// Clear all statuses (e.g., when OBS disconnects)
export function clearAllStatuses(): void {
	obsDeviceStatuses.set(new Map());
	obsBrowserStatuses.set(new Map());
	lastDeviceCheck.set(null);
}

// Get status for a specific device
export function getDeviceStatus(configId: string): ObsDeviceStatus | undefined {
	return get(obsDeviceStatuses).get(configId);
}

// Get status for a specific browser source
export function getBrowserStatus(configId: string): ObsBrowserSourceStatus | undefined {
	return get(obsBrowserStatuses).get(configId);
}

// Derived: All required devices that are currently failing (not available)
export const failingRequiredDevices = derived(
	[obsDeviceStatuses, deviceConfigs],
	([$statuses, $configs]) => {
		return $configs.filter((config) => {
			if (!config.required) return false;
			const status = $statuses.get(config.id);
			// If no status yet, don't count as failing
			if (!status) return false;
			return !status.available;
		});
	}
);

// Derived: All required devices that are available and properly assigned
export const healthyRequiredDevices = derived(
	[obsDeviceStatuses, deviceConfigs],
	([$statuses, $configs]) => {
		return $configs.filter((config) => {
			if (!config.required) return false;
			const status = $statuses.get(config.id);
			if (!status) return false;
			return status.available && status.assigned;
		});
	}
);

// Derived: Browser sources that have URL mismatches
export const mismatchedBrowserSources = derived(
	[obsBrowserStatuses, browserSourceConfigs],
	([$statuses, $configs]) => {
		return $configs.filter((config) => {
			const status = $statuses.get(config.id);
			if (!status) return false;
			return !status.matches;
		});
	}
);

// Derived: Browser sources with pending refreshes
export const pendingBrowserRefreshes = derived(obsBrowserStatuses, ($statuses) => {
	const pending: ObsBrowserSourceStatus[] = [];
	$statuses.forEach((status) => {
		if (status.refreshPending) {
			pending.push(status);
		}
	});
	return pending;
});

// Set refresh pending for a browser source
export function setBrowserRefreshPending(configId: string, pending: boolean): void {
	obsBrowserStatuses.update((map) => {
		const status = map.get(configId);
		if (status) {
			const newMap = new Map(map);
			newMap.set(configId, {
				...status,
				refreshPending: pending,
				refreshSuccess: pending ? undefined : status.refreshSuccess
			});
			return newMap;
		}
		return map;
	});
}

// Set refresh success for a browser source
export function setBrowserRefreshSuccess(configId: string, success: boolean): void {
	obsBrowserStatuses.update((map) => {
		const status = map.get(configId);
		if (status) {
			const newMap = new Map(map);
			newMap.set(configId, {
				...status,
				refreshPending: false,
				refreshSuccess: success
			});
			return newMap;
		}
		return map;
	});
}
