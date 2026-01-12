<script lang="ts">
	import { cn } from '$lib/utils.js';

	interface SelectContentProps {
		class?: string;
		children?: import('svelte').Snippet;
	}

	let { class: className, children }: SelectContentProps = $props();
</script>

<div class={cn('p-1', className)}>
	{@render children?.()}
</div>