<script lang="ts">
	import { cn } from '$lib/utils.js';

	interface SelectProps {
		value?: string;
		disabled?: boolean;
		class?: string;
		children?: import('svelte').Snippet;
		id?: string;
		onValueChange?: (value: string) => void;
	}

	let { value = '', disabled = false, class: className, children, id, onValueChange }: SelectProps = $props();

	let isOpen = $state(false);
	let selectedValue = $state(value);

	function toggle() {
		if (disabled) return;
		isOpen = !isOpen;
	}

	function close() {
		isOpen = false;
	}

	function selectItem(val: string) {
		selectedValue = val;
		onValueChange?.(val);
		close();
	}

	// Close on outside click
	function handleClickOutside(event: MouseEvent) {
		const selectEl = document.getElementById(id || 'select-trigger');
		if (selectEl && !selectEl.contains(event.target as Node)) {
			close();
		}
	}

	$effect(() => {
		if (typeof window !== 'undefined') {
			if (isOpen) {
				document.addEventListener('click', handleClickOutside);
			} else {
				document.removeEventListener('click', handleClickOutside);
			}
		}
	});
</script>

<div class="relative">
	<button
		id={id || 'select-trigger'}
		type="button"
		{disabled}
		class={cn(
			'flex h-10 w-full items-center justify-between rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50',
			className
		)}
		onclick={toggle}
	>
		{@render children?.()}
		<svg
			class="h-4 w-4 opacity-50"
			fill="none"
			stroke="currentColor"
			viewBox="0 0 24 24"
			xmlns="http://www.w3.org/2000/svg"
		>
			<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
		</svg>
	</button>

	{#if isOpen}
		<div
			class="absolute top-0 left-0 z-50 w-full min-w-[8rem] overflow-hidden rounded-md border bg-popover text-popover-foreground shadow-md animate-in fade-in-0 zoom-in-95"
		>
			{@render children?.()}
		</div>
	{/if}
</div>