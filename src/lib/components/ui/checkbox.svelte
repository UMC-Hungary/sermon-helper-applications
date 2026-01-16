<script lang="ts">
	import { cn } from '$lib/utils';

	interface Props {
		checked?: boolean;
		disabled?: boolean;
		onchange?: (event: Event) => void;
		className?: string;
	}

	let { checked = $bindable(false), disabled = false, onchange, className = '' }: Props = $props();
</script>

<input
	type="checkbox"
	{checked}
	{disabled}
	{onchange}
	class={cn(
		'h-4 w-4 rounded border border-primary text-primary ring-offset-background',
		'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2',
		'disabled:cursor-not-allowed disabled:opacity-50',
		'accent-primary',
		className
	)}
/>
