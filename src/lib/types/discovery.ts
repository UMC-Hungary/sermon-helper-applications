/**
 * Discovery server types for mDNS/DNS-SD service discovery
 * and mobile companion app integration.
 */

/** Discovery server info returned when starting the server */
export interface DiscoveryServerInfo {
	running: boolean;
	port: number;
	addresses: string[];
	serviceName: string;
	authRequired: boolean;
	/** URL to API documentation (Swagger UI) */
	docsUrl: string;
}

/** Full discovery server status */
export interface DiscoveryServerStatus {
	running: boolean;
	port: number | null;
	addresses: string[];
	connectedClients: number;
	mdnsRegistered: boolean;
	/** URL to API documentation (Swagger UI) */
	docsUrl: string | null;
}

/** System status sent to mobile clients */
export interface DiscoverySystemStatus {
	obsConnected: boolean;
	obsStreaming: boolean;
	obsRecording: boolean;
	rodeInterface: boolean;
	mainDisplay: boolean;
	secondaryDisplay: boolean;
	youtubeLoggedIn: boolean;
}

/** OBS-specific status for mobile clients */
export interface DiscoveryObsStatus {
	connected: boolean;
	streaming: boolean;
	recording: boolean;
	streamTimecode: string | null;
	recordTimecode: string | null;
}

/** Network interface with name and address */
export interface NetworkInterface {
	name: string;
	address: string;
	isPrimary: boolean;
}

/** Categorized network addresses */
export interface NetworkAddresses {
	/** Localhost addresses (127.0.0.1) - only accessible from this computer */
	localhost: string[];
	/** LAN addresses - accessible from devices on the same network */
	lan: NetworkInterface[];
	/** All addresses as flat list */
	all: string[];
}

/** Discovery settings stored in app settings */
export interface DiscoverySettings {
	enabled: boolean;
	autoStart: boolean;
	port: number;
	authToken: string | null;
	authRequired: boolean;
	instanceName: string;
}

/** Default discovery settings */
export const DEFAULT_DISCOVERY_SETTINGS: DiscoverySettings = {
	enabled: false,
	autoStart: false,
	port: 8765,
	authToken: null,
	authRequired: true,
	instanceName: 'Sermon Helper'
};
