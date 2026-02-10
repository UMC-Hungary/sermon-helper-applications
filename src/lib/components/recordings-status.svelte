<script lang="ts">
	import { CheckCircle2, Clock, Upload, AlertCircle, ExternalLink, SquareCheck, Square, Loader2, Eye, EyeOff, Link } from 'lucide-svelte';
	import type { ServiceEvent, EventRecording, YouTubePrivacyStatus } from '$lib/types/event';
	import {
		formatDuration,
		getUploadableRecordings,
		hasMultipleUploadableRecordings,
		MIN_RECORDING_DURATION_SECONDS
	} from '$lib/types/event';
	import { eventStore } from '$lib/stores/event-store';
	import Input from '$lib/components/ui/input.svelte';
	import { _ } from 'svelte-i18n';

	interface Props {
		event: ServiceEvent;
		compact?: boolean;
	}

	let { event, compact = false }: Props = $props();

	// Derived state
	let recordings = $derived(event.recordings ?? []);
	let uploadableRecordings = $derived(getUploadableRecordings(event));
	let needsSelection = $derived(hasMultipleUploadableRecordings(event));

	// Get recording status
	function getRecordingStatus(rec: EventRecording): 'uploaded' | 'uploading' | 'pending' | 'short' {
		if (rec.uploaded) return 'uploaded';
		if (rec.uploadSession) return 'uploading';
		if (rec.file.duration >= MIN_RECORDING_DURATION_SECONDS || rec.whitelisted) return 'pending';
		return 'short';
	}

	// Get upload progress percentage from persisted session
	function getUploadProgress(rec: EventRecording): number {
		if (!rec.uploadSession) return 0;
		if (rec.uploadSession.fileSize === 0) return 0;
		return Math.round((rec.uploadSession.bytesUploaded / rec.uploadSession.fileSize) * 100);
	}

	// Toggle whitelist for a recording
	async function toggleWhitelist(recordingId: string, currentValue: boolean) {
		await eventStore.whitelistRecording(event.id, recordingId, !currentValue);
	}

	// Update custom title
	async function updateTitle(recordingId: string, title: string) {
		await eventStore.updateRecordingTitle(event.id, recordingId, title);
	}

	// Get the upload privacy status for this event
	let uploadPrivacy = $derived(event.uploadPrivacyStatus ?? 'public');

	// Get visibility icon and style
	function getVisibilityStyle(privacy: YouTubePrivacyStatus): { class: string } {
		switch (privacy) {
			case 'public':
				return { class: 'text-green-600 bg-green-50 dark:bg-green-950' };
			case 'unlisted':
				return { class: 'text-yellow-600 bg-yellow-50 dark:bg-yellow-950' };
			case 'private':
				return { class: 'text-red-600 bg-red-50 dark:bg-red-950' };
		}
	}

	// Open video URL
	function openVideo(url: string) {
		window.open(url, '_blank');
	}
</script>

{#if recordings.length === 0}
	<div class="text-sm text-muted-foreground py-2">
		{$_('recordings.noRecordings') || 'No recordings yet'}
	</div>
{:else}
	<div class="space-y-2">
		{#each recordings as recording (recording.id)}
			{@const status = getRecordingStatus(recording)}
			{@const progress = getUploadProgress(recording)}
			<div class="flex items-start gap-2 py-1">
				<!-- Status icon -->
				<div class="mt-0.5">
					{#if status === 'uploaded'}
						<CheckCircle2 class="h-4 w-4 text-green-600" />
					{:else if status === 'uploading'}
						<Loader2 class="h-4 w-4 text-blue-600 animate-spin" />
					{:else if status === 'pending'}
						<Clock class="h-4 w-4 text-blue-600" />
					{:else}
						<AlertCircle class="h-4 w-4 text-amber-600" />
					{/if}
				</div>

				<!-- Recording info -->
				<div class="flex-1 min-w-0">
					<div class="flex items-center gap-2">
						<span class="text-sm font-medium truncate">{recording.file.name}</span>
						<span class="text-xs text-muted-foreground">
							{formatDuration(recording.file.duration)}
						</span>
						<!-- Visibility badge -->
						<span class="inline-flex items-center gap-0.5 text-[10px] font-medium px-1.5 py-0.5 rounded-full {getVisibilityStyle(uploadPrivacy).class}">
							{#if uploadPrivacy === 'public'}
								<Eye class="h-2.5 w-2.5" />
							{:else if uploadPrivacy === 'unlisted'}
								<Link class="h-2.5 w-2.5" />
							{:else}
								<EyeOff class="h-2.5 w-2.5" />
							{/if}
							{$_(`events.form.privacyOptions.${uploadPrivacy}`)}
						</span>
					</div>

					<!-- Upload progress bar -->
					{#if status === 'uploading'}
						<div class="mt-1.5 space-y-1">
							<div class="flex items-center justify-between text-xs">
								<span class="text-blue-600 font-medium">
									{$_('events.form.recording.status.uploading')}
								</span>
								<span class="text-muted-foreground">{progress}%</span>
							</div>
							<div class="h-1.5 w-full rounded-full bg-muted overflow-hidden">
								<div
									class="h-full rounded-full bg-blue-600 transition-all duration-300"
									style="width: {progress}%"
								></div>
							</div>
						</div>
					{:else if status === 'uploaded' && recording.videoUrl}
						<button
							type="button"
							onclick={() => recording.videoUrl && openVideo(recording.videoUrl)}
							class="flex items-center gap-1 text-xs text-blue-600 hover:underline mt-0.5"
						>
							<ExternalLink class="h-3 w-3" />
							{$_('recordings.viewOnYoutube') || 'View on YouTube'}
						</button>
					{:else if status === 'short' && !compact}
						<div class="flex items-center gap-2 mt-1">
							<button
								type="button"
								onclick={() => toggleWhitelist(recording.id, recording.whitelisted)}
								class="flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground"
							>
								{#if recording.whitelisted}
									<SquareCheck class="h-4 w-4" />
								{:else}
									<Square class="h-4 w-4" />
								{/if}
								{$_('recordings.includeInUpload') || 'Include in upload'}
							</button>
						</div>
					{/if}

					<!-- Custom title input for multiple recordings -->
					{#if needsSelection && status === 'pending' && !compact}
						<div class="mt-2">
							<Input
								type="text"
								placeholder={$_('recordings.customTitle') || 'Custom title (optional)'}
								value={recording.customTitle ?? ''}
								onchange={(e: Event) => updateTitle(recording.id, (e.target as HTMLInputElement).value)}
								class="text-xs h-7"
							/>
						</div>
					{/if}
				</div>

				<!-- Whitelist toggle for non-short recordings in selection mode -->
				{#if needsSelection && status === 'pending' && !compact}
					<button
						type="button"
						onclick={() => toggleWhitelist(recording.id, recording.whitelisted)}
						class="p-1 hover:bg-muted rounded"
						title={recording.whitelisted ? 'Remove from queue' : 'Add to queue'}
					>
						{#if recording.whitelisted}
							<SquareCheck class="h-4 w-4 text-green-600" />
						{:else}
							<Square class="h-4 w-4 text-muted-foreground" />
						{/if}
					</button>
				{/if}
			</div>
		{/each}

		<!-- Summary -->
		{#if !compact}
			<div class="pt-2 border-t border-border text-xs text-muted-foreground">
				{#if uploadableRecordings.length === 0}
					{$_('recordings.noneReady') || 'No recordings ready for upload'}
				{:else if uploadableRecordings.length === 1}
					{$_('recordings.oneReady') || '1 recording ready for upload'}
				{:else}
					{$_('recordings.multipleReady', { values: { count: uploadableRecordings.length } }) || `${uploadableRecordings.length} recordings ready`}
				{/if}
			</div>
		{/if}
	</div>
{/if}
