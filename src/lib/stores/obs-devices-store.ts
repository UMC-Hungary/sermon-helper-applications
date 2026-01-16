// OBS Devices Configuration Store
// Manages device and browser source configurations stored in app settings

import { derived, get } from 'svelte/store';
import { appSettings, appSettingsStore } from '$lib/utils/app-settings-store';
import type {
	ObsDeviceConfig,
	ObsBrowserSourceConfig,
	ObsDevicesSettings,
	ObsDeviceType
} from '$lib/types/obs-devices';
import {
	DEFAULT_OBS_DEVICES_SETTINGS,
	generateConfigId,
	getPropertyNameForType
} from '$lib/types/obs-devices';

// Derived store for OBS devices settings
export const obsDevicesSettings = derived(
	appSettings,
	($settings) => $settings?.obsDevicesSettings ?? DEFAULT_OBS_DEVICES_SETTINGS
);

// Derived store for device configs only
export const deviceConfigs = derived(
	obsDevicesSettings,
	($settings) => $settings.devices
);

// Derived store for browser source configs only
export const browserSourceConfigs = derived(
	obsDevicesSettings,
	($settings) => $settings.browserSources
);

// Derived store for required device configs (shown in sidebar)
export const requiredDeviceConfigs = derived(
	deviceConfigs,
	($devices) => $devices.filter((d) => d.required)
);

// Helper to save updated settings
async function saveSettings(settings: ObsDevicesSettings): Promise<void> {
	await appSettingsStore.set('obsDevicesSettings', settings);
}

// Device CRUD operations
export const obsDevicesStore = {
	// Add a new device configuration
	async addDevice(
		type: ObsDeviceType,
		name: string,
		targetSourceName: string,
		expectedValue: string,
		required: boolean = false
	): Promise<ObsDeviceConfig> {
		const currentSettings = get(obsDevicesSettings);
		const newDevice: ObsDeviceConfig = {
			id: generateConfigId(),
			type,
			name,
			required,
			targetSourceName,
			expectedValue
		};

		const updatedSettings: ObsDevicesSettings = {
			...currentSettings,
			devices: [...currentSettings.devices, newDevice]
		};

		await saveSettings(updatedSettings);
		return newDevice;
	},

	// Update an existing device configuration
	async updateDevice(id: string, updates: Partial<Omit<ObsDeviceConfig, 'id'>>): Promise<void> {
		const currentSettings = get(obsDevicesSettings);
		const updatedDevices = currentSettings.devices.map((device) =>
			device.id === id ? { ...device, ...updates } : device
		);

		await saveSettings({
			...currentSettings,
			devices: updatedDevices
		});
	},

	// Remove a device configuration
	async removeDevice(id: string): Promise<void> {
		const currentSettings = get(obsDevicesSettings);
		const updatedDevices = currentSettings.devices.filter((device) => device.id !== id);

		await saveSettings({
			...currentSettings,
			devices: updatedDevices
		});
	},

	// Add a new browser source configuration
	async addBrowserSource(
		name: string,
		targetSourceName: string,
		urlTemplate: string
	): Promise<ObsBrowserSourceConfig> {
		const currentSettings = get(obsDevicesSettings);
		const newSource: ObsBrowserSourceConfig = {
			id: generateConfigId(),
			name,
			targetSourceName,
			urlTemplate
		};

		const updatedSettings: ObsDevicesSettings = {
			...currentSettings,
			browserSources: [...currentSettings.browserSources, newSource]
		};

		await saveSettings(updatedSettings);
		return newSource;
	},

	// Update an existing browser source configuration
	async updateBrowserSource(
		id: string,
		updates: Partial<Omit<ObsBrowserSourceConfig, 'id'>>
	): Promise<void> {
		const currentSettings = get(obsDevicesSettings);
		const updatedSources = currentSettings.browserSources.map((source) =>
			source.id === id ? { ...source, ...updates } : source
		);

		await saveSettings({
			...currentSettings,
			browserSources: updatedSources
		});
	},

	// Remove a browser source configuration
	async removeBrowserSource(id: string): Promise<void> {
		const currentSettings = get(obsDevicesSettings);
		const updatedSources = currentSettings.browserSources.filter((source) => source.id !== id);

		await saveSettings({
			...currentSettings,
			browserSources: updatedSources
		});
	},

	// Get a device config by ID
	getDevice(id: string): ObsDeviceConfig | undefined {
		return get(deviceConfigs).find((d) => d.id === id);
	},

	// Get a browser source config by ID
	getBrowserSource(id: string): ObsBrowserSourceConfig | undefined {
		return get(browserSourceConfigs).find((s) => s.id === id);
	},

	// Get property name for a device type
	getPropertyName(type: ObsDeviceType): string {
		return getPropertyNameForType(type);
	}
};
