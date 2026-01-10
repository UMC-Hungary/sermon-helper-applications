<script lang="ts">
	import { cn } from "$lib/utils";
	import { getContext } from 'svelte';
	import type { Writable } from 'svelte/store';

	export let value: string;
	export let className: string = "";

	const activeTab = getContext<Writable<string>>('tabs');

	$: isActive = $activeTab === value;

	const baseClasses = "data-[state=active]:bg-background data-[state=active]:text-foreground focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:outline-ring inline-flex items-center justify-center gap-1.5 rounded-md px-2 py-1 text-sm font-medium whitespace-nowrap transition-[color,box-shadow] focus-visible:ring-[3px] focus-visible:outline-1 disabled:pointer-events-none disabled:opacity-50 data-[state=active]:shadow-sm [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4";

	function handleClick() {
		activeTab.set(value);
	}
</script>

<button
	type="button"
	data-slot="tabs-trigger"
	data-state={isActive ? "active" : "inactive"}
	class={cn(baseClasses, className)}
	onclick={handleClick}
>
	<slot />
</button>
