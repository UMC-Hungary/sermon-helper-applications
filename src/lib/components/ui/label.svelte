<script lang="ts">
	import { cn } from "$lib/utils";
	import type { HTMLLabelAttributes } from "svelte/elements";

	type LabelProps = HTMLLabelAttributes;

	export let className: string = "";
	export let htmlFor: string | undefined = undefined;

	const baseClasses = "flex items-center gap-2 text-sm leading-none font-medium select-none group-data-[disabled=true]:pointer-events-none group-data-[disabled=true]:opacity-50 peer-disabled:cursor-not-allowed peer-disabled:opacity-50";
</script>

<label
	for={htmlFor}
	data-slot="label"
	class={cn(baseClasses, className)}
	{...$$restProps}
>
	<slot />
</label>
