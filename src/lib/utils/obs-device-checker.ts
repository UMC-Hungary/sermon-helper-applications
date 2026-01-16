// OBS Device Checker Utility
// Validates device configurations against OBS and auto-assigns when possible

import { get } from 'svelte/store';
import { obsWebSocket } from './obs-websocket';
import { deviceConfigs, browserSourceConfigs } from '$lib/stores/obs-devices-store';
import {
	updateDeviceStatus,
	updateBrowserStatus,
	clearAllStatuses,
	isCheckingDevices,
	lastDeviceCheck,
	setBrowserRefreshPending,
	setBrowserRefreshSuccess
} from '$lib/stores/obs-device-status-store';
import { todayEvent, upcomingEvents } from '$lib/stores/event-store';
import type {
	ObsDeviceConfig,
	ObsBrowserSourceConfig,
	ObsDeviceStatus,
	ObsBrowserSourceStatus
} from '$lib/types/obs-devices';
import { getPropertyNameForType, interpolateUrlTemplate } from '$lib/types/obs-devices';

/**
 * Check all configured OBS devices and browser sources
 * Called every 5 minutes by the refresh store
 */
export async function checkAllObsDevices(): Promise<void> {
	// Check if OBS is connected
	if (!obsWebSocket.isConnected()) {
		console.log('OBS not connected, skipping device check');
		clearAllStatuses();
		return;
	}

	// Prevent concurrent checks
	if (get(isCheckingDevices)) {
		console.log('Device check already in progress, skipping');
		return;
	}

	isCheckingDevices.set(true);

	try {
		const devices = get(deviceConfigs);
		const browserSources = get(browserSourceConfigs);

		// Check all device configurations
		for (const device of devices) {
			try {
				await checkDevice(device);
			} catch (error) {
				console.error(`Failed to check device ${device.name}:`, error);
				updateDeviceStatus({
					configId: device.id,
					available: false,
					assigned: false,
					lastChecked: Date.now(),
					error: error instanceof Error ? error.message : 'Check failed'
				});
			}
		}

		// Check all browser source configurations
		for (const source of browserSources) {
			try {
				await checkBrowserSource(source);
			} catch (error) {
				console.error(`Failed to check browser source ${source.name}:`, error);
				updateBrowserStatus({
					configId: source.id,
					currentUrl: '',
					expectedUrl: '',
					matches: false,
					lastChecked: Date.now(),
					refreshPending: false
				});
			}
		}

		lastDeviceCheck.set(Date.now());
	} finally {
		isCheckingDevices.set(false);
	}
}

/**
 * Check a single device configuration
 */
async function checkDevice(config: ObsDeviceConfig): Promise<ObsDeviceStatus> {
	const propertyName = getPropertyNameForType(config.type);

	// Get current settings for the OBS source
	let currentSettings: Record<string, unknown>;
	try {
		currentSettings = await obsWebSocket.getInputSettings(config.targetSourceName);
	} catch (error) {
		// Source doesn't exist
		const status: ObsDeviceStatus = {
			configId: config.id,
			available: false,
			assigned: false,
			lastChecked: Date.now(),
			error: `OBS source "${config.targetSourceName}" not found`
		};
		updateDeviceStatus(status);
		return status;
	}

	const currentValue = currentSettings[propertyName] as string | undefined;

	// Get available devices from OBS
	let availableDevices;
	try {
		availableDevices = await obsWebSocket.getInputPropertyItems(
			config.targetSourceName,
			propertyName
		);
	} catch (error) {
		const status: ObsDeviceStatus = {
			configId: config.id,
			available: false,
			assigned: false,
			lastChecked: Date.now(),
			error: `Failed to get available devices: ${error instanceof Error ? error.message : 'Unknown'}`
		};
		updateDeviceStatus(status);
		return status;
	}

	// Check if the expected device is in the available list
	const deviceAvailable = availableDevices.some(
		(device) => device.itemValue === config.expectedValue
	);

	// Check if currently assigned correctly
	const isAssigned = currentValue === config.expectedValue;

	// If device is available but not assigned, auto-assign it
	if (deviceAvailable && !isAssigned) {
		try {
			await obsWebSocket.setInputSettings(config.targetSourceName, {
				[propertyName]: config.expectedValue
			});
			console.log(`Auto-assigned ${config.name} to ${config.targetSourceName}`);

			const status: ObsDeviceStatus = {
				configId: config.id,
				available: true,
				assigned: true,
				lastChecked: Date.now()
			};
			updateDeviceStatus(status);
			return status;
		} catch (error) {
			console.error(`Failed to auto-assign ${config.name}:`, error);
			const status: ObsDeviceStatus = {
				configId: config.id,
				available: true,
				assigned: false,
				lastChecked: Date.now(),
				error: `Auto-assign failed: ${error instanceof Error ? error.message : 'Unknown'}`
			};
			updateDeviceStatus(status);
			return status;
		}
	}

	// Device found and assigned correctly
	if (deviceAvailable && isAssigned) {
		const status: ObsDeviceStatus = {
			configId: config.id,
			available: true,
			assigned: true,
			lastChecked: Date.now()
		};
		updateDeviceStatus(status);
		return status;
	}

	// Device not available
	const status: ObsDeviceStatus = {
		configId: config.id,
		available: false,
		assigned: false,
		lastChecked: Date.now(),
		error: `Device "${config.expectedValue}" not found in available ${config.type}s`
	};
	updateDeviceStatus(status);
	return status;
}

/**
 * Check a single browser source configuration
 */
async function checkBrowserSource(
	config: ObsBrowserSourceConfig
): Promise<ObsBrowserSourceStatus> {
	// Get current settings for the OBS browser source
	let currentSettings: Record<string, unknown>;
	try {
		currentSettings = await obsWebSocket.getInputSettings(config.targetSourceName);
	} catch (error) {
		const status: ObsBrowserSourceStatus = {
			configId: config.id,
			currentUrl: '',
			expectedUrl: '',
			matches: false,
			lastChecked: Date.now(),
			refreshPending: false
		};
		updateBrowserStatus(status);
		return status;
	}

	const currentUrl = (currentSettings.url as string) || '';

	// Get event data for variable interpolation
	// First try today's event, then the first upcoming event
	const event = get(todayEvent) ?? get(upcomingEvents)[0];

	if (!event) {
		// No event to compare against - mark as matching (nothing to do)
		const status: ObsBrowserSourceStatus = {
			configId: config.id,
			currentUrl,
			expectedUrl: '',
			matches: true,
			lastChecked: Date.now(),
			refreshPending: false
		};
		updateBrowserStatus(status);
		return status;
	}

	// Interpolate the URL template with event data
	const expectedUrl = interpolateUrlTemplate(
		config.urlTemplate,
		event.textus || '',
		event.leckio || ''
	);

	// Compare URLs
	const matches = currentUrl === expectedUrl;

	// If URLs don't match, try to update the browser source
	if (!matches && expectedUrl) {
		try {
			await obsWebSocket.setInputSettings(config.targetSourceName, {
				url: expectedUrl
			});
			console.log(`Updated browser source ${config.name} URL`);

			// Wait a moment then verify
			await new Promise((resolve) => setTimeout(resolve, 500));

			// Re-fetch to verify
			const verifySettings = await obsWebSocket.getInputSettings(config.targetSourceName);
			const verifyUrl = (verifySettings.url as string) || '';
			const verifyMatches = verifyUrl === expectedUrl;

			const status: ObsBrowserSourceStatus = {
				configId: config.id,
				currentUrl: verifyUrl,
				expectedUrl,
				matches: verifyMatches,
				lastChecked: Date.now(),
				refreshPending: false,
				refreshSuccess: verifyMatches
			};
			updateBrowserStatus(status);
			return status;
		} catch (error) {
			console.error(`Failed to update browser source ${config.name}:`, error);
			const status: ObsBrowserSourceStatus = {
				configId: config.id,
				currentUrl,
				expectedUrl,
				matches: false,
				lastChecked: Date.now(),
				refreshPending: false,
				refreshSuccess: false
			};
			updateBrowserStatus(status);
			return status;
		}
	}

	// URLs match or no expected URL
	const status: ObsBrowserSourceStatus = {
		configId: config.id,
		currentUrl,
		expectedUrl,
		matches: true,
		lastChecked: Date.now(),
		refreshPending: false
	};
	updateBrowserStatus(status);
	return status;
}

/**
 * Manually refresh a browser source
 * Called when user clicks the refresh icon
 */
export async function manualRefreshBrowserSource(configId: string): Promise<boolean> {
	const configs = get(browserSourceConfigs);
	const config = configs.find((c) => c.id === configId);

	if (!config) {
		console.error('Browser source config not found:', configId);
		return false;
	}

	if (!obsWebSocket.isConnected()) {
		console.error('OBS not connected');
		return false;
	}

	setBrowserRefreshPending(configId, true);

	try {
		// Get event data
		const event = get(todayEvent) ?? get(upcomingEvents)[0];
		if (!event) {
			setBrowserRefreshSuccess(configId, false);
			return false;
		}

		// Calculate expected URL
		const expectedUrl = interpolateUrlTemplate(
			config.urlTemplate,
			event.textus || '',
			event.leckio || ''
		);

		// Update the browser source URL
		await obsWebSocket.setInputSettings(config.targetSourceName, {
			url: expectedUrl
		});

		// Wait a moment
		await new Promise((resolve) => setTimeout(resolve, 500));

		// Re-fetch and validate
		const currentSettings = await obsWebSocket.getInputSettings(config.targetSourceName);
		const currentUrl = (currentSettings.url as string) || '';
		const matches = currentUrl === expectedUrl;

		// Update status
		updateBrowserStatus({
			configId,
			currentUrl,
			expectedUrl,
			matches,
			lastChecked: Date.now(),
			refreshPending: false,
			refreshSuccess: matches
		});

		setBrowserRefreshSuccess(configId, matches);
		return matches;
	} catch (error) {
		console.error('Manual refresh failed:', error);
		setBrowserRefreshSuccess(configId, false);
		return false;
	}
}

/**
 * Force re-check a single device
 */
export async function recheckDevice(configId: string): Promise<void> {
	const configs = get(deviceConfigs);
	const config = configs.find((c) => c.id === configId);

	if (!config) {
		console.error('Device config not found:', configId);
		return;
	}

	if (!obsWebSocket.isConnected()) {
		console.error('OBS not connected');
		return;
	}

	try {
		await checkDevice(config);
	} catch (error) {
		console.error('Recheck failed:', error);
	}
}
