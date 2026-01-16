// OBS Device and Source Configuration Types

/**
 * Display or audio device available in OBS
 * Retrieved via GetInputPropertiesListPropertyItems
 */
export interface ObsDevice {
	itemName: string; // Display name from OBS
	itemValue: string; // Unique identifier (display_uuid or device_id)
}

/**
 * OBS input source info
 */
export interface ObsInputInfo {
	inputName: string;
	inputKind: string;
	inputUuid?: string;
}

/**
 * Device type for configuration
 */
export type ObsDeviceType = 'display' | 'audio';

/**
 * Configured device/source mapping stored in app settings
 */
export interface ObsDeviceConfig {
	id: string;
	type: ObsDeviceType;
	name: string; // User-friendly name (shown in sidebar)
	required: boolean; // Show error if not available
	targetSourceName: string; // OBS source to assign to
	expectedValue: string; // Expected device_id or display_uuid
}

/**
 * Get the property name for a device type
 */
export function getPropertyNameForType(type: ObsDeviceType): string {
	return type === 'audio' ? 'device_id' : 'display_uuid';
}

/**
 * Browser source configuration with URL template
 */
export interface ObsBrowserSourceConfig {
	id: string;
	name: string; // User-friendly name
	targetSourceName: string; // OBS browser source name
	urlTemplate: string; // URL with ${textus} and ${lekcio} placeholders
}

/**
 * Current runtime status of a configured device
 */
export interface ObsDeviceStatus {
	configId: string;
	available: boolean; // Device exists in OBS property list
	assigned: boolean; // OBS source has correct setting
	lastChecked: number; // Timestamp
	error?: string;
}

/**
 * Current runtime status of a browser source
 */
export interface ObsBrowserSourceStatus {
	configId: string;
	currentUrl: string; // Current URL in OBS
	expectedUrl: string; // Expected URL based on event
	matches: boolean; // URLs match
	lastChecked: number;
	refreshPending: boolean; // Waiting for refresh confirmation
	refreshSuccess?: boolean; // Last refresh result
}

/**
 * All OBS device configurations (stored in app settings)
 */
export interface ObsDevicesSettings {
	devices: ObsDeviceConfig[];
	browserSources: ObsBrowserSourceConfig[];
}

/**
 * Default empty settings
 */
export const DEFAULT_OBS_DEVICES_SETTINGS: ObsDevicesSettings = {
	devices: [],
	browserSources: []
};

/**
 * Generate a unique ID for a new config
 */
export function generateConfigId(): string {
	return crypto.randomUUID();
}

/**
 * Interpolate URL template with event variables
 * Replaces ${textus} and ${lekcio} with actual values
 */
export function interpolateUrlTemplate(
	template: string,
	textus: string,
	lekcio: string
): string {
	return template
		.replace(/\$\{textus\}/g, encodeURIComponent(textus))
		.replace(/\$\{lekcio\}/g, encodeURIComponent(lekcio));
}

/**
 * Check if a URL template contains variables
 */
export function hasTemplateVariables(template: string): boolean {
	return template.includes('${textus}') || template.includes('${lekcio}');
}
