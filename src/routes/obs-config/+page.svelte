<script lang="ts">
	import { _ } from 'svelte-i18n';
	import ObsConnectionSettings from '$lib/components/obs-connection-settings.svelte';
	import ObsDeviceConfigs from '$lib/components/obs-device-configs.svelte';
</script>

<div class="mt-12 lg:mt-0">
	<h2 class="text-3xl font-bold tracking-tight">{$_('obsConfig.title')}</h2>
	<p class="text-muted-foreground">{$_('obsConfig.subtitle')}</p>
</div>

<div class="grid gap-6 lg:grid-cols-2 mt-6">
	<ObsConnectionSettings />
	<div class="space-y-6">
		<ObsDeviceConfigs />
	</div>
</div>
