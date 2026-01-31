<script lang="ts">
	import ImportExportSettings from '$lib/components/import-export-settings.svelte';
	import UpdateSettings from '$lib/components/update-settings.svelte';
</script>

<div class="grid gap-6 lg:grid-cols-2">
	<div class="space-y-6">
		<ImportExportSettings />
	</div>
	<div class="space-y-6">
		<UpdateSettings />
	</div>
</div>
