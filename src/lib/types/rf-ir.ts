/**
 * RF/IR Remote Control Type Definitions
 * For Broadlink device integration
 */

export interface BroadlinkDevice {
	id: string;
	name: string;
	type: string; // Device type hex (e.g., "0x5f36")
	model: string; // Model name (e.g., "RM4 Pro")
	host: string; // IP address
	mac: string; // MAC address (format: aa:bb:cc:dd:ee:ff)
	lastSeen: number; // Timestamp
	isDefault: boolean;
}

export interface RfIrCommand {
	id: string;
	name: string;
	slug: string; // URL-safe identifier (e.g., "projector-power-on")
	deviceId: string; // Associated Broadlink device ID
	code: string; // Hex-encoded IR/RF signal data
	type: 'ir' | 'rf';
	category: string; // Category for grouping (e.g., "projector", "screen", "hvac")
	createdAt: number;
	updatedAt: number;
}

export interface RfIrSettings {
	enabled: boolean;
	autoDiscovery: boolean;
	discoveryTimeout: number; // Seconds (default: 5)
	devices: BroadlinkDevice[];
	commands: RfIrCommand[];
}

export interface LearnModeState {
	active: boolean;
	deviceId: string | null;
	type: 'ir' | 'rf' | null;
	learnedCode: string | null;
	error: string | null;
}

export const DEFAULT_RF_IR_SETTINGS: RfIrSettings = {
	enabled: false,
	autoDiscovery: true,
	discoveryTimeout: 5,
	devices: [],
	commands: []
};

export const DEFAULT_LEARN_MODE_STATE: LearnModeState = {
	active: false,
	deviceId: null,
	type: null,
	learnedCode: null,
	error: null
};

// Common categories for organizing commands
export const COMMAND_CATEGORIES = [
	'projector',
	'screen',
	'hvac',
	'lighting',
	'audio',
	'other'
] as const;

export type CommandCategory = (typeof COMMAND_CATEGORIES)[number];
