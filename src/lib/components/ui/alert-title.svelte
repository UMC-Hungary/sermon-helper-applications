<script lang="ts">
	import { cn } from "$lib/utils";
	import type { HTMLAttributes } from "svelte/elements";

	type AlertTitleProps = HTMLAttributes<HTMLDivElement>;

	export let className: string = "";

	const baseClasses = "col-start-2 line-clamp-1 min-h-4 font-medium tracking-tight";
</script>

<div
	data-slot="alert-title"
	class={cn(baseClasses, className)}
	{...$$restProps}
>
	<slot />
</div>
