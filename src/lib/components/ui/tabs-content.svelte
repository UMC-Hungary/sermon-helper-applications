<script lang="ts">
	import { cn } from "$lib/utils";
	import { getContext } from 'svelte';
	import type { Writable } from 'svelte/store';

	export let value: string;
	export let className: string = "";

	const activeTab = getContext<Writable<string>>('tabs');

	$: isActive = $activeTab === value;
</script>

{#if isActive}
	<div data-slot="tabs-content" class={cn("space-y-6", className)}>
		<slot />
	</div>
{/if}
