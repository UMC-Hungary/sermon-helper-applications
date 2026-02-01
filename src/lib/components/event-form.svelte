<script lang="ts">
	import { _ } from 'svelte-i18n';
	import Card from '$lib/components/ui/card.svelte';
	import Button from '$lib/components/ui/button.svelte';
	import Input from '$lib/components/ui/input.svelte';
	import Label from '$lib/components/ui/label.svelte';
	import Textarea from '$lib/components/ui/textarea.svelte';
	import Badge from '$lib/components/ui/badge.svelte';
	import Checkbox from '$lib/components/ui/checkbox.svelte';
	import Tabs from '$lib/components/ui/tabs.svelte';
	import TabsList from '$lib/components/ui/tabs-list.svelte';
	import TabsTrigger from '$lib/components/ui/tabs-trigger.svelte';
	import TabsContent from '$lib/components/ui/tabs-content.svelte';
	import TranslationSelector from '$lib/components/translation-selector.svelte';
	import BibleSuggestions from '$lib/components/bible-suggestions.svelte';
	import { toast } from '$lib/utils/toast';
	import { bibleApi } from '$lib/utils/bible-api';
	import { isV2Translation, type LegacySuggestion } from '$lib/types/bible';
	import { debounce } from '$lib/utils/debounce';
	import {
		createEmptyEvent,
		generateCalculatedTitle,
		getRecordingStatus,
		getYouTubeBroadcastUrl,
		getYouTubeVideoUrl,
		getUploadProgress,
		type ServiceEvent
	} from '$lib/types/event';
	import { appSettingsStore } from '$lib/utils/app-settings-store';
	import { Search, Save, X, Loader2, BookOpen, Edit2, Check, Youtube, ExternalLink, Upload, Video, Globe, Link, Lock } from 'lucide-svelte';
	import { systemStore } from '$lib/stores/system-store';
	import { eventStore } from '$lib/stores/event-store';
	import { youtubeApi } from '$lib/utils/youtube-api';
	import { scheduleYoutubeBroadcast } from '$lib/utils/youtube-helpers';
	import YouTubeLoginModal from '$lib/components/youtube-login-modal.svelte';

	interface Props {
		event?: ServiceEvent;
		originalEventId?: string; // Set if editing an existing event
		onSave: (event: ServiceEvent) => void;
		onCancel: () => void;
	}

	let { event, originalEventId, onSave, onCancel }: Props = $props();

	// Form state
	let formData = $state<ServiceEvent>(event ? { ...event, ...(event.autoUploadEnabled ? {} : { autoUploadEnabled: true }) } : createEmptyEvent());

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

	// Recording/upload status (derived from formData)
	const recordingStatus = $derived(getRecordingStatus(formData));
	const uploadProgress = $derived(getUploadProgress(formData));

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

	let showYoutubeLoginModal = $state(false);

	// Determine if this is a stored event (vs a new draft)
	const isStoredEvent = $derived(!!originalEventId);

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

		await scheduleYoutubeBroadcast(formData);

		// Sync youtubeScheduledId from store since formData is a local copy
		const updatedEvent = eventStore.getEventById(formData.id);
		if (updatedEvent?.youtubeScheduledId) {
			formData.youtubeScheduledId = updatedEvent.youtubeScheduledId;
		}
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

<div class="grid gap-6 lg:grid-cols-3 min-h-screen mt-6">
	<!-- Left Column: Main Edit Block and YouTube Scheduling -->
	<div class="space-y-6 lg:col-span-1">
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
				</div>
			</svelte:fragment>
		</Card>

		<!-- YouTube Scheduling (only shown for stored events) -->
		{#if isStoredEvent}
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
							<Button onclick={handleScheduleYoutube} disabled={formData.isBroadcastScheduling}>
								{#if formData.isBroadcastScheduling}
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
		{/if}

		<!-- Recording & Upload -->
		<Card>
			<svelte:fragment slot="title">
				<Video class="h-5 w-5 mr-2 inline" />
				{$_('events.form.recording.title')}
			</svelte:fragment>
			<svelte:fragment slot="content">
				<div class="space-y-4">
					<!-- Auto-upload toggle -->
					<div class="flex items-center gap-3">
						<Checkbox
							id="auto-upload"
							bind:checked={formData.autoUploadEnabled}
						/>
						<Label for="auto-upload" className="text-sm font-normal cursor-pointer">
							{$_('events.form.recording.autoUpload')}
						</Label>
					</div>

					<!-- Upload visibility dropdown -->
					<div class="space-y-2">
						<Label for="upload-privacy">{$_('events.form.recording.uploadVisibility')}</Label>
						<select
							id="upload-privacy"
							bind:value={formData.uploadPrivacyStatus}
							class="flex h-10 w-full max-w-xs rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2"
						>
							<option value="public">{$_('events.form.privacyOptions.public')}</option>
							<option value="unlisted">{$_('events.form.privacyOptions.unlisted')}</option>
							<option value="private">{$_('events.form.privacyOptions.private')}</option>
						</select>
					</div>

					<!-- Status section (only shown if event has broadcast or recording) -->
					{#if formData.youtubeScheduledId || formData.youtubeUploadedId || recordingStatus !== 'none'}
						<div class="border-t pt-4 mt-4 space-y-3">
							<!-- Live Broadcast Status -->
							{#if formData.youtubeScheduledId}
								<div class="flex items-center justify-between">
									<div class="flex items-center gap-2">
										<Youtube class="h-4 w-4 text-red-500" />
										<span class="text-sm font-medium">{$_('events.form.recording.liveBroadcast')}</span>
									</div>
									<div class="flex items-center gap-2">
										<Button
											buttonVariant="outline"
											buttonSize="sm"
											href={getYouTubeBroadcastUrl(formData) ?? undefined}
											target="_blank"
										>
											<ExternalLink class="h-3 w-3 mr-1" />
											{$_('events.form.recording.watchBroadcast')}
										</Button>
										<Badge variant="secondary">
											{#if formData.youtubePrivacyStatus === 'public'}
												<Globe class="h-3 w-3 mr-1" />
											{:else if formData.youtubePrivacyStatus === 'unlisted'}
												<Link class="h-3 w-3 mr-1" />
											{:else}
												<Lock class="h-3 w-3 mr-1" />
											{/if}
											{$_(`events.form.privacyOptions.${formData.youtubePrivacyStatus}`)}
										</Badge>
									</div>
								</div>
							{/if}

							<!-- Recording/Upload Status -->
							<div class="flex items-center justify-between">
								<div class="flex items-center gap-2">
									<Video class="h-4 w-4 text-muted-foreground" />
									<span class="text-sm font-medium">{$_('events.form.recording.uploadedRecording')}</span>
								</div>
								<div class="flex items-center gap-2">
									{#if recordingStatus === 'uploaded' && formData.youtubeUploadedId}
										<Button
											buttonVariant="outline"
											buttonSize="sm"
											href={getYouTubeVideoUrl(formData) ?? undefined}
											target="_blank"
										>
											<ExternalLink class="h-3 w-3 mr-1" />
											{$_('events.form.recording.watchRecording')}
										</Button>
									{:else if recordingStatus === 'uploading'}
										<div class="flex items-center gap-2">
											<div class="w-24 h-2 bg-muted rounded-full overflow-hidden">
												<div
													class="h-full bg-primary transition-all"
													style="width: {uploadProgress}%"
												></div>
											</div>
											<span class="text-xs text-muted-foreground">{uploadProgress}%</span>
										</div>
									{:else if recordingStatus === 'pending' && !formData.autoUploadEnabled}
										<Button buttonVariant="outline" buttonSize="sm" disabled>
											<Upload class="h-3 w-3 mr-1" />
											{$_('events.form.recording.actions.startUpload')}
										</Button>
									{/if}
									<Badge
										variant={recordingStatus === 'uploaded' ? 'success' : recordingStatus === 'failed' ? 'destructive' : 'secondary'}
									>
										{$_(`events.form.recording.status.${recordingStatus}`)}
									</Badge>
									{#if recordingStatus === 'uploaded'}
										<Badge variant="secondary">
											{#if formData.uploadPrivacyStatus === 'public'}
												<Globe class="h-3 w-3 mr-1" />
											{:else if formData.uploadPrivacyStatus === 'unlisted'}
												<Link class="h-3 w-3 mr-1" />
											{:else}
												<Lock class="h-3 w-3 mr-1" />
											{/if}
											{$_(`events.form.privacyOptions.${formData.uploadPrivacyStatus}`)}
										</Badge>
									{/if}
								</div>
							</div>
						</div>
					{/if}
				</div>
			</svelte:fragment>
		</Card>
	</div>

	<!-- Right Column: Bible References -->
	<div class="md:col-span-1 lg:col-span-2">
		<!-- Bible References with Tabs -->
		<Card class="h-full">
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
										<div>
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
	</div>
</div>

<!-- Spacer for fixed action bar -->
<div class="h-20"></div>

<!-- Actions - Fixed at bottom -->
<div class="fixed bottom-0 left-0 right-0 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 border-t py-3 px-6 z-50">
	<div class="flex justify-end gap-2 mx-auto">
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
