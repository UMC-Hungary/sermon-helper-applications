<script lang="ts">
	import EventSessionStatus from '$lib/components/event-session-status.svelte';
	import VideoUploader from '$lib/components/video-uploader.svelte';
	import { currentSession } from '$lib/stores/event-session-store';
	import { hasUploads } from '$lib/stores/unified-uploads-store';

	// Show section if there's an active session OR uploads in progress/pending
	$: showSection = $currentSession || $hasUploads;
</script>

{#if showSection}
	<div class="space-y-3">
		<!-- Current session status (OBS streaming/recording state) -->
		{#if $currentSession}
			<EventSessionStatus />
		{/if}

		<!-- Video Uploader (unified list of all uploads) -->
		<VideoUploader />
	</div>
{/if}
