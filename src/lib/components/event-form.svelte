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
	import { isV2Translation, type BibleTranslation, type BibleVerse, type LegacySuggestion } from '$lib/types/bible';
	import { debounce } from '$lib/utils/debounce';
	import { createEmptyEvent, generateCalculatedTitle, type ServiceEvent, type YouTubePrivacyStatus } from '$lib/types/event';
	import { appSettingsStore } from '$lib/utils/app-settings-store';
	import { Search, Save, X, Loader2, BookOpen, Edit2, Check, Youtube, ExternalLink } from 'lucide-svelte';
	import { systemStore } from '$lib/stores/system-store';
	import { youtubeApi } from '$lib/utils/youtube-api';
	import YouTubeLoginModal from '$lib/components/youtube-login-modal.svelte';

	interface Props {
		event?: ServiceEvent;
		originalEventId?: string; // Set if editing an existing event
		onSave: (event: ServiceEvent) => void;
		onCancel: () => void;
	}

	let { event, originalEventId, onSave, onCancel }: Props = $props();

	// Form state
	let formData = $state<ServiceEvent>(event ? { ...event } : createEmptyEvent());

	// Debounced function to save draft
	const debouncedSaveDraft = debounce(async (data: ServiceEvent, origId: string | null) => {
		try {
			await appSettingsStore.set('draftEvent', data);
			await appSettingsStore.set('draftEventOriginalId', origId);
			await appSettingsStore.set('draftSaved', false); // Mark as unsaved draft
		} catch (error) {
			console.error('Failed to save draft:', error);
		}
	}, 500);

	// Auto-save draft when formData changes
	$effect(() => {
		// Create a copy to track changes
		const data = { ...formData };
		// Determine if this is an edit (use originalEventId or event.id)
		const origId = originalEventId || (event?.id !== formData.id ? null : event?.id) || null;
		debouncedSaveDraft(data, origId);
	});

	// Bible fetch state
	let textusLoading = $state(false);
	let leckioLoading = $state(false);
	let textusQuery = $state(event?.textus || '');
	let leckioQuery = $state(event?.leckio || '');
	let suggestions = $state<LegacySuggestion[]>([]);
	let showSuggestions = $state(false);
	let activeSuggestionField = $state<'textus' | 'leckio' | null>(null);

	// Calculated title for YouTube (derived from formData)
	const calculatedTitle = $derived(generateCalculatedTitle(formData));
	const calculatedTitleLength = $derived(calculatedTitle.length);
	const MAX_TITLE_LENGTH = 100;

	// Debounced fetch for suggestions (legacy translations)
	const debouncedFetchSuggestions = debounce(async (term: string, field: 'textus' | 'leckio') => {
		if (term.length < 2) {
			suggestions = [];
			showSuggestions = false;
			return;
		}

		try {
			const result = await bibleApi.fetchSuggestions(term);
			suggestions = result;
			showSuggestions = result.length > 0;
			activeSuggestionField = field;
		} catch (error) {
			console.error('Failed to fetch suggestions:', error);
			suggestions = [];
			showSuggestions = false;
		}
	}, 300);

	// Debounced fetch for V2 translations
	const debouncedFetchV2 = debounce(async (term: string, translation: BibleTranslation, field: 'textus' | 'leckio') => {
		if (term.length < 2) return;

		const loadingState = field === 'textus' ? (textusLoading = true) : (leckioLoading = true);

		try {
			const result = await bibleApi.fetchVerses(term, translation);
			if (field === 'textus') {
				formData.textus = result.label;
				formData.textusVerses = result.verses;
				textusLoading = false;
			} else {
				formData.leckio = result.label;
				formData.leckioVerses = result.verses;
				leckioLoading = false;
			}
		} catch (error) {
			if (field === 'textus') textusLoading = false;
			else leckioLoading = false;
			console.error('Failed to fetch verses:', error);
		}
	}, 500);

	// Handle Bible query input
	function handleBibleInput(field: 'textus' | 'leckio', value: string) {
		if (field === 'textus') {
			textusQuery = value;
		} else {
			leckioQuery = value;
		}

		const translation = field === 'textus' ? formData.textusTranslation : formData.leckioTranslation;

		if (isV2Translation(translation)) {
			suggestions = [];
			showSuggestions = false;
			debouncedFetchV2(value, translation, field);
		} else {
			debouncedFetchSuggestions(value, field);
		}
	}

	// Handle translation change
	function handleTranslationChange(field: 'textus' | 'leckio', translation: BibleTranslation) {
		if (field === 'textus') {
			formData.textusTranslation = translation;
			formData.textusVerses = [];
		} else {
			formData.leckioTranslation = translation;
			formData.leckioVerses = [];
		}

		const query = field === 'textus' ? textusQuery : leckioQuery;
		if (query.length >= 2) {
			if (isV2Translation(translation)) {
				debouncedFetchV2(query, translation, field);
			} else {
				debouncedFetchSuggestions(query, field);
			}
		}
	}

	// Handle suggestion selection
	async function handleSuggestionSelect(field: 'textus' | 'leckio', suggestion: LegacySuggestion) {
		if (field === 'textus') {
			textusQuery = suggestion.label;
			textusLoading = true;
		} else {
			leckioQuery = suggestion.label;
			leckioLoading = true;
		}

		suggestions = [];
		showSuggestions = false;

		const translation = field === 'textus' ? formData.textusTranslation : formData.leckioTranslation;

		try {
			const result = await bibleApi.fetchLegacy(suggestion.link, translation);
			if (field === 'textus') {
				formData.textus = result.label;
				formData.textusVerses = result.verses;
				textusLoading = false;
			} else {
				formData.leckio = result.label;
				formData.leckioVerses = result.verses;
				leckioLoading = false;
			}
		} catch (error) {
			if (field === 'textus') textusLoading = false;
			else leckioLoading = false;
			console.error('Failed to fetch verses:', error);
		}
	}

	// Manual search
	async function handleSearch(field: 'textus' | 'leckio') {
		const query = field === 'textus' ? textusQuery : leckioQuery;
		const translation = field === 'textus' ? formData.textusTranslation : formData.leckioTranslation;

		if (!query.trim()) return;

		if (field === 'textus') textusLoading = true;
		else leckioLoading = true;

		try {
			const result = await bibleApi.fetchVerses(query, translation);
			if (field === 'textus') {
				formData.textus = result.label;
				formData.textusVerses = result.verses;
				textusLoading = false;
			} else {
				formData.leckio = result.label;
				formData.leckioVerses = result.verses;
				leckioLoading = false;
			}
		} catch (error) {
			if (field === 'textus') textusLoading = false;
			else leckioLoading = false;
			toast({
				title: $_('events.form.fetchError'),
				description: error instanceof Error ? error.message : 'Unknown error',
				variant: 'error'
			});
		}
	}

	// Save form
	function handleSave() {
		if (!formData.title.trim()) {
			toast({
				title: $_('events.form.validation.title'),
				variant: 'warning'
			});
			return;
		}

		if (!formData.date) {
			toast({
				title: $_('events.form.validation.date'),
				variant: 'warning'
			});
			return;
		}

		formData.updatedAt = new Date().toISOString();
		onSave(formData);
	}

	// Clear verses for a field
	function clearVerses(field: 'textus' | 'leckio') {
		if (field === 'textus') {
			formData.textus = '';
			formData.textusVerses = [];
			textusQuery = '';
		} else {
			formData.leckio = '';
			formData.leckioVerses = [];
			leckioQuery = '';
		}
	}

	// YouTube scheduling state
	let showYoutubeLoginModal = $state(false);
	let isSchedulingYoutube = $state(false);

	// Schedule YouTube broadcast
	async function handleScheduleYoutube() {
		if (!$systemStore.youtubeLoggedIn) {
			showYoutubeLoginModal = true;
			return;
		}

		if (!formData.date || !formData.time) {
			toast({
				title: $_('events.form.validation.date'),
				variant: 'warning'
			});
			return;
		}

		isSchedulingYoutube = true;
		try {
			// Create scheduled start time in ISO format
			const scheduledStartTime = new Date(`${formData.date}T${formData.time}:00`).toISOString();

			const broadcast = await youtubeApi.createBroadcast({
				title: calculatedTitle,
				description: generateYoutubeDescription(),
				scheduledStartTime,
				privacyStatus: 'unlisted', // Default to unlisted for church services
				enableDvr: true,
				enableEmbed: true
			});

			formData.youtubeScheduledId = broadcast.id;

			toast({
				title: $_('youtube.scheduling.scheduled'),
				description: $_('youtube.scheduling.scheduledDescription'),
				variant: 'success'
			});
		} catch (error) {
			toast({
				title: $_('youtube.modal.error'),
				description: error instanceof Error ? error.message : $_('youtube.scheduling.failed'),
				variant: 'error'
			});
		} finally {
			isSchedulingYoutube = false;
		}
	}

	// Generate description from event data
	function generateYoutubeDescription(): string {
		const parts: string[] = [];

		if (formData.speaker) {
			parts.push(`${$_('events.form.speaker')}: ${formData.speaker}`);
		}
		if (formData.textus) {
			parts.push(`Textus: ${formData.textus}`);
		}
		if (formData.leckio) {
			parts.push(`Lekció: ${formData.leckio}`);
		}
		if (formData.description) {
			parts.push('');
			parts.push(formData.description);
		}

		return parts.join('\n');
	}

	// Handle successful YouTube login - auto-schedule if triggered from form
	function handleYoutubeLoginSuccess() {
		// After successful login, automatically schedule
		handleScheduleYoutube();
	}

	// Toggle verse editing mode
	function toggleEditing(field: 'textus' | 'leckio', index: number) {
		if (field === 'textus') {
			formData.textusVerses = formData.textusVerses.map((v, i) =>
				i === index ? { ...v, editing: !v.editing } : v
			);
		} else {
			formData.leckioVerses = formData.leckioVerses.map((v, i) =>
				i === index ? { ...v, editing: !v.editing } : v
			);
		}
	}

	// Update verse text
	function handleVerseChange(field: 'textus' | 'leckio', index: number, text: string) {
		if (field === 'textus') {
			formData.textusVerses = formData.textusVerses.map((v, i) =>
				i === index ? { ...v, text } : v
			);
		} else {
			formData.leckioVerses = formData.leckioVerses.map((v, i) =>
				i === index ? { ...v, text } : v
			);
		}
	}
</script>

<div class="space-y-6">
	<!-- Basic Info -->
	<Card>
		<svelte:fragment slot="title">{$_('events.form.basicInfo')}</svelte:fragment>
		<svelte:fragment slot="content">
			<div class="grid gap-4 md:grid-cols-2">
				<div class="md:col-span-2 space-y-2">
					<Label for="event-title">{$_('events.form.title')}</Label>
					<Input
						id="event-title"
						bind:value={formData.title}
						placeholder={$_('events.form.titlePlaceholder')}
					/>
				</div>

				<div class="space-y-2">
					<Label for="event-date">{$_('events.form.date')}</Label>
					<Input id="event-date" type="date" bind:value={formData.date} />
				</div>

				<div class="space-y-2">
					<Label for="event-time">{$_('events.form.time')}</Label>
					<Input id="event-time" type="time" bind:value={formData.time} />
				</div>

				<div class="md:col-span-2 space-y-2">
					<Label for="event-speaker">{$_('events.form.speaker')}</Label>
					<Input
						id="event-speaker"
						bind:value={formData.speaker}
						placeholder={$_('events.form.speakerPlaceholder')}
					/>
				</div>

				<div class="md:col-span-2 space-y-2">
					<Label for="event-description">{$_('events.form.description')}</Label>
					<Textarea
						id="event-description"
						bind:value={formData.description}
						placeholder={$_('events.form.descriptionPlaceholder')}
						rows={3}
					/>
				</div>

				<!-- YouTube Privacy Setting -->
				<div class="md:col-span-2 space-y-2">
					<Label for="event-privacy">{$_('events.form.privacy')}</Label>
					<select
						id="event-privacy"
						bind:value={formData.youtubePrivacyStatus}
						class="flex h-10 w-full max-w-xs rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2"
					>
						<option value="public">{$_('events.form.privacyOptions.public')}</option>
						<option value="unlisted">{$_('events.form.privacyOptions.unlisted')}</option>
						<option value="private">{$_('events.form.privacyOptions.private')}</option>
					</select>
				</div>

				<!-- Calculated YouTube Title -->
				<div class="md:col-span-2 space-y-2">
					<div class="flex items-center justify-between">
						<Label>{$_('events.form.calculatedTitle')}</Label>
						<Badge
							variant={calculatedTitleLength > MAX_TITLE_LENGTH ? 'destructive' : 'secondary'}
							className="text-xs"
						>
							[{calculatedTitleLength}/{MAX_TITLE_LENGTH}]
						</Badge>
					</div>
					<div
						class="p-3 rounded-md border bg-muted/50 text-sm {calculatedTitleLength > MAX_TITLE_LENGTH ? 'border-destructive' : ''}"
					>
						{calculatedTitle || $_('events.form.calculatedTitleEmpty')}
					</div>
				</div>
			</div>
		</svelte:fragment>
	</Card>

	<!-- Bible References with Tabs -->
	<Card>
		<svelte:fragment slot="title">
			<BookOpen class="h-5 w-5 mr-2 inline" />
			{$_('events.form.bibleReferences')}
		</svelte:fragment>
		<svelte:fragment slot="content">
			<Tabs defaultValue="textus">
				<TabsList className="grid w-full max-w-md grid-cols-2 mb-4">
					<TabsTrigger value="textus">{$_('bible.tabs.textus')}</TabsTrigger>
					<TabsTrigger value="leckio">{$_('bible.tabs.leckio')}</TabsTrigger>
				</TabsList>

				{#each ['textus', 'leckio'] as field}
					{@const isTextus = field === 'textus'}
					{@const loading = isTextus ? textusLoading : leckioLoading}
					{@const query = isTextus ? textusQuery : leckioQuery}
					{@const translation = isTextus ? formData.textusTranslation : formData.leckioTranslation}
					{@const verses = isTextus ? formData.textusVerses : formData.leckioVerses}
					{@const label = isTextus ? formData.textus : formData.leckio}

					<TabsContent value={field}>
						<div class="space-y-4">
							<!-- Search controls -->
							<div class="flex gap-2 flex-wrap">
								<div class="w-36 space-y-2">
									<Label>{$_('bible.search.translation')}</Label>
									<TranslationSelector
										value={translation}
										onValueChange={(v) => handleTranslationChange(field as 'textus' | 'leckio', v)}
										id={`${field}-translation`}
									/>
								</div>

								<div class="flex-1 min-w-[200px] space-y-2 relative">
									<Label for={`${field}-search`}>{$_('bible.search.label')}</Label>
									<Input
										id={`${field}-search`}
										value={query}
										oninput={(e: Event & { currentTarget: HTMLInputElement }) =>
											handleBibleInput(field as 'textus' | 'leckio', e.currentTarget.value)}
										placeholder={$_('events.form.biblePlaceholder')}
									/>
									{#if showSuggestions && activeSuggestionField === field}
										<BibleSuggestions
											{suggestions}
											visible={true}
											onSelect={(s) => handleSuggestionSelect(field as 'textus' | 'leckio', s)}
										/>
									{/if}
								</div>

								<Button
									className="mt-auto"
									onclick={() => handleSearch(field as 'textus' | 'leckio')}
									disabled={loading}
								>
									{#if loading}
										<Loader2 class="mr-2 h-4 w-4 animate-spin" />
									{:else}
										<Search class="mr-2 h-4 w-4" />
									{/if}
									{$_('bible.search.fetch')}
								</Button>
							</div>

							<!-- Verses with inline editing -->
							{#if verses.length > 0}
								<div class="border rounded-md bg-muted/30">
									<div class="flex items-center justify-between p-3 border-b">
										<Badge variant="secondary">{label}</Badge>
										<Button
											buttonVariant="ghost"
											buttonSize="sm"
											onclick={() => clearVerses(field as 'textus' | 'leckio')}
										>
											<X class="h-4 w-4 mr-1" />
											{$_('events.form.clear')}
										</Button>
									</div>
									<div class="max-h-60 overflow-y-auto">
										{#each verses as verse, index (index)}
											<div
												class="flex gap-3 items-start p-2 hover:bg-accent/30 transition-colors {index !== verses.length - 1 ? 'border-b' : ''}"
											>
												<Badge variant="outline" className="shrink-0 mt-0.5 text-xs">
													{verse.chapter}:{verse.verse}
												</Badge>
												{#if verse.editing}
													<Textarea
														value={verse.text}
														oninput={(e: Event & { currentTarget: HTMLTextAreaElement }) =>
															handleVerseChange(field as 'textus' | 'leckio', index, e.currentTarget.value)}
														className="flex-1 min-h-[60px] text-sm"
													/>
												{:else}
													<p class="flex-1 text-sm leading-relaxed">{verse.text}</p>
												{/if}
												<Button
													buttonVariant="ghost"
													buttonSize="icon"
													className="shrink-0 h-8 w-8"
													onclick={() => toggleEditing(field as 'textus' | 'leckio', index)}
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
								</div>
							{/if}
						</div>
					</TabsContent>
				{/each}
			</Tabs>
		</svelte:fragment>
	</Card>

	<!-- YouTube Scheduling -->
	<Card>
		<svelte:fragment slot="title">
			<Youtube class="h-5 w-5 mr-2 inline" />
			{$_('youtube.scheduling.title')}
		</svelte:fragment>
		<svelte:fragment slot="content">
			{#if formData.youtubeScheduledId}
				<!-- Scheduled state -->
				<div
					class="flex items-center justify-between p-3 bg-green-50 dark:bg-green-900/20 rounded-md border border-green-200 dark:border-green-800"
				>
					<div class="flex items-center gap-2">
						<Check class="h-5 w-5 text-green-600 dark:text-green-400" />
						<span class="font-medium text-green-800 dark:text-green-200"
							>{$_('youtube.scheduling.scheduled')}</span
						>
					</div>
					<Button
						buttonVariant="outline"
						buttonSize="sm"
						href={youtubeApi.getYoutubeStudioUrl(formData.youtubeScheduledId)}
						target="_blank"
					>
						<ExternalLink class="h-4 w-4 mr-1" />
						{$_('youtube.scheduling.viewInStudio')}
					</Button>
				</div>
			{:else}
				<!-- Not scheduled state -->
				<div class="space-y-3">
					<p class="text-sm text-muted-foreground">
						{$_('youtube.scheduling.notScheduled')}
					</p>
					<Button onclick={handleScheduleYoutube} disabled={isSchedulingYoutube}>
						{#if isSchedulingYoutube}
							<Loader2 class="mr-2 h-4 w-4 animate-spin" />
							{$_('youtube.scheduling.scheduling')}
						{:else}
							<Youtube class="mr-2 h-4 w-4" />
							{$_('youtube.scheduling.scheduleButton')}
						{/if}
					</Button>
				</div>
			{/if}
		</svelte:fragment>
	</Card>

	<!-- Actions -->
	<div class="flex justify-end gap-2">
		<Button buttonVariant="outline" onclick={onCancel}>
			<X class="mr-2 h-4 w-4" />
			{$_('events.form.cancel')}
		</Button>
		<Button onclick={handleSave}>
			<Save class="mr-2 h-4 w-4" />
			{$_('events.form.save')}
		</Button>
	</div>
</div>

<!-- YouTube Login Modal -->
<YouTubeLoginModal
	open={showYoutubeLoginModal}
	onClose={() => (showYoutubeLoginModal = false)}
	onSuccess={handleYoutubeLoginSuccess}
/>
