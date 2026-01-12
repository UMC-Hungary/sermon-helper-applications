<script lang="ts">
	import { cn } from '$lib/utils.js';

	interface SelectValueProps {
		placeholder?: string;
		class?: string;
		children?: import('svelte').Snippet;
	}

	let { placeholder, class: className, children }: SelectValueProps = $props();
</script>

<span class={cn('block truncate', className)}>
	{#if children}
		{@render children()}
	{:else if placeholder}
		{placeholder}
	{/if}
</span>