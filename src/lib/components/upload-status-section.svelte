<script lang="ts">
	import { _ } from 'svelte-i18n';
	import Card from '$lib/components/ui/card.svelte';
	import EventSessionStatus from '$lib/components/event-session-status.svelte';
	import PendingUploadsList from '$lib/components/pending-uploads-list.svelte';
	import { currentSession } from '$lib/stores/event-session-store';
	import { hasPendingUploads } from '$lib/stores/pending-uploads-store';

	// Show section if there's an active session OR pending uploads
	$: showSection = $currentSession || $hasPendingUploads;
</script>

{#if showSection}
	<div class="space-y-3">
		<!-- Current session status (if active) - has its own Card -->
		{#if $currentSession}
			<EventSessionStatus />
		{/if}

		<!-- Pending uploads from past events -->
		{#if $hasPendingUploads}
			<Card className="p-3">
				<PendingUploadsList />
			</Card>
		{/if}
	</div>
{/if}
