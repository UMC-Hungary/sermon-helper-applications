<script lang="ts">
	import { _ } from 'svelte-i18n';
	import { goto } from '$app/navigation';
	import EventForm from '$lib/components/event-form.svelte';
	import { toast } from '$lib/utils/toast';
	import { eventStore } from '$lib/stores/event-store';
	import { appSettings, appSettingsStore } from '$lib/utils/app-settings-store';
	import { systemStore } from '$lib/stores/system-store';
	import { scheduleYoutubeBroadcast } from '$lib/utils/youtube-helpers';
	import type { ServiceEvent } from '$lib/types/event';

	// Check for draft to restore
	const draft = $appSettings.draftEvent;
	const draftSaved = $appSettings.draftSaved;

	// Only use draft if it exists and wasn't saved (and no originalId means it's a new event draft)
	const initialEvent = (draft && !draftSaved && !$appSettings.draftEventOriginalId) ? draft : undefined;

	async function handleSave(event: ServiceEvent) {
		try {
			// Save the event first
			await eventStore.addEvent(event);

			// Try to auto-schedule on YouTube if logged in (this updates the event in the store)
			if ($systemStore.youtubeLoggedIn) {
				await scheduleYoutubeBroadcast(event);
			}

			// Clear draft and mark as saved
			await appSettingsStore.set('draftSaved', true);
			await appSettingsStore.set('draftEvent', null);
			await appSettingsStore.set('draftEventOriginalId', null);

			toast({
				title: $_('events.toasts.created.title'),
				description: $_('events.toasts.created.description'),
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
		<h2 class="text-3xl font-bold tracking-tight">{$_('events.addEvent')}</h2>
	</div>

	<EventForm
		event={initialEvent}
		onSave={handleSave}
		onCancel={handleCancel}
	/>
</div>
