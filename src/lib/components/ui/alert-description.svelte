<script lang="ts">
	import { cn } from "$lib/utils";
	import type { HTMLAttributes } from "svelte/elements";

	type AlertDescriptionProps = HTMLAttributes<HTMLDivElement>;

	export let className: string = "";

	const baseClasses = "text-muted-foreground col-start-2 grid justify-items-start gap-1 text-sm [&_p]:leading-relaxed";
</script>

<div
	data-slot="alert-description"
	class={cn(baseClasses, className)}
	{...$$restProps}
>
	<slot />
</div>
