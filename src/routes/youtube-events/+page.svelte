<script lang="ts">
	import YoutubeEventsView from '$lib/components/youtube-events-view.svelte';
	import { systemStore } from '$lib/stores/system-store';

	const handleRecheck = () => {
		// Placeholder for system status recheck
	};
</script>

<YoutubeEventsView systemStatus={$systemStore} onRecheck={handleRecheck} />
