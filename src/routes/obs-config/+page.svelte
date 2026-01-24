<script lang="ts">
	import { _ } from 'svelte-i18n';
	import ObsConnectionSettings from '$lib/components/obs-connection-settings.svelte';
	import ObsDeviceConfigs from '$lib/components/obs-device-configs.svelte';
	import ImportExportSettings from '$lib/components/import-export-settings.svelte';
	import UpdateSettings from '$lib/components/update-settings.svelte';
	import DiscoverySettings from '$lib/components/discovery-settings.svelte';
</script>

<div class="mt-12 lg:mt-0">
	<h2 class="text-3xl font-bold tracking-tight">{$_('settings.title')}</h2>
	<p class="text-muted-foreground">{$_('settings.subtitle')}</p>
</div>

<div class="grid gap-6 lg:grid-cols-2 mt-6">
	<div class="space-y-6">
		<ObsConnectionSettings />
		<DiscoverySettings />
	</div>
	<div class="space-y-6">
		<ObsDeviceConfigs />
		<ImportExportSettings />
		<UpdateSettings />
	</div>
</div>
