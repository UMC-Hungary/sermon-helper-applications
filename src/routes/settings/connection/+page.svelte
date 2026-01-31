<script lang="ts">
	import ObsConnectionSettings from '$lib/components/obs-connection-settings.svelte';
	import DiscoverySettings from '$lib/components/discovery-settings.svelte';
</script>

<div class="grid gap-6 lg:grid-cols-2">
	<div class="space-y-6">
		<ObsConnectionSettings />
	</div>
	<div class="space-y-6">
		<DiscoverySettings />
	</div>
</div>
