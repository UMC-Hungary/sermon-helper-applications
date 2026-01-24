/**
 * Discovery server store for managing mDNS/DNS-SD service state.
 *
 * This store manages:
 * - Starting/stopping the discovery server
 * - Broadcasting system status updates to connected mobile clients
 * - Tracking server state and connected clients
 */

import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type {
	DiscoveryServerInfo,
	DiscoveryServerStatus,
	DiscoverySystemStatus,
	DiscoveryObsStatus,
	DiscoverySettings
} from '$lib/types/discovery';
import { DEFAULT_DISCOVERY_SETTINGS } from '$lib/types/discovery';

/** Current server status */
export const discoveryServerStatus = writable<DiscoveryServerStatus>({
	running: false,
	port: null,
	addresses: [],
	connectedClients: 0,
	mdnsRegistered: false
});

/** Whether the server is currently starting/stopping */
export const discoveryServerLoading = writable<boolean>(false);

/** Last error message */
export const discoveryServerError = writable<string | null>(null);

/** Derived store for whether the server is running */
export const isDiscoveryServerRunning = derived(
	discoveryServerStatus,
	($status) => $status.running
);

/** Discovery server manager class */
class DiscoveryServerManager {
	private eventUnlisten: (() => void) | null = null;

	/** Initialize the server manager and set up event listeners */
	async init(): Promise<void> {
		// Listen for server events from Tauri
		const unlistenStarted = await listen<DiscoveryServerInfo>(
			'discovery-server-started',
			(event) => {
				discoveryServerStatus.update((status) => ({
					...status,
					running: true,
					port: event.payload.port,
					addresses: event.payload.addresses,
					mdnsRegistered: true
				}));
				discoveryServerError.set(null);
			}
		);

		const unlistenStopped = await listen<void>('discovery-server-stopped', () => {
			discoveryServerStatus.update((status) => ({
				...status,
				running: false,
				port: null,
				connectedClients: 0,
				mdnsRegistered: false
			}));
		});

		this.eventUnlisten = () => {
			unlistenStarted();
			unlistenStopped();
		};

		// Get initial status
		await this.refreshStatus();
	}

	/** Refresh the server status from the backend */
	async refreshStatus(): Promise<void> {
		try {
			const status = await invoke<DiscoveryServerStatus>('get_discovery_server_status');
			discoveryServerStatus.set(status);
		} catch (error) {
			console.error('Failed to get discovery server status:', error);
		}
	}

	/** Start the discovery server */
	async start(settings: DiscoverySettings): Promise<DiscoveryServerInfo> {
		discoveryServerLoading.set(true);
		discoveryServerError.set(null);

		try {
			const info = await invoke<DiscoveryServerInfo>('start_discovery_server', {
				port: settings.port,
				authToken: settings.authRequired ? settings.authToken : null,
				instanceName: settings.instanceName
			});

			discoveryServerStatus.update((status) => ({
				...status,
				running: true,
				port: info.port,
				addresses: info.addresses,
				mdnsRegistered: true
			}));

			return info;
		} catch (error) {
			const message = error instanceof Error ? error.message : String(error);
			discoveryServerError.set(message);
			throw error;
		} finally {
			discoveryServerLoading.set(false);
		}
	}

	/** Stop the discovery server */
	async stop(): Promise<void> {
		discoveryServerLoading.set(true);
		discoveryServerError.set(null);

		try {
			await invoke('stop_discovery_server');
			discoveryServerStatus.update((status) => ({
				...status,
				running: false,
				port: null,
				connectedClients: 0,
				mdnsRegistered: false
			}));
		} catch (error) {
			const message = error instanceof Error ? error.message : String(error);
			discoveryServerError.set(message);
			throw error;
		} finally {
			discoveryServerLoading.set(false);
		}
	}

	/** Generate a new auth token */
	async generateAuthToken(): Promise<string> {
		return await invoke<string>('generate_discovery_auth_token');
	}

	/** Get local IP addresses */
	async getLocalAddresses(): Promise<string[]> {
		return await invoke<string[]>('get_local_ip_addresses');
	}

	/** Update the system status (broadcasts to connected mobile clients) */
	async updateSystemStatus(status: DiscoverySystemStatus): Promise<void> {
		try {
			await invoke('update_discovery_system_status', { status });
		} catch (error) {
			// Silently ignore errors (server might not be running)
			console.debug('Failed to update discovery system status:', error);
		}
	}

	/** Update the OBS status (broadcasts to connected mobile clients) */
	async updateObsStatus(status: DiscoveryObsStatus): Promise<void> {
		try {
			await invoke('update_discovery_obs_status', { status });
		} catch (error) {
			// Silently ignore errors (server might not be running)
			console.debug('Failed to update discovery OBS status:', error);
		}
	}

	/** Cleanup event listeners */
	destroy(): void {
		if (this.eventUnlisten) {
			this.eventUnlisten();
			this.eventUnlisten = null;
		}
	}
}

/** Singleton instance of the discovery server manager */
export const discoveryServerManager = new DiscoveryServerManager();
