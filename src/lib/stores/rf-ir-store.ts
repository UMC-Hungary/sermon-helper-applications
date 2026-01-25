/**
 * RF/IR Settings Store
 * Manages Broadlink devices and IR/RF commands
 */

import { writable, derived, get } from 'svelte/store';
import { appSettingsStore } from '$lib/utils/app-settings-store';
import { generateSlug } from '$lib/utils/slug';
import type {
	RfIrSettings,
	RfIrCommand,
	BroadlinkDevice,
	LearnModeState
} from '$lib/types/rf-ir';
import { DEFAULT_RF_IR_SETTINGS, DEFAULT_LEARN_MODE_STATE } from '$lib/types/rf-ir';

// Main settings store
export const rfIrSettings = writable<RfIrSettings>(DEFAULT_RF_IR_SETTINGS);

// Learn mode state
export const learnModeState = writable<LearnModeState>(DEFAULT_LEARN_MODE_STATE);

// Derived stores for convenience
export const rfIrCommands = derived(rfIrSettings, ($s) => $s.commands);
export const rfIrDevices = derived(rfIrSettings, ($s) => $s.devices);
export const rfIrEnabled = derived(rfIrSettings, ($s) => $s.enabled);

export const defaultDevice = derived(rfIrDevices, ($devices) =>
	$devices.find((d) => d.isDefault)
);

export const commandsByCategory = derived(rfIrCommands, ($commands) => {
	const grouped = new Map<string, RfIrCommand[]>();
	for (const cmd of $commands) {
		const category = cmd.category || 'uncategorized';
		if (!grouped.has(category)) grouped.set(category, []);
		grouped.get(category)!.push(cmd);
	}
	return grouped;
});

export const commandCategories = derived(rfIrCommands, ($commands) => {
	const categories = new Set<string>();
	for (const cmd of $commands) {
		categories.add(cmd.category || 'uncategorized');
	}
	return Array.from(categories).sort();
});

// Store operations
export const rfIrStore = {
	/**
	 * Load settings from persistent storage
	 */
	async load(): Promise<void> {
		const settings = await appSettingsStore.get('rfIrSettings');
		if (settings) {
			rfIrSettings.set(settings);
		}
	},

	/**
	 * Save current settings to persistent storage
	 */
	async persist(): Promise<void> {
		const settings = get(rfIrSettings);
		await appSettingsStore.set('rfIrSettings', settings);

		// Sync to discovery server if running
		try {
			const { broadlinkService } = await import('$lib/utils/broadlink-service');
			await broadlinkService.syncCommandsToServer();
		} catch (error) {
			// Ignore sync errors - server might not be running
		}
	},

	/**
	 * Update settings and persist
	 */
	async update(updates: Partial<RfIrSettings>): Promise<void> {
		rfIrSettings.update((s) => ({ ...s, ...updates }));
		await this.persist();
	},

	/**
	 * Enable or disable RF/IR functionality
	 */
	async setEnabled(enabled: boolean): Promise<void> {
		await this.update({ enabled });
	},

	// ==================== Device Operations ====================

	/**
	 * Add a new Broadlink device
	 */
	async addDevice(
		device: Omit<BroadlinkDevice, 'id' | 'lastSeen' | 'isDefault'>
	): Promise<string> {
		const id = crypto.randomUUID();
		const settings = get(rfIrSettings);
		const isFirst = settings.devices.length === 0;

		rfIrSettings.update((s) => ({
			...s,
			devices: [
				...s.devices,
				{
					...device,
					id,
					lastSeen: Date.now(),
					isDefault: isFirst // First device is default
				}
			]
		}));
		await this.persist();
		return id;
	},

	/**
	 * Update an existing device
	 */
	async updateDevice(id: string, updates: Partial<BroadlinkDevice>): Promise<void> {
		rfIrSettings.update((s) => ({
			...s,
			devices: s.devices.map((d) => (d.id === id ? { ...d, ...updates } : d))
		}));
		await this.persist();
	},

	/**
	 * Remove a device and its associated commands
	 */
	async removeDevice(id: string): Promise<void> {
		rfIrSettings.update((s) => {
			const devices = s.devices.filter((d) => d.id !== id);
			// If we removed the default, make the first remaining device default
			if (devices.length > 0 && !devices.some((d) => d.isDefault)) {
				devices[0].isDefault = true;
			}
			return {
				...s,
				devices,
				// Remove commands associated with this device
				commands: s.commands.filter((c) => c.deviceId !== id)
			};
		});
		await this.persist();
	},

	/**
	 * Set a device as the default
	 */
	async setDefaultDevice(id: string): Promise<void> {
		rfIrSettings.update((s) => ({
			...s,
			devices: s.devices.map((d) => ({ ...d, isDefault: d.id === id }))
		}));
		await this.persist();
	},

	/**
	 * Get a device by ID
	 */
	getDevice(id: string): BroadlinkDevice | undefined {
		const settings = get(rfIrSettings);
		return settings.devices.find((d) => d.id === id);
	},

	/**
	 * Update device last seen timestamp
	 */
	async updateDeviceLastSeen(id: string): Promise<void> {
		rfIrSettings.update((s) => ({
			...s,
			devices: s.devices.map((d) => (d.id === id ? { ...d, lastSeen: Date.now() } : d))
		}));
		await this.persist();
	},

	// ==================== Command Operations ====================

	/**
	 * Add a new RF/IR command
	 */
	async addCommand(
		command: Omit<RfIrCommand, 'id' | 'slug' | 'createdAt' | 'updatedAt'>
	): Promise<string> {
		const settings = get(rfIrSettings);
		const slug = generateSlug(
			command.name,
			settings.commands.map((c) => c.slug)
		);
		const now = Date.now();
		const id = crypto.randomUUID();

		rfIrSettings.update((s) => ({
			...s,
			commands: [
				...s.commands,
				{
					...command,
					id,
					slug,
					createdAt: now,
					updatedAt: now
				}
			]
		}));
		await this.persist();
		return slug;
	},

	/**
	 * Update an existing command
	 */
	async updateCommand(
		id: string,
		updates: Partial<Omit<RfIrCommand, 'id' | 'createdAt'>>
	): Promise<void> {
		rfIrSettings.update((s) => {
			const cmd = s.commands.find((c) => c.id === id);
			if (!cmd) return s;

			// Regenerate slug if name changed
			let newSlug = cmd.slug;
			if (updates.name && updates.name !== cmd.name) {
				const otherSlugs = s.commands.filter((c) => c.id !== id).map((c) => c.slug);
				newSlug = generateSlug(updates.name, otherSlugs);
			}

			return {
				...s,
				commands: s.commands.map((c) =>
					c.id === id
						? { ...c, ...updates, slug: updates.slug ?? newSlug, updatedAt: Date.now() }
						: c
				)
			};
		});
		await this.persist();
	},

	/**
	 * Remove a command
	 */
	async removeCommand(id: string): Promise<void> {
		rfIrSettings.update((s) => ({
			...s,
			commands: s.commands.filter((c) => c.id !== id)
		}));
		await this.persist();
	},

	/**
	 * Get a command by ID
	 */
	getCommand(id: string): RfIrCommand | undefined {
		const settings = get(rfIrSettings);
		return settings.commands.find((c) => c.id === id);
	},

	/**
	 * Get a command by slug
	 */
	getCommandBySlug(slug: string): RfIrCommand | undefined {
		const settings = get(rfIrSettings);
		return settings.commands.find((c) => c.slug === slug);
	},

	/**
	 * Get all commands for a specific device
	 */
	getCommandsByDevice(deviceId: string): RfIrCommand[] {
		const settings = get(rfIrSettings);
		return settings.commands.filter((c) => c.deviceId === deviceId);
	},

	/**
	 * Get all commands in a category
	 */
	getCommandsByCategory(category: string): RfIrCommand[] {
		const settings = get(rfIrSettings);
		return settings.commands.filter((c) => c.category === category);
	},

	// ==================== Bulk Operations ====================

	/**
	 * Import commands from external source (e.g., sermon-helper-service)
	 */
	async importCommands(
		commands: Array<Omit<RfIrCommand, 'id' | 'slug' | 'createdAt' | 'updatedAt'>>,
		options: { skipExisting?: boolean; replaceExisting?: boolean } = {}
	): Promise<{ imported: number; skipped: number; replaced: number }> {
		const settings = get(rfIrSettings);
		const existingSlugs = settings.commands.map((c) => c.slug);
		const now = Date.now();

		let imported = 0;
		let skipped = 0;
		let replaced = 0;

		const newCommands: RfIrCommand[] = [...settings.commands];

		for (const cmd of commands) {
			const slug = generateSlug(
				cmd.name,
				newCommands.map((c) => c.slug)
			);
			const existing = newCommands.find((c) =>
				c.name.toLowerCase() === cmd.name.toLowerCase()
			);

			if (existing) {
				if (options.replaceExisting) {
					const idx = newCommands.findIndex((c) => c.id === existing.id);
					newCommands[idx] = {
						...cmd,
						id: existing.id,
						slug: existing.slug,
						createdAt: existing.createdAt,
						updatedAt: now
					};
					replaced++;
				} else {
					skipped++;
				}
			} else {
				newCommands.push({
					...cmd,
					id: crypto.randomUUID(),
					slug,
					createdAt: now,
					updatedAt: now
				});
				imported++;
			}
		}

		rfIrSettings.update((s) => ({ ...s, commands: newCommands }));
		await this.persist();

		return { imported, skipped, replaced };
	},

	/**
	 * Export all commands for backup
	 */
	exportCommands(): RfIrCommand[] {
		const settings = get(rfIrSettings);
		return settings.commands;
	},

	/**
	 * Clear all commands
	 */
	async clearCommands(): Promise<void> {
		rfIrSettings.update((s) => ({ ...s, commands: [] }));
		await this.persist();
	},

	/**
	 * Clear all devices and commands
	 */
	async clearAll(): Promise<void> {
		rfIrSettings.set(DEFAULT_RF_IR_SETTINGS);
		await this.persist();
	}
};

// Learn mode operations
export const learnModeStore = {
	/**
	 * Start learning mode
	 */
	start(deviceId: string, type: 'ir' | 'rf'): void {
		learnModeState.set({
			active: true,
			deviceId,
			type,
			learnedCode: null,
			error: null
		});
	},

	/**
	 * Complete learning with code
	 */
	complete(code: string): void {
		learnModeState.update((s) => ({
			...s,
			active: false,
			learnedCode: code
		}));
	},

	/**
	 * Fail learning with error
	 */
	fail(error: string): void {
		learnModeState.update((s) => ({
			...s,
			active: false,
			error
		}));
	},

	/**
	 * Cancel/reset learning mode
	 */
	reset(): void {
		learnModeState.set(DEFAULT_LEARN_MODE_STATE);
	}
};
