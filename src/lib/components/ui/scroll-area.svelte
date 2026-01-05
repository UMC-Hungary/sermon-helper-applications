<script lang="ts">
	import { cn } from "$lib/utils";
	import type { HTMLAttributes } from "svelte/elements";

	type ScrollAreaProps = HTMLAttributes<HTMLDivElement>;

	export let className: string = "";

	const baseClasses = "relative overflow-auto";
</script>

<div class={cn(baseClasses, className)} data-scroll-area>
	<slot />
</div>