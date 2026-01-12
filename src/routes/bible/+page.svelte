<script lang="ts">
	import { _ } from 'svelte-i18n';
	import Card from '$lib/components/ui/card.svelte';
	import Button from '$lib/components/ui/button.svelte';
	import Input from '$lib/components/ui/input.svelte';
	import Label from '$lib/components/ui/label.svelte';
	import Textarea from '$lib/components/ui/textarea.svelte';
	import Badge from '$lib/components/ui/badge.svelte';
	import Tabs from '$lib/components/ui/tabs.svelte';
	import TabsList from '$lib/components/ui/tabs-list.svelte';
	import TabsTrigger from '$lib/components/ui/tabs-trigger.svelte';
	import TabsContent from '$lib/components/ui/tabs-content.svelte';
	import TranslationSelector from '$lib/components/translation-selector.svelte';
	import BibleSuggestions from '$lib/components/bible-suggestions.svelte';
	import { toast } from '$lib/utils/toast';
	import { bibleApi } from '$lib/utils/bible-api';
	import { bibleStore } from '$lib/stores/bible-store';
	import { isV2Translation, type BibleTranslation, type LegacySuggestion } from '$lib/types/bible';
	import { debounce } from '$lib/utils/debounce';
	import { Search, Save, Edit2, Check, Loader2, X } from 'lucide-svelte';

	// Tab type for type safety
	type TabType = 'textus' | 'leckio';

	// Subscribe to store
	const state = $derived($bibleStore);

	// Debounced function to fetch suggestions for legacy translations
	const debouncedFetchSuggestions = debounce(async (term: string, tab: TabType) => {
		if (term.length < 2) {
			bibleStore.clearSuggestions();
			return;
		}

		try {
			const suggestions = await bibleApi.fetchSuggestions(term);
			bibleStore.setSuggestions(suggestions, tab);
		} catch (error) {
			console.error('Failed to fetch suggestions:', error);
			bibleStore.clearSuggestions();
		}
	}, 300);

	// Debounced function to fetch verses for V2 translations (immediate)
	const debouncedFetchV2 = debounce(async (term: string, translation: BibleTranslation, tab: TabType) => {
		if (term.length < 2) {
			return;
		}

		bibleStore.setLoading(tab, true);

		try {
			const result = await bibleApi.fetchVerses(term, translation);
			bibleStore.setVerses(tab, result.verses, result.label);
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : 'Unknown error';
			bibleStore.setError(tab, errorMessage);
		}
	}, 500);

	// Handle input change
	function handleQueryInput(tab: TabType, value: string) {
		bibleStore.setQuery(tab, value);

		const translation = state[tab].translation;

		if (isV2Translation(translation)) {
			// V2: Fetch verses immediately as user types
			bibleStore.clearSuggestions();
			if (value.length >= 2) {
				debouncedFetchV2(value, translation, tab);
			}
		} else {
			// Legacy: Show suggestions for selection
			debouncedFetchSuggestions(value, tab);
		}
	}

	// Handle translation change
	function handleTranslationChange(tab: TabType, translation: BibleTranslation) {
		bibleStore.setTranslation(tab, translation);
		bibleStore.clearSuggestions();
		bibleStore.clearVerses(tab);

		// If there's already a query, re-fetch with new translation
		const query = state[tab].query;
		if (query.length >= 2) {
			if (isV2Translation(translation)) {
				debouncedFetchV2(query, translation, tab);
			} else {
				debouncedFetchSuggestions(query, tab);
			}
		}
	}

	// Handle suggestion selection (legacy translations)
	async function handleSuggestionSelect(tab: TabType, suggestion: LegacySuggestion) {
		bibleStore.setQuery(tab, suggestion.label);
		bibleStore.clearSuggestions();
		bibleStore.setLoading(tab, true);

		const translation = state[tab].translation;

		try {
			const result = await bibleApi.fetchLegacy(suggestion.link, translation);
			bibleStore.setVerses(tab, result.verses, result.label);

			toast({
				title: $_('bible.toasts.fetched.title'),
				description: $_('bible.toasts.fetched.description', {
					values: { count: result.verses.length, reference: result.label }
				}),
				variant: 'success'
			});
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : 'Unknown error';
			bibleStore.setError(tab, errorMessage);

			toast({
				title: $_('bible.toasts.error.title'),
				description: errorMessage,
				variant: 'error'
			});
		}
	}

	// Manual search/fetch (for button click or Enter key)
	async function handleSearch(tab: TabType) {
		const { query, translation } = state[tab];

		if (!query.trim()) {
			toast({
				title: $_('bible.toasts.error.title'),
				description: $_('bible.toasts.error.emptyQuery'),
				variant: 'warning'
			});
			return;
		}

		bibleStore.setLoading(tab, true);
		bibleStore.clearSuggestions();

		toast({
			title: $_('bible.toasts.fetching.title'),
			description: $_('bible.toasts.fetching.description', { values: { reference: query } }),
			variant: 'info'
		});

		try {
			const result = await bibleApi.fetchVerses(query, translation);
			bibleStore.setVerses(tab, result.verses, result.label);

			toast({
				title: $_('bible.toasts.fetched.title'),
				description: $_('bible.toasts.fetched.description', {
					values: { count: result.verses.length, reference: result.label }
				}),
				variant: 'success'
			});
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : 'Unknown error';
			bibleStore.setError(tab, errorMessage);

			toast({
				title: $_('bible.toasts.error.title'),
				description: errorMessage,
				variant: 'error'
			});
		}
	}

	// Handle Enter key in input
	function handleKeyDown(event: KeyboardEvent, tab: TabType) {
		if (event.key === 'Enter') {
			event.preventDefault();
			handleSearch(tab);
		} else if (event.key === 'Escape') {
			bibleStore.clearSuggestions();
		}
	}

	// Toggle verse editing
	function toggleEditing(tab: TabType, index: number) {
		bibleStore.toggleEditing(tab, index);
	}

	// Update verse text
	function handleVerseChange(tab: TabType, index: number, text: string) {
		bibleStore.updateVerse(tab, index, text);
	}

	// Clear verses
	function handleClear(tab: TabType) {
		bibleStore.clearVerses(tab);
		bibleStore.setQuery(tab, '');
	}

	// Save all changes
	function handleSave(tab: TabType) {
		// Finish all editing first
		bibleStore.finishAllEditing(tab);

		// TODO: Persist to Tauri store
		toast({
			title: $_('bible.toasts.saved.title'),
			description: $_('bible.toasts.saved.description'),
			variant: 'success'
		});
	}
</script>

<!-- Page Header -->
<div class="mt-12 lg:mt-0">
	<h2 class="text-3xl font-bold tracking-tight">{$_('bible.title')}</h2>
	<p class="text-muted-foreground">{$_('bible.subtitle')}</p>
</div>

<!-- Tabs -->
<Tabs defaultValue="textus">
	<TabsList className="grid w-full max-w-md grid-cols-2">
		<TabsTrigger value="textus">{$_('bible.tabs.textus')}</TabsTrigger>
		<TabsTrigger value="leckio">{$_('bible.tabs.leckio')}</TabsTrigger>
	</TabsList>

	{#each ['textus', 'leckio'] as tab}
		{@const tabState = state[tab as TabType]}
		{@const showSuggestions = state.showSuggestions && state.activeSuggestionTab === tab}

		<TabsContent value={tab}>
			<!-- Search Card -->
			<Card>
				<svelte:fragment slot="title">
					{$_(tab === 'textus' ? 'bible.search.titleTextus' : 'bible.search.titleLeckio')}
				</svelte:fragment>
				<svelte:fragment slot="description">{$_('bible.search.description')}</svelte:fragment>
				<svelte:fragment slot="content">
					<div class="flex gap-2 flex-wrap">
						<!-- Translation Selector -->
						<div class="w-36 space-y-2">
							<Label>{$_('bible.search.translation')}</Label>
							<TranslationSelector
								value={tabState.translation}
								onValueChange={(v) => handleTranslationChange(tab as TabType, v)}
								id={`${tab}-translation`}
							/>
						</div>

						<!-- Reference Input -->
						<div class="flex-1 min-w-[200px] space-y-2 relative">
							<Label for={`${tab}-search`}>{$_('bible.search.label')}</Label>
							<Input
								id={`${tab}-search`}
								value={tabState.query}
								oninput={(e: Event & { currentTarget: HTMLInputElement }) => handleQueryInput(tab as TabType, e.currentTarget.value)}
								onkeydown={(e: KeyboardEvent) => handleKeyDown(e, tab as TabType)}
								placeholder={$_(
									tab === 'textus'
										? 'bible.search.placeholderTextus'
										: 'bible.search.placeholderLeckio'
								)}
							/>
							<BibleSuggestions
								suggestions={state.suggestions}
								visible={showSuggestions}
								onSelect={(s) => handleSuggestionSelect(tab as TabType, s)}
							/>
						</div>

						<!-- Fetch Button -->
						<Button
							className="mt-auto"
							onclick={() => handleSearch(tab as TabType)}
							disabled={tabState.loading}
						>
							{#if tabState.loading}
								<Loader2 class="mr-2 h-4 w-4 animate-spin" />
							{:else}
								<Search class="mr-2 h-4 w-4" />
							{/if}
							{$_('bible.search.fetch')}
						</Button>
					</div>

					<!-- V2 translation hint -->
					{#if isV2Translation(tabState.translation)}
						<p class="mt-2 text-xs text-muted-foreground">
							{$_('bible.search.v2Hint')}
						</p>
					{/if}
				</svelte:fragment>
			</Card>

			<!-- Error Display -->
			{#if tabState.error}
				<Card className="border-destructive">
					<svelte:fragment slot="content">
						<div class="flex items-center gap-2 text-destructive">
							<X class="h-4 w-4" />
							<p class="text-sm">{tabState.error}</p>
						</div>
					</svelte:fragment>
				</Card>
			{/if}

			<!-- Verses Card -->
			{#if tabState.verses.length > 0}
				<Card>
					<svelte:fragment slot="header">
						<div class="flex items-center justify-between w-full">
							<div>
								<div class="leading-none font-semibold">
									{$_('bible.verses.title')} - {tabState.label}
								</div>
								<div class="text-muted-foreground text-sm">{$_('bible.verses.description')}</div>
							</div>
							<div class="flex gap-2">
								<Button buttonSize="sm" buttonVariant="outline" onclick={() => handleClear(tab as TabType)}>
									<X class="mr-2 h-4 w-4" />
									{$_('bible.verses.clear')}
								</Button>
								<Button buttonSize="sm" onclick={() => handleSave(tab as TabType)}>
									<Save class="mr-2 h-4 w-4" />
									{$_('bible.verses.saveAll')}
								</Button>
							</div>
						</div>
					</svelte:fragment>
					<svelte:fragment slot="content">
						<div class="space-y-1">
							{#each tabState.verses as verse, index (index)}
								<div
									class="flex gap-3 items-start p-2 hover:bg-accent/30 transition-colors {index !==
									tabState.verses.length - 1
										? 'border-b'
										: ''}"
								>
									<Badge variant="outline" className="shrink-0 mt-0.5 text-xs">
										{verse.chapter}:{verse.verse}
									</Badge>
									{#if verse.editing}
										<Textarea
											value={verse.text}
											oninput={(e: Event & { currentTarget: HTMLTextAreaElement }) =>
												handleVerseChange(tab as TabType, index, e.currentTarget.value)}
											className="flex-1 min-h-[60px] text-sm"
										/>
									{:else}
										<p class="flex-1 text-sm leading-relaxed">{verse.text}</p>
									{/if}
									<Button
										buttonVariant="ghost"
										buttonSize="icon"
										className="shrink-0 h-8 w-8"
										onclick={() => toggleEditing(tab as TabType, index)}
									>
										{#if verse.editing}
											<Check class="h-4 w-4" />
										{:else}
											<Edit2 class="h-4 w-4" />
										{/if}
									</Button>
								</div>
							{/each}
						</div>
					</svelte:fragment>
				</Card>
			{/if}
		</TabsContent>
	{/each}
</Tabs>
