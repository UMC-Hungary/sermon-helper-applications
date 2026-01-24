import { derived, get } from 'svelte/store';
import { appSettings, appSettingsStore } from '$lib/utils/app-settings-store';
import type { ServiceEvent, EventUploadSession } from '$lib/types/event';
import { isEventToday, isEventUpcoming, sortEventsByDate, hasPendingUploads } from '$lib/types/event';

// Derived store for event list from app settings (with fallback to empty array)
export const eventList = derived(appSettings, ($appSettings) => $appSettings?.eventList ?? []);

// Derived store for today's event
export const todayEvent = derived(eventList, ($eventList) => {
	const list = $eventList ?? [];
	return list.find(isEventToday) ?? null;
});

// Derived store for upcoming events (sorted)
export const upcomingEvents = derived(eventList, ($eventList) => {
	const list = $eventList ?? [];
	return sortEventsByDate(list.filter(isEventUpcoming));
});

// Derived store for past events (sorted, most recent first)
export const pastEvents = derived(eventList, ($eventList) => {
	const list = $eventList ?? [];
	const today = new Date().toISOString().split('T')[0];
	return sortEventsByDate(list.filter((e) => e.date < today)).reverse();
});

// Derived store for events with pending uploads (any event, past or future)
export const eventsWithPendingUploads = derived(eventList, ($eventList) => {
	const list = $eventList ?? [];
	return list.filter(hasPendingUploads);
});

// Flattened list of all pending upload sessions with their parent event info
export interface PendingUploadWithEvent {
	event: ServiceEvent;
	session: EventUploadSession;
}

export const allPendingUploads = derived(eventList, ($eventList) => {
	const list = $eventList ?? [];
	const pending: PendingUploadWithEvent[] = [];

	for (const event of list) {
		if (!event.uploadSessions) continue;
		for (const session of event.uploadSessions) {
			if (session.status === 'pending' || session.status === 'paused' || session.status === 'failed') {
				pending.push({ event, session });
			}
		}
	}

	// Sort by most recent first
	return pending.sort((a, b) => b.session.startedAt - a.session.startedAt);
});

// Event store operations
export const eventStore = {
	// Add a new event
	async addEvent(event: ServiceEvent): Promise<void> {
		const current = get(eventList) ?? [];
		const updated = [...current, event];
		await appSettingsStore.set('eventList', updated);
	},

	// Update an existing event
	async updateEvent(id: string, partial: Partial<ServiceEvent>): Promise<void> {
		const current = get(eventList) ?? [];
		const updated = current.map((event) =>
			event.id === id
				? { ...event, ...partial, updatedAt: new Date().toISOString() }
				: event
		);
		await appSettingsStore.set('eventList', updated);
	},

	// Delete an event
	async deleteEvent(id: string): Promise<void> {
		const current = get(eventList) ?? [];
		const updated = current.filter((event) => event.id !== id);
		await appSettingsStore.set('eventList', updated);
	},

	// Get event by ID
	getEventById(id: string): ServiceEvent | undefined {
		return (get(eventList) ?? []).find((event) => event.id === id);
	},

	// Get today's event
	getTodayEvent(): ServiceEvent | null {
		return get(todayEvent);
	},

	// Get upcoming events
	getUpcomingEvents(): ServiceEvent[] {
		return get(upcomingEvents);
	},

	// Check if there's an event for today
	hasTodayEvent(): boolean {
		return get(todayEvent) !== null;
	},

	// Add or update an upload session for an event
	async saveUploadSession(eventId: string, session: EventUploadSession): Promise<void> {
		const current = get(eventList) ?? [];
		const updated = current.map((event) => {
			if (event.id !== eventId) return event;

			const sessions = event.uploadSessions ?? [];
			const existingIndex = sessions.findIndex((s) => s.id === session.id);

			const updatedSessions =
				existingIndex >= 0
					? sessions.map((s, i) => (i === existingIndex ? session : s))
					: [...sessions, session];

			return {
				...event,
				uploadSessions: updatedSessions,
				updatedAt: new Date().toISOString()
			};
		});
		await appSettingsStore.set('eventList', updated);
	},

	// Remove an upload session from an event
	async removeUploadSession(eventId: string, sessionId: string): Promise<void> {
		const current = get(eventList) ?? [];
		const updated = current.map((event) => {
			if (event.id !== eventId) return event;

			const sessions = event.uploadSessions ?? [];
			return {
				...event,
				uploadSessions: sessions.filter((s) => s.id !== sessionId),
				updatedAt: new Date().toISOString()
			};
		});
		await appSettingsStore.set('eventList', updated);
	},

	// Get upload session by ID (searches all events)
	getUploadSession(sessionId: string): { event: ServiceEvent; session: EventUploadSession } | null {
		const list = get(eventList) ?? [];
		for (const event of list) {
			if (!event.uploadSessions) continue;
			const session = event.uploadSessions.find((s) => s.id === sessionId);
			if (session) {
				return { event, session };
			}
		}
		return null;
	},

	// Get all pending upload sessions across all events
	getAllPendingUploads(): PendingUploadWithEvent[] {
		return get(allPendingUploads);
	}
};
