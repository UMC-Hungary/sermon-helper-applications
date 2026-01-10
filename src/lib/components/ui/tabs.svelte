<script lang="ts" context="module">
	import { writable } from 'svelte/store';

	export function createTabsContext(defaultValue: string) {
		return writable(defaultValue);
	}
</script>

<script lang="ts">
	import { cn } from "$lib/utils";
	import { setContext } from 'svelte';

	export let defaultValue: string = "";
	export let className: string = "";

	const activeTab = createTabsContext(defaultValue);
	setContext('tabs', activeTab);
</script>

<div data-slot="tabs" class={cn("space-y-6", className)}>
	<slot />
</div>
