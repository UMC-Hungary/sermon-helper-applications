import { derived, get } from 'svelte/store';
import { appSettings, appSettingsStore } from '$lib/utils/app-settings-store';
import type { ServiceEvent } from '$lib/types/event';
import { isEventToday, isEventUpcoming, sortEventsByDate } from '$lib/types/event';

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
};
