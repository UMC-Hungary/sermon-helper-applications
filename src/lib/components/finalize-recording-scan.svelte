<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { Loader2, Search, Plus, FolderOpen } from 'lucide-svelte';
	import Button from '$lib/components/ui/button.svelte';
	import { _ } from 'svelte-i18n';
	import { formatDuration, type RecordingFile, type ServiceEvent } from '$lib/types/event';

	interface Props {
		event: ServiceEvent;
		onRecordingFound: (file: RecordingFile) => void;
		knownPaths: Set<string>;
	}

	let { event, onRecordingFound, knownPaths }: Props = $props();

	let isScanning = $state(false);
	let scanResults = $state<RecordingFile[]>([]);
	let hasScanned = $state(false);

	// Determine session time boundaries from activities
	function getSessionBounds(): { start: number; end: number } {
		const activities = event.activities ?? [];
		const startTypes = ['SESSION_STARTED', 'RECORD_STARTED'];
		const endTypes = ['RECORD_STOPPED', 'STREAM_STOPPED'];

		const firstStart = activities.find((a) => startTypes.includes(a.type));
		const lastEnd = activities.findLast((a) => endTypes.includes(a.type));

		return {
			start: firstStart?.timestamp ?? 0,
			end: lastEnd?.timestamp ?? Date.now()
		};
	}

	async function handleScan() {
		if (!event.recordingDirectory) return;

		isScanning = true;
		scanResults = [];

		try {
			const { start, end } = getSessionBounds();
			const results = await invoke<RecordingFile[]>('scan_recording_directory', {
				directory: event.recordingDirectory,
				sessionStart: start,
				sessionEnd: end
			});

			// Filter out already known recordings
			scanResults = results.filter((r) => !knownPaths.has(r.path));
		} catch (error) {
			console.error('[FinalizeRecordingScan] Scan failed:', error);
		} finally {
			isScanning = false;
			hasScanned = true;
		}
	}

	function handleAdd(file: RecordingFile) {
		onRecordingFound(file);
		scanResults = scanResults.filter((r) => r.path !== file.path);
	}
</script>

<div class="space-y-3">
	<h3 class="text-sm font-medium">{$_('finalize.scan.title')}</h3>

	{#if !event.recordingDirectory}
		<p class="text-sm text-muted-foreground">
			{$_('finalize.scan.noDirectory')}
		</p>
	{:else}
		<div class="flex items-center gap-2 text-xs text-muted-foreground">
			<FolderOpen class="h-3.5 w-3.5" />
			<span class="truncate">{event.recordingDirectory}</span>
		</div>

		<Button
			buttonVariant="outline"
			buttonSize="sm"
			onclick={handleScan}
			disabled={isScanning}
		>
			{#if isScanning}
				<Loader2 class="h-4 w-4 animate-spin" />
				{$_('finalize.scan.scanning')}
			{:else}
				<Search class="h-4 w-4" />
				{$_('finalize.scan.startScan')}
			{/if}
		</Button>

		{#if hasScanned && scanResults.length === 0}
			<p class="text-sm text-muted-foreground">{$_('finalize.scan.noneFound')}</p>
		{/if}

		{#if scanResults.length > 0}
			<p class="text-sm text-muted-foreground">
				{$_('finalize.scan.found', { values: { count: scanResults.length } })}
			</p>
			<div class="space-y-1.5">
				{#each scanResults as file (file.path)}
					<div class="flex items-center justify-between gap-2 rounded-md border px-3 py-2">
						<div class="min-w-0 flex-1">
							<span class="text-sm font-medium truncate block">{file.name}</span>
							<span class="text-xs text-muted-foreground">{formatDuration(file.duration)}</span>
						</div>
						<Button
							buttonVariant="ghost"
							buttonSize="sm"
							onclick={() => handleAdd(file)}
						>
							<Plus class="h-4 w-4" />
							{$_('finalize.scan.addToEvent')}
						</Button>
					</div>
				{/each}
			</div>
		{/if}
	{/if}
</div>
