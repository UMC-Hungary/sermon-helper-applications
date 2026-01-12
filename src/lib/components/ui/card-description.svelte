<script lang="ts">
	import { cn } from '$lib/utils.js';

	interface CardDescriptionProps {
		class?: string;
		children?: import('svelte').Snippet;
	}

	let { class: className, children }: CardDescriptionProps = $props();
</script>

<p class={cn('text-sm text-muted-foreground', className)}>
	{@render children?.()}
</p>