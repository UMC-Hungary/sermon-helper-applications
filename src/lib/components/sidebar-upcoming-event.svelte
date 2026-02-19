<script lang="ts">
	import { CheckCircle2, Loader2, Edit, RefreshCw, Check, FileText, FolderOpen } from 'lucide-svelte';
	import Button from '$lib/components/ui/button.svelte';
	import Card from '$lib/components/ui/card.svelte';
	import Badge from '$lib/components/ui/badge.svelte';
	import { upcomingEvents, eventStore } from '$lib/stores/event-store';
    import {isEventToday, formatEventDate, generateCalculatedTitle} from '$lib/types/event';
	import { _ } from 'svelte-i18n';
	import { toast } from '$lib/utils/toast';
	import SidebarStreamingControls from '$lib/components/sidebar-streaming-controls.svelte';
	import RecordingsStatus from '$lib/components/recordings-status.svelte';
	import { browserSourceConfigs } from '$lib/stores/obs-devices-store';
	import { obsBrowserStatuses } from '$lib/stores/obs-device-status-store';
	import { manualRefreshBrowserSource } from '$lib/utils/obs-device-checker';
	import { generatePptx } from '$lib/utils/pptx-generator';
	import { saveFile, pickOutputFolder, openFolder } from '$lib/utils/file-saver';
	import { appSettings, appSettingsStore } from '$lib/utils/app-settings-store';
	import { isTauriApp } from '$lib/utils/storage-helpers';

	export let onMobileMenuClose: () => void = () => {};

	// Reactive: get the soonest upcoming event (first in sorted list)
	$: nextEvent = $upcomingEvents.length > 0 ? $upcomingEvents[0] : null;
	$: isToday = nextEvent ? isEventToday(nextEvent) : false;

	// State for PPTX generation
	let generatingTextus = false;
	let generatingLeckio = false;
	$: isTauri = isTauriApp();
	$: pptxOutputPath = $appSettings.pptxOutputPath;

	// Handle PPTX generation
	async function handleGeneratePptx(type: 'textus' | 'leckio') {
		if (!nextEvent) return;

		const verses = type === 'textus' ? nextEvent.textusVerses : nextEvent.leckioVerses;
		const reference = type === 'textus' ? nextEvent.textus : nextEvent.leckio;

		if (!verses?.length || !reference) {
			toast({
				title: $_('toasts.error.title'),
				description: $_('sidebar.upcomingEvent.pptx.noVerses'),
				variant: 'error'
			});
			return;
		}

		// Check if output path is configured (Tauri only)
		if (isTauri && !pptxOutputPath) {
			const selectedPath = await pickOutputFolder();
			if (!selectedPath) {
				toast({
					title: $_('toasts.error.title'),
					description: $_('sidebar.upcomingEvent.pptx.noFolder'),
					variant: 'error'
				});
				return;
			}
			await appSettingsStore.set('pptxOutputPath', selectedPath);
		}

		// Set loading state
		if (type === 'textus') {
			generatingTextus = true;
		} else {
			generatingLeckio = true;
		}

		try {
			// Generate PPTX
			const blob = await generatePptx({ reference, verses, type });
			const filename = `${type === 'textus' ? 'textus' : 'lekcio'}.pptx`;

			// Save or download
			const result = await saveFile(blob, filename, $appSettings.pptxOutputPath);

			if (result.success) {
				// Update event with generation timestamp
				const timestampField = type === 'textus' ? 'textusGeneratedAt' : 'leckioGeneratedAt';
				await eventStore.updateEvent(nextEvent.id, { [timestampField]: new Date().toISOString() });

				toast({
					title: $_('toasts.success.title'),
					description: $_('sidebar.upcomingEvent.pptx.generated'),
					variant: 'success'
				});
			} else {
				toast({
					title: $_('toasts.error.title'),
					description: result.error || $_('sidebar.upcomingEvent.pptx.failed'),
					variant: 'error'
				});
			}
		} catch (error) {
			toast({
				title: $_('toasts.error.title'),
				description: error instanceof Error ? error.message : $_('sidebar.upcomingEvent.pptx.failed'),
				variant: 'error'
			});
		} finally {
			if (type === 'textus') {
				generatingTextus = false;
			} else {
				generatingLeckio = false;
			}
		}
	}

	// Change output folder
	async function handleChangeFolder() {
		const selectedPath = await pickOutputFolder();
		if (selectedPath) {
			await appSettingsStore.set('pptxOutputPath', selectedPath);
		}
	}
</script>

<Card className="p-4">
	<div class="flex items-center justify-between mb-3">
		<h3 class="font-medium text-sm text-card-foreground">{$_('sidebar.upcomingEvent.title')}</h3>
		{#if nextEvent && isToday}
			<Badge variant="success" className="text-xs">{$_('sidebar.upcomingEvent.today')}</Badge>
		{/if}
	</div>

	{#if nextEvent}
		<div class="space-y-3">
			<!-- Streaming Controls (only show for today's event) -->
			{#if isToday}
				<SidebarStreamingControls event={nextEvent} />
			{/if}

			<!-- Event Title -->
			<p class="text-sm font-medium text-card-foreground line-clamp-2">{generateCalculatedTitle(nextEvent)}</p>

			<!-- Scheduled Date -->
			<div>
				<span class="text-xs text-muted-foreground">{$_('sidebar.upcomingEvent.scheduledDate')}</span>
				<p class="text-sm text-card-foreground">{formatEventDate(nextEvent.date)}</p>
			</div>

			<!-- Recordings Status (compact view in sidebar) -->
			{#if isToday && isTauri && (nextEvent.recordings?.length ?? 0) > 0}
				<div class="pt-2 border-t border-border">
					<span class="text-xs text-muted-foreground mb-2 block">{$_('sidebar.upcomingEvent.recordings') || 'Recordings'}</span>
					<RecordingsStatus event={nextEvent} compact />
				</div>
			{/if}

			<!-- Textus (if present) -->
			{#if nextEvent.textus}
				<div>
					<span class="text-xs text-muted-foreground">{$_('sidebar.upcomingEvent.textus')}</span>
					<p class="text-sm font-medium text-card-foreground">{nextEvent.textus}</p>
				</div>
			{/if}

			<!-- Leckio (if present) -->
			{#if nextEvent.leckio}
				<div>
					<span class="text-xs text-muted-foreground">{$_('sidebar.upcomingEvent.leckio')}</span>
					<p class="text-sm font-medium text-card-foreground">{nextEvent.leckio}</p>
				</div>
			{/if}

			<!-- PPTX Generation Buttons -->
			{#if nextEvent.textus || nextEvent.leckio}
				<div class="pt-2 border-t border-border">
					<span class="text-xs text-muted-foreground mb-2 block">{$_('sidebar.upcomingEvent.pptx.title')}</span>
					<div class="flex gap-2">
						{#if nextEvent.textus}
							<Button
								buttonVariant="outline"
								buttonSize="sm"
								className="flex-1"
								onclick={() => handleGeneratePptx('textus')}
								disabled={generatingTextus || !nextEvent.textusVerses?.length}
							>
								{#if generatingTextus}
									<Loader2 class="h-4 w-4 mr-1 animate-spin" />
								{:else if nextEvent.textusGeneratedAt}
									<Check class="h-4 w-4 mr-1 text-green-600" />
								{:else}
									<FileText class="h-4 w-4 mr-1" />
								{/if}
								Textus
							</Button>
						{/if}
						{#if nextEvent.leckio}
							<Button
								buttonVariant="outline"
								buttonSize="sm"
								className="flex-1"
								onclick={() => handleGeneratePptx('leckio')}
								disabled={generatingLeckio || !nextEvent.leckioVerses?.length}
							>
								{#if generatingLeckio}
									<Loader2 class="h-4 w-4 mr-1 animate-spin" />
								{:else if nextEvent.leckioGeneratedAt}
									<Check class="h-4 w-4 mr-1 text-green-600" />
								{:else}
									<FileText class="h-4 w-4 mr-1" />
								{/if}
								Lekció
							</Button>
						{/if}
					</div>
					{#if isTauri && pptxOutputPath}
						<button
							type="button"
							onclick={() => openFolder(pptxOutputPath)}
							class="flex items-center gap-1 mt-2 text-xs text-muted-foreground hover:text-foreground transition-colors cursor-pointer"
						>
							<FolderOpen class="h-3 w-3" />
							<span class="truncate">{pptxOutputPath}</span>
						</button>
					{/if}
					{#if isTauri}
						<Button
							buttonVariant="ghost"
							buttonSize="sm"
							className="w-full mt-1 text-xs"
							onclick={handleChangeFolder}
						>
							{$_('sidebar.upcomingEvent.pptx.changeFolder')}
						</Button>
					{/if}
				</div>
			{/if}

			<!-- Browser Source Status -->
			{#if $browserSourceConfigs.length > 0}
				<div class="pt-2 border-t border-border">
					<span class="text-xs text-muted-foreground mb-2 block">{$_('sidebar.upcomingEvent.browserSources')}</span>
					{#each $browserSourceConfigs as config (config.id)}
						{@const status = $obsBrowserStatuses.get(config.id)}
						<div class="flex items-center justify-between py-1">
							<span class="text-sm text-muted-foreground">{config.name}</span>
							{#if status?.refreshPending}
								<Loader2 class="h-4 w-4 text-blue-600 animate-spin" />
							{:else if status?.matches}
								<CheckCircle2 class="h-4 w-4 text-green-600" />
							{:else if status?.refreshSuccess === false}
								<button
									type="button"
									onclick={() => manualRefreshBrowserSource(config.id)}
									class="p-1 hover:bg-muted rounded transition-colors"
									title={$_('sidebar.upcomingEvent.refreshBrowserSource')}
								>
									<RefreshCw class="h-4 w-4 text-amber-600" />
								</button>
							{:else}
								<button
									type="button"
									onclick={() => manualRefreshBrowserSource(config.id)}
									class="p-1 hover:bg-muted rounded transition-colors"
									title={$_('sidebar.upcomingEvent.refreshBrowserSource')}
								>
									<RefreshCw class="h-4 w-4 text-amber-600" />
								</button>
							{/if}
						</div>
					{/each}
				</div>
			{/if}

			<!-- Edit Button -->
			<Button
				buttonVariant="outline"
				buttonSize="sm"
				className="w-full"
				href={`/events/${nextEvent.id}`}
				onclick={() => onMobileMenuClose()}
			>
				<Edit class="h-4 w-4 mr-2" />
				{$_('sidebar.upcomingEvent.edit')}
			</Button>
		</div>
	{:else}
		<!-- Empty State -->
		<div class="text-center py-4">
			<p class="text-sm text-muted-foreground">{$_('sidebar.upcomingEvent.noUpcoming.title')}</p>
			<p class="text-xs text-muted-foreground mt-1">{$_('sidebar.upcomingEvent.noUpcoming.description')}</p>
		</div>
	{/if}
</Card>
