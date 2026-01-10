<script lang="ts">
	import { cn } from "$lib/utils";

	export let value: string = "";
	export let placeholder: string | undefined = undefined;
	export let disabled: boolean = false;
	export let className: string = "";

	const baseClasses = "placeholder:text-muted-foreground selection:bg-primary selection:text-primary-foreground dark:bg-input/30 border-input w-full min-w-0 rounded-md border bg-transparent px-3 py-2 text-base shadow-xs transition-[color,box-shadow] outline-none disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 md:text-sm focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px] aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive resize-none";
</script>

<textarea
	bind:value
	{placeholder}
	{disabled}
	data-slot="textarea"
	class={cn(baseClasses, className)}
	{...$$restProps}
></textarea>
