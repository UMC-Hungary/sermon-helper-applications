import { get } from 'svelte/store';
import { youtubeApi } from './youtube-api';
import { systemStore } from '$lib/stores/system-store';
import { generateCalculatedTitle, type ServiceEvent } from '$lib/types/event';

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
 * Schedule a new YouTube broadcast for an event
 * Returns the broadcast ID if successful, null if not logged in
 * Throws on API errors
 */
export async function scheduleYoutubeBroadcast(event: ServiceEvent): Promise<string | null> {
	if (!get(systemStore).youtubeLoggedIn) return null;

	const scheduledStartTime = new Date(
		`${event.date}T${event.time || '10:00'}:00`
	).toISOString();

	const broadcast = await youtubeApi.createBroadcast({
		title: generateCalculatedTitle(event),
		description: generateYoutubeDescription(event),
		scheduledStartTime,
		privacyStatus: event.youtubePrivacyStatus || 'public',
		enableDvr: true,
		enableEmbed: true
	});

	return broadcast.id;
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
		privacyStatus: event.youtubePrivacyStatus || 'public',
		enableDvr: true,
		enableEmbed: true
	});
}
