import { get } from 'svelte/store';
import { youtubeApi } from './youtube-api';
import { systemStore } from '$lib/stores/system-store';
import { eventStore } from '$lib/stores/event-store';
import { generateCalculatedTitle, type ServiceEvent } from '$lib/types/event';
import { toast } from './toast';

/**
 * Generate YouTube description from event data
 */
export function generateYoutubeDescription(event: ServiceEvent): string {
	const parts: string[] = [];
	if (event.speaker) parts.push(`Speaker: ${event.speaker}`);
	if (event.textus) parts.push(`Textus: ${event.textus}`);
	if (event.leckio) parts.push(`Lekció: ${event.leckio}`);
	if (event.description) {
		parts.push('');
		parts.push(event.description);
	}
	return parts.join('\n');
}

/**
 * Schedule a new YouTube broadcast for an event.
 * Handles loading state, API call, event update, and toast notifications.
 * Updates the event directly in the store.
 */
export async function scheduleYoutubeBroadcast(event: ServiceEvent): Promise<void> {
	if (!get(systemStore).youtubeLoggedIn) return;

	// Set scheduling state on the event
	await eventStore.updateEvent(event.id, { isBroadcastScheduling: true });

	try {
		const scheduledStartTime = new Date(
			`${event.date}T${event.time}:00`
		).toISOString();

		const broadcast = await youtubeApi.createBroadcast({
			title: generateCalculatedTitle(event),
			description: generateYoutubeDescription(event),
			scheduledStartTime,
			privacyStatus: event.youtubePrivacyStatus,
			enableDvr: true,
			enableEmbed: true
		});

		// Update event with broadcast ID and clear scheduling state
		await eventStore.updateEvent(event.id, {
			youtubeScheduledId: broadcast.id,
			isBroadcastScheduling: false
		});

		toast({
			title: 'YouTube Event Scheduled',
			description: 'The YouTube live event has been created successfully',
			variant: 'success'
		});
	} catch (error) {
		// Clear scheduling state on error
		await eventStore.updateEvent(event.id, { isBroadcastScheduling: false });

		toast({
			title: 'Error',
			description: error instanceof Error ? error.message : 'Failed to schedule YouTube event',
			variant: 'error'
		});
	}
}

/**
 * Update an existing YouTube broadcast with event data
 * Does nothing if not logged in or no broadcast ID
 * Throws on API errors
 */
export async function updateYoutubeBroadcast(event: ServiceEvent): Promise<void> {
	if (!get(systemStore).youtubeLoggedIn || !event.youtubeScheduledId) return;

	const scheduledStartTime = new Date(
		`${event.date}T${event.time || '10:00'}:00`
	).toISOString();

	await youtubeApi.updateBroadcast(event.youtubeScheduledId, {
		title: generateCalculatedTitle(event),
		description: generateYoutubeDescription(event),
		scheduledStartTime,
		privacyStatus: event.youtubePrivacyStatus,
		enableDvr: true,
		enableEmbed: true
	});
}
