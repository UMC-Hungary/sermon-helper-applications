<script lang="ts">
	import { cn } from '$lib/utils.js';
	import type { LegacySuggestion } from '$lib/types/bible';
	import { Tag } from 'lucide-svelte';

	interface Props {
		suggestions: LegacySuggestion[];
		visible: boolean;
		onSelect: (suggestion: LegacySuggestion) => void;
		class?: string;
	}

	let { suggestions, visible, onSelect, class: className }: Props = $props();

	function handleSelect(suggestion: LegacySuggestion) {
		onSelect(suggestion);
	}

	function handleKeyDown(event: KeyboardEvent, suggestion: LegacySuggestion) {
		if (event.key === 'Enter' || event.key === ' ') {
			event.preventDefault();
			handleSelect(suggestion);
		}
	}
</script>

{#if visible && suggestions.length > 0}
	<div
		class={cn(
			'absolute top-full left-0 z-50 w-full mt-1 max-h-60 overflow-auto rounded-md border bg-popover text-popover-foreground shadow-md',
			className
		)}
		role="listbox"
	>
		<div class="px-2 py-1.5 text-xs font-semibold text-muted-foreground border-b">
			Suggestions:
		</div>
		<div class="p-1">
			{#each suggestions as suggestion, index (suggestion.link)}
				<button
					type="button"
					role="option"
					aria-selected={false}
					tabindex={0}
					class="relative flex w-full cursor-pointer select-none items-center rounded-sm py-2 px-2 text-sm outline-none hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground"
					onclick={() => handleSelect(suggestion)}
					onkeydown={(e) => handleKeyDown(e, suggestion)}
				>
					<Tag class="h-4 w-4 mr-2 shrink-0 text-muted-foreground" />
					<span class="truncate">{suggestion.label}</span>
				</button>
			{/each}
		</div>
	</div>
{/if}
