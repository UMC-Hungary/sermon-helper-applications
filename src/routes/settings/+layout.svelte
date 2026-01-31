<script lang="ts">
	import { _ } from 'svelte-i18n';
	import SettingsNav from '$lib/components/settings-nav.svelte';
</script>

<div class="mt-12 lg:mt-0">
	<h2 class="text-3xl font-bold tracking-tight">{$_('settings.title')}</h2>
	<p class="text-muted-foreground">{$_('settings.subtitle')}</p>
</div>

<div class="mt-6">
	<SettingsNav />
	<slot />
</div>
