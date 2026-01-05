<script lang="ts">
	import { cn } from "$lib/utils";
	import type { HTMLAttributes } from "svelte/elements";

	type AlertProps = HTMLAttributes<HTMLDivElement> & {
		variant?: "default" | "destructive";
	};

	export let variant: AlertProps["variant"] = "default";
	export let className: string = "";

	const baseClasses = "relative w-full rounded-lg border px-4 py-3 text-sm grid has-[>svg]:grid-cols-[calc(var(--spacing)*4)_1fr] grid-cols-[0_1fr] has-[>svg]:gap-x-3 gap-y-0.5 items-start [&>svg]:size-4 [&>svg]:translate-y-0.5 [&>svg]:text-current";
	const variantClasses = {
		default: "bg-card text-card-foreground",
		destructive: "text-destructive bg-card [&>svg]:text-current *:data-[slot=alert-description]:text-destructive/90"
	};

	$: variantClass = variantClasses[variant || "default"];
</script>

<div
	data-slot="alert"
	class={cn(baseClasses, variantClass, className)}
	role="alert"
	{...$$restProps}
>
	<slot />
</div>
