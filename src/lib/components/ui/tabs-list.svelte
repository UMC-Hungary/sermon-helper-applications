<script lang="ts">
	import { cn } from "$lib/utils";

	export let className: string = "";

	const baseClasses = "bg-muted text-muted-foreground inline-flex h-9 w-fit items-center justify-center rounded-lg p-1";
</script>

<div data-slot="tabs-list" class={cn(baseClasses, className)}>
	<slot />
</div>
