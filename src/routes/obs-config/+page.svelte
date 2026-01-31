<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';

	onMount(() => {
		goto('/settings/connection', { replaceState: true });
	});
</script>

<div class="flex items-center justify-center min-h-[50vh]">
	<p class="text-muted-foreground">Redirecting to settings...</p>
</div>
