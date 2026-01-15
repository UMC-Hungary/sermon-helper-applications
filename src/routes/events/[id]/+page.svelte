<script lang="ts">
	import { _ } from 'svelte-i18n';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import EventForm from '$lib/components/event-form.svelte';
	import { toast } from '$lib/utils/toast';
	import { eventStore, upcomingEvents, pastEvents } from '$lib/stores/event-store';
	import { appSettings, appSettingsStore } from '$lib/utils/app-settings-store';
	import { systemStore } from '$lib/stores/system-store';
	import { updateYoutubeBroadcast } from '$lib/utils/youtube-helpers';
	import type { ServiceEvent } from '$lib/types/event';

	// Get event ID from URL
	const eventId = page.params.id;

	// Find the event to edit
	const allEvents = [...$upcomingEvents, ...$pastEvents];
	const existingEvent = allEvents.find(e => e.id === eventId);

	// Check for draft to restore (only if editing the same event)
	const draft = $appSettings.draftEvent;
	const draftSaved = $appSettings.draftSaved;
	const draftOriginalId = $appSettings.draftEventOriginalId;

	// Use draft if it exists, wasn't saved, and is for this event
	const initialEvent = (draft && !draftSaved && draftOriginalId === eventId)
		? draft
		: existingEvent;

	// Redirect if event not found
	if (!existingEvent && !initialEvent) {
		goto('/events');
	}

	async function handleSave(event: ServiceEvent) {
		try {
			// Try to update YouTube broadcast if already scheduled and logged in
			if (event.youtubeScheduledId && $systemStore.youtubeLoggedIn) {
				try {
					await updateYoutubeBroadcast(event);
				} catch (ytError) {
					// Log error but continue with save
					console.error('YouTube update failed:', ytError);
					toast({
						title: $_('youtube.scheduling.updateFailed'),
						description: ytError instanceof Error ? ytError.message : 'Unknown error',
						variant: 'warning'
					});
				}
			}

			// Save the event
			await eventStore.updateEvent(event.id, event);

			// Clear draft and mark as saved
			await appSettingsStore.set('draftSaved', true);
			await appSettingsStore.set('draftEvent', null);
			await appSettingsStore.set('draftEventOriginalId', null);

			toast({
				title: $_('events.toasts.updated.title'),
				description: $_('events.toasts.updated.description'),
				variant: 'success'
			});

			// Navigate back to list
			goto('/events');
		} catch (error) {
			toast({
				title: $_('events.toasts.error.title'),
				description: error instanceof Error ? error.message : 'Unknown error',
				variant: 'error'
			});
		}
	}

	async function handleCancel() {
		// Clear draft
		await appSettingsStore.set('draftSaved', true);
		await appSettingsStore.set('draftEvent', null);
		await appSettingsStore.set('draftEventOriginalId', null);

		// Navigate back to list
		goto('/events');
	}
</script>

<div class="mt-12 lg:mt-0">
	<div class="mb-6">
		<h2 class="text-3xl font-bold tracking-tight">{$_('common.edit')}</h2>
	</div>

	{#if initialEvent}
		<EventForm
			event={initialEvent}
			originalEventId={eventId}
			onSave={handleSave}
			onCancel={handleCancel}
		/>
	{:else}
		<p class="text-muted-foreground">{$_('common.loading')}</p>
	{/if}
</div>
