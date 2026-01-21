<script lang="ts">
	import { _ } from 'svelte-i18n';
	import Button from '$lib/components/ui/button.svelte';
	import Badge from '$lib/components/ui/badge.svelte';
	import type { RecordingFile } from '$lib/types/event-session';
	import { formatDuration, formatFileSize, getMostRecentRecording, getLongestRecording } from '$lib/utils/recording-file-selector';
	import { FileVideo, Clock, HardDrive, ChevronRight, Calendar } from 'lucide-svelte';

	export let open: boolean = false;
	export let candidates: RecordingFile[] = [];
	export let onSelect: (file: RecordingFile) => void = () => {};
	export let onCancel: () => void = () => {};

	let dialogElement: HTMLDialogElement;

	// Dialog open/close effect
	$: if (open && dialogElement) {
		dialogElement.showModal();
	} else if (!open && dialogElement) {
		dialogElement.close();
	}

	function handleDialogClose() {
		onCancel();
	}

	function handleSelect(file: RecordingFile) {
		onSelect(file);
	}

	function handleUseLatest() {
		const latest = getMostRecentRecording(candidates);
		if (latest) {
			onSelect(latest);
		}
	}

	function handleUseLongest() {
		const longest = getLongestRecording(candidates);
		if (longest) {
			onSelect(longest);
		}
	}

	function formatTime(timestamp: number): string {
		return new Date(timestamp).toLocaleTimeString(undefined, {
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	function formatDate(timestamp: number): string {
		return new Date(timestamp).toLocaleDateString(undefined, {
			month: 'short',
			day: 'numeric'
		});
	}
</script>

<dialog
	bind:this={dialogElement}
	class="fixed z-50 bg-background border rounded-lg shadow-lg max-w-lg w-full backdrop:bg-black/50 backdrop:backdrop-blur-sm p-0"
	onclose={handleDialogClose}
>
	<div class="p-6 space-y-4">
		<!-- Header -->
		<div class="space-y-2">
			<h2 class="text-lg font-semibold flex items-center gap-2">
				<FileVideo class="h-5 w-5" />
				{$_('recording.selectFile.title')}
			</h2>
			<p class="text-sm text-muted-foreground">
				{$_('recording.selectFile.description', { values: { count: candidates.length } })}
			</p>
		</div>

		<!-- Recording List -->
		<div class="space-y-2 max-h-80 overflow-y-auto">
			{#each candidates as file, index}
				<button
					class="w-full p-3 border rounded-lg hover:bg-accent/50 transition-colors text-left flex items-center gap-3 group"
					onclick={() => handleSelect(file)}
				>
					<div class="flex-1 min-w-0">
						<div class="font-medium truncate">{file.name}</div>
						<div class="flex flex-wrap gap-x-4 gap-y-1 mt-1 text-xs text-muted-foreground">
							<span class="flex items-center gap-1">
								<Clock class="h-3 w-3" />
								{formatDuration(file.duration)}
							</span>
							<span class="flex items-center gap-1">
								<HardDrive class="h-3 w-3" />
								{formatFileSize(file.size)}
							</span>
							<span class="flex items-center gap-1">
								<Calendar class="h-3 w-3" />
								{formatDate(file.modifiedAt)} {formatTime(file.modifiedAt)}
							</span>
						</div>
					</div>
					{#if index === 0}
						<Badge variant="secondary">{$_('common.latest') || 'Latest'}</Badge>
					{/if}
					<ChevronRight class="h-4 w-4 text-muted-foreground group-hover:text-foreground transition-colors" />
				</button>
			{/each}
		</div>

		<!-- Quick Actions -->
		{#if candidates.length > 1}
			<div class="flex gap-2 pt-2 border-t">
				<Button buttonVariant="outline" onclick={handleUseLatest} className="flex-1">
					{$_('recording.selectFile.useLatest')}
				</Button>
				<Button buttonVariant="outline" onclick={handleUseLongest} className="flex-1">
					{$_('recording.selectFile.useLongest') || 'Use Longest'}
				</Button>
			</div>
		{/if}

		<!-- Footer -->
		<div class="flex justify-end pt-2 border-t">
			<Button buttonVariant="ghost" onclick={onCancel}>
				{$_('common.cancel')}
			</Button>
		</div>
	</div>
</dialog>
