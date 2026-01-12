<script lang="ts">
	import { cn } from '$lib/utils.js';
	import { _ } from 'svelte-i18n';
	import { TRANSLATIONS, type BibleTranslation, type TranslationOption } from '$lib/types/bible';
	import { ChevronDown, Check } from 'lucide-svelte';

	interface Props {
		value: BibleTranslation;
		onValueChange: (value: BibleTranslation) => void;
		class?: string;
		id?: string;
	}

	let { value, onValueChange, class: className, id = 'translation-selector' }: Props = $props();

	let isOpen = $state(false);

	function toggle() {
		isOpen = !isOpen;
	}

	function close() {
		isOpen = false;
	}

	function selectTranslation(code: BibleTranslation) {
		onValueChange(code);
		close();
	}

	function handleClickOutside(event: MouseEvent) {
		const selectEl = document.getElementById(id);
		if (selectEl && !selectEl.contains(event.target as Node)) {
			close();
		}
	}

	$effect(() => {
		if (typeof window !== 'undefined') {
			if (isOpen) {
				document.addEventListener('click', handleClickOutside);
				return () => document.removeEventListener('click', handleClickOutside);
			}
		}
	});

	const currentTranslation = $derived(TRANSLATIONS.find(t => t.code === value));
	const v2Translations = $derived(TRANSLATIONS.filter(t => t.type === 'v2'));
	const legacyTranslations = $derived(TRANSLATIONS.filter(t => t.type === 'legacy'));
</script>

<div {id} class={cn('relative', className)}>
	<button
		type="button"
		class="flex h-10 w-full items-center justify-between rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2"
		onclick={toggle}
	>
		<span class="truncate">{currentTranslation?.name || value}</span>
		<ChevronDown class="h-4 w-4 opacity-50 ml-2 shrink-0" />
	</button>

	{#if isOpen}
		<div
			class="absolute top-full left-0 z-50 w-full min-w-[14rem] mt-1 overflow-hidden rounded-md border bg-popover text-popover-foreground shadow-md"
		>
			<!-- V2 Translations Section -->
			<div class="px-2 py-1.5 text-xs font-semibold text-muted-foreground border-b">
				{$_('bible.translations.v2Label')}
			</div>
			<div class="p-1">
				{#each v2Translations as translation}
					<button
						type="button"
						class={cn(
							'relative flex w-full cursor-pointer select-none items-center rounded-sm py-1.5 px-2 text-sm outline-none hover:bg-accent hover:text-accent-foreground',
							value === translation.code && 'bg-accent'
						)}
						onclick={() => selectTranslation(translation.code)}
					>
						{#if value === translation.code}
							<Check class="h-4 w-4 mr-2 shrink-0" />
						{:else}
							<span class="w-4 h-4 mr-2"></span>
						{/if}
						<span class="truncate">{translation.name}</span>
					</button>
				{/each}
			</div>

			<!-- Legacy Translations Section -->
			<div class="px-2 py-1.5 text-xs font-semibold text-muted-foreground border-t border-b">
				{$_('bible.translations.legacyLabel')}
			</div>
			<div class="p-1">
				{#each legacyTranslations as translation}
					<button
						type="button"
						class={cn(
							'relative flex w-full cursor-pointer select-none items-center rounded-sm py-1.5 px-2 text-sm outline-none hover:bg-accent hover:text-accent-foreground',
							value === translation.code && 'bg-accent'
						)}
						onclick={() => selectTranslation(translation.code)}
					>
						{#if value === translation.code}
							<Check class="h-4 w-4 mr-2 shrink-0" />
						{:else}
							<span class="w-4 h-4 mr-2"></span>
						{/if}
						<span class="truncate">{translation.name}</span>
					</button>
				{/each}
			</div>
		</div>
	{/if}
</div>
