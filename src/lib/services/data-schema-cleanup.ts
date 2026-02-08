// Data Schema Cleanup Service
// Removes outdated data on startup to ensure clean schema
// Future: can add more cleanup tasks as schema evolves

import { CURRENT_EVENT_VERSION, type ServiceEvent } from '$lib/types/event';
import { appSettingsStore } from '$lib/utils/app-settings-store';

class DataSchemaCleanupService {
	private hasRun = false;

	async runCleanup(): Promise<void> {
		if (this.hasRun) {
			console.log('[DataSchemaCleanup] Already ran this session, skipping');
			return;
		}

		console.log('[DataSchemaCleanup] Starting cleanup...');

		await this.cleanupOutdatedEvents();
		await this.recoverStuckSessions();

		this.hasRun = true;
		console.log('[DataSchemaCleanup] Cleanup complete');
	}

	// Recover events stuck in intermediate states (e.g. FINALIZING after a crash/refresh)
	private async recoverStuckSessions(): Promise<void> {
		const events: ServiceEvent[] = (await appSettingsStore.get('eventList')) || [];

		let modified = false;
		const recovered = events.map((event) => {
			if (event.sessionState === 'FINALIZING') {
				console.log(`[DataSchemaCleanup] Recovering stuck FINALIZING session for event: ${event.id}`);
				modified = true;
				return {
					...event,
					sessionState: 'ACTIVE' as const,
					sessionCompletionError: 'Session interrupted — automation was reset on restart'
				};
			}
			return event;
		});

		if (modified) {
			await appSettingsStore.set('eventList', recovered);
			console.log('[DataSchemaCleanup] Recovered stuck sessions');
		}
	}

	private async cleanupOutdatedEvents(): Promise<void> {
		const events = (await appSettingsStore.get('eventList')) || [];

		// Keep events that have the current version
		const validEvents = events.filter(
			(event: ServiceEvent) => event.version === CURRENT_EVENT_VERSION
		);

		const removedCount = events.length - validEvents.length;
		if (removedCount > 0) {
			console.log(`[DataSchemaCleanup] Removed ${removedCount} outdated events (missing version ${CURRENT_EVENT_VERSION})`);
			await appSettingsStore.set('eventList', validEvents);
		} else {
			console.log('[DataSchemaCleanup] No outdated events to remove');
		}
	}
}

export const dataSchemaCleanup = new DataSchemaCleanupService();
