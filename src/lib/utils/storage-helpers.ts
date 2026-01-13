import { load } from '@tauri-apps/plugin-store';

// Storage interface that all store implementations must follow
export interface StorageBackend {
	get(key: string): Promise<any>;
	set(key: string, value: any): Promise<void>;
	save(): Promise<void>;
}

// Check if running in Tauri environment
export function isTauriApp(): boolean {
	return (
		typeof window !== 'undefined' &&
		// @ts-ignore - Tauri internal property
		typeof (window as any).__TAURI_INTERNALS__ !== 'undefined'
	);
}

// Check if localStorage is available
export function isLocalStorageAvailable(): boolean {
	return typeof window !== 'undefined' && typeof window.localStorage !== 'undefined';
}

// LocalStorage fallback for browser development
export class LocalStorageStore implements StorageBackend {
	constructor(private storeName: string) {}

	async get(key: string): Promise<any> {
		if (!isLocalStorageAvailable()) {
			return null;
		}
		const value = localStorage.getItem(`${this.storeName}_${key}`);
		return value ? JSON.parse(value) : null;
	}

	async set(key: string, value: any): Promise<void> {
		if (!isLocalStorageAvailable()) {
			return;
		}
		localStorage.setItem(`${this.storeName}_${key}`, JSON.stringify(value));
	}

	async save(): Promise<void> {
		// localStorage is auto-saving
	}
}

// In-memory fallback for SSR or when localStorage is not available
export class MemoryStore implements StorageBackend {
	private data: Record<string, any> = {};

	constructor(private storeName: string) {}

	async get(key: string): Promise<any> {
		return this.data[key] ?? null;
	}

	async set(key: string, value: any): Promise<void> {
		this.data[key] = value;
	}

	async save(): Promise<void> {
		// In-memory, no save needed
	}
}

// Factory function to create the appropriate store backend
export async function createStorageBackend(storeName: string): Promise<StorageBackend> {
	if (isTauriApp()) {
		try {
			return await load(storeName);
		} catch (error) {
			console.warn(`Failed to initialize Tauri store for ${storeName}, using fallback:`, error);
		}
	}

	// Browser environment - use localStorage if available, otherwise memory store
	if (isLocalStorageAvailable()) {
		return new LocalStorageStore(storeName);
	}

	return new MemoryStore(storeName);
}
