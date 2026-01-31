<script lang="ts">
	import RfIrSettings from '$lib/components/rf-ir-settings.svelte';
	import ObsDeviceConfigs from '$lib/components/obs-device-configs.svelte';
</script>

<div class="grid gap-6 lg:grid-cols-2">
	<div class="space-y-6">
		<RfIrSettings />
	</div>
	<div class="space-y-6">
		<ObsDeviceConfigs />
	</div>
</div>
