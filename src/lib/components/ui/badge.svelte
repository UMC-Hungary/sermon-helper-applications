<script lang="ts">
	import { cn } from "$lib/utils";
	import type { HTMLAttributes } from "svelte/elements";

	type BadgeProps = HTMLAttributes<HTMLDivElement> & {
		variant?: "default" | "secondary" | "destructive" | "outline" | "success" | "warning";
	};

	export let variant: BadgeProps["variant"] = "default";
	export let className: string = "";

	const baseClasses = "inline-flex items-center rounded-full border px-2 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2";
	const variants = {
		default: "border-transparent bg-primary text-primary-foreground shadow hover:bg-primary/80",
		secondary: "border-transparent bg-secondary text-secondary-foreground hover:bg-secondary/80",
		destructive: "border-transparent bg-destructive text-white shadow hover:bg-destructive/80",
		outline: "text-foreground",
		success: "border-transparent bg-[rgb(34,197,94)] text-white",
		warning: "border-transparent bg-[rgb(251,146,60)] text-white",
	};

	$: variantClass = variants[variant || "default"];
	$: computedClass = cn(baseClasses, variantClass, className);
</script>

<div class={computedClass} {...$$restProps}>
	<slot />
</div>