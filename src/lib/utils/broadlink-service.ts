/**
 * Broadlink Service
 * Frontend service for interacting with Broadlink devices via Tauri commands
 */

import { invoke } from '@tauri-apps/api/core';
import { rfIrStore, learnModeStore, rfIrSettings, rfIrDevices } from '$lib/stores/rf-ir-store';
import { get } from 'svelte/store';
import type { BroadlinkDevice, RfIrCommand } from '$lib/types/rf-ir';

/** Stored command format for discovery server */
interface StoredRfIrCommand {
	id: string;
	name: string;
	slug: string;
	deviceHost: string;
	deviceMac: string;
	deviceType: string;
	code: string;
	signalType: string;
	category: string;
}

export interface DiscoveredDevice {
	type: string;
	model: string;
	host: string;
	mac: string;
	name: string;
}

export interface LearnResult {
	code?: string;
	error?: string;
}

export interface SendResult {
	success: boolean;
	error?: string;
}

/**
 * Check if running in Tauri environment
 */
function isTauri(): boolean {
	return typeof window !== 'undefined' && '__TAURI__' in window;
}

export const broadlinkService = {
	/**
	 * Discover Broadlink devices on the network
	 */
	async discoverDevices(timeout: number = 5): Promise<DiscoveredDevice[]> {
		if (!isTauri()) {
			console.warn('Broadlink discovery requires Tauri environment');
			return [];
		}

		try {
			const devices = await invoke<DiscoveredDevice[]>('broadlink_discover', { timeout });
			return devices;
		} catch (error) {
			console.error('Failed to discover Broadlink devices:', error);
			throw error;
		}
	},

	/**
	 * Discover and add new devices to the store
	 */
	async discoverAndAddDevices(timeout: number = 5): Promise<{
		found: number;
		added: number;
		updated: number;
	}> {
		const discovered = await this.discoverDevices(timeout);
		const settings = get(rfIrSettings);
		const existingMacs = new Set(settings.devices.map((d) => d.mac.toLowerCase()));

		let added = 0;
		let updated = 0;

		for (const device of discovered) {
			const macLower = device.mac.toLowerCase();
			const existing = settings.devices.find((d) => d.mac.toLowerCase() === macLower);

			if (existing) {
				// Update existing device
				await rfIrStore.updateDevice(existing.id, {
					host: device.host,
					lastSeen: Date.now()
				});
				updated++;
			} else {
				// Add new device
				await rfIrStore.addDevice({
					name: device.name || device.model,
					type: device.type,
					model: device.model,
					host: device.host,
					mac: device.mac
				});
				added++;
			}
		}

		return { found: discovered.length, added, updated };
	},

	/**
	 * Start learning mode on a device
	 */
	async startLearning(deviceId: string, signalType: 'ir' | 'rf'): Promise<string> {
		if (!isTauri()) {
			throw new Error('Broadlink learning requires Tauri environment');
		}

		const device = rfIrStore.getDevice(deviceId);
		if (!device) {
			throw new Error('Device not found');
		}

		learnModeStore.start(deviceId, signalType);

		try {
			const result = await invoke<LearnResult>('broadlink_learn', {
				host: device.host,
				mac: device.mac,
				devtype: device.type,
				signalType
			});

			if (result.error) {
				learnModeStore.fail(result.error);
				throw new Error(result.error);
			}

			if (result.code) {
				learnModeStore.complete(result.code);
				return result.code;
			}

			throw new Error('No code received');
		} catch (error) {
			const message = error instanceof Error ? error.message : String(error);
			learnModeStore.fail(message);
			throw error;
		}
	},

	/**
	 * Cancel ongoing learning mode
	 */
	async cancelLearning(): Promise<void> {
		if (!isTauri()) {
			learnModeStore.reset();
			return;
		}

		try {
			await invoke('broadlink_cancel_learn');
		} finally {
			learnModeStore.reset();
		}
	},

	/**
	 * Send an IR/RF code to a device
	 */
	async sendCode(deviceId: string, code: string): Promise<void> {
		if (!isTauri()) {
			throw new Error('Broadlink sending requires Tauri environment');
		}

		const device = rfIrStore.getDevice(deviceId);
		if (!device) {
			throw new Error('Device not found');
		}

		try {
			const result = await invoke<SendResult>('broadlink_send', {
				host: device.host,
				mac: device.mac,
				devtype: device.type,
				code
			});

			if (!result.success) {
				throw new Error(result.error || 'Failed to send code');
			}

			// Update device last seen
			await rfIrStore.updateDeviceLastSeen(deviceId);
		} catch (error) {
			console.error('Failed to send code:', error);
			throw error;
		}
	},

	/**
	 * Execute a command by its object
	 */
	async executeCommand(command: RfIrCommand): Promise<void> {
		await this.sendCode(command.deviceId, command.code);
	},

	/**
	 * Execute a command by its slug
	 */
	async executeBySlug(slug: string): Promise<void> {
		const command = rfIrStore.getCommandBySlug(slug);
		if (!command) {
			throw new Error(`Command not found: ${slug}`);
		}
		await this.executeCommand(command);
	},

	/**
	 * Execute a command by its ID
	 */
	async executeById(id: string): Promise<void> {
		const command = rfIrStore.getCommand(id);
		if (!command) {
			throw new Error(`Command not found: ${id}`);
		}
		await this.executeCommand(command);
	},

	/**
	 * Test if a device is reachable
	 */
	async testDevice(deviceId: string): Promise<boolean> {
		if (!isTauri()) {
			return false;
		}

		const device = rfIrStore.getDevice(deviceId);
		if (!device) {
			return false;
		}

		try {
			const result = await invoke<boolean>('broadlink_test_device', {
				host: device.host,
				mac: device.mac,
				devtype: device.type
			});
			return result;
		} catch {
			return false;
		}
	},

	/**
	 * Learn a command and save it directly
	 */
	async learnAndSave(
		deviceId: string,
		signalType: 'ir' | 'rf',
		name: string,
		category: string
	): Promise<string> {
		const code = await this.startLearning(deviceId, signalType);

		const slug = await rfIrStore.addCommand({
			name,
			deviceId,
			code,
			type: signalType,
			category
		});

		// Sync to discovery server
		await this.syncCommandsToServer();

		return slug;
	},

	/**
	 * Sync RF/IR commands to the discovery server for API access
	 */
	async syncCommandsToServer(): Promise<void> {
		if (!isTauri()) {
			return;
		}

		const settings = get(rfIrSettings);
		const devices = get(rfIrDevices);

		// Convert commands to stored format with device info
		const storedCommands: StoredRfIrCommand[] = settings.commands.map((cmd) => {
			const device = devices.find((d) => d.id === cmd.deviceId);
			return {
				id: cmd.id,
				name: cmd.name,
				slug: cmd.slug,
				deviceHost: device?.host || '',
				deviceMac: device?.mac || '',
				deviceType: device?.type || '',
				code: cmd.code,
				signalType: cmd.type,
				category: cmd.category
			};
		});

		try {
			await invoke('update_discovery_rfir_commands', { commands: storedCommands });
		} catch (error) {
			console.error('Failed to sync RF/IR commands to server:', error);
		}
	}
};
