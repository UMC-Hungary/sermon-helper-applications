<script lang="ts">
	import { cn } from '$lib/utils.js';

	interface SelectItemProps {
		value: string;
		disabled?: boolean;
		class?: string;
		children?: import('svelte').Snippet;
		onclick?: () => void;
	}

	let { value, disabled = false, class: className, children, onclick }: SelectItemProps = $props();

	function handleClick() {
		if (disabled) return;
		onclick?.();
	}
</script>

<div
	role="menuitem"
	class={cn(
		'relative flex cursor-default select-none items-center rounded-sm py-1.5 px-2 text-sm outline-none focus:bg-accent focus:text-accent-foreground data-[disabled]:pointer-events-none data-[disabled]:opacity-50',
		className,
		disabled && 'pointer-events-none opacity-50'
	)}
	data-disabled={disabled}
	onclick={handleClick}
>
	{@render children?.()}
</div>