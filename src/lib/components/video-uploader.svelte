<script lang="ts">
	import { _ } from 'svelte-i18n';
	import Button from '$lib/components/ui/button.svelte';
	import Badge from '$lib/components/ui/badge.svelte';
	import Card from '$lib/components/ui/card.svelte';
	import Alert from '$lib/components/ui/alert.svelte';
	import AlertTitle from '$lib/components/ui/alert-title.svelte';
	import AlertDescription from '$lib/components/ui/alert-description.svelte';
	import {
		unifiedUploads,
		hasUploads,
		uploadCount,
		type UnifiedUploadItem
	} from '$lib/stores/unified-uploads-store';
	import { pendingUploadsStore } from '$lib/stores/pending-uploads-store';
	import { systemStore } from '$lib/stores/system-store';
	import {
		Play,
		X,
		RefreshCw,
		AlertCircle,
		Youtube,
		ExternalLink,
		Upload,
		Loader2
	} from 'lucide-svelte';
	import { toast } from 'svelte-sonner';

	let resumingId: string | null = null;

	// Check if YouTube is connected
	$: isYouTubeConnected = $systemStore.youtubeLoggedIn;

	// Check if any upload needs YouTube but it's not connected
	$: showYouTubeWarning = $hasUploads && !isYouTubeConnected &&
		$unifiedUploads.some(u => u.platform === 'youtube');

	// Get status icon component
	function getStatusIcon(status: UnifiedUploadItem['status']) {
		switch (status) {
			case 'uploading':
				return Loader2;
			case 'processing':
				return RefreshCw;
			case 'paused':
			case 'pending':
				return Upload;
			case 'failed':
				return AlertCircle;
			default:
				return Upload;
		}
	}

	// Get status badge variant
	function getStatusVariant(
		status: UnifiedUploadItem['status']
	): 'default' | 'secondary' | 'destructive' | 'warning' | 'success' {
		switch (status) {
			case 'uploading':
				return 'default';
			case 'processing':
				return 'default';
			case 'paused':
				return 'warning';
			case 'pending':
				return 'secondary';
			case 'failed':
				return 'destructive';
			default:
				return 'secondary';
		}
	}

	// Check if status icon should animate
	function shouldAnimate(status: UnifiedUploadItem['status']): boolean {
		return status === 'uploading' || status === 'processing';
	}

	// Format progress
	function formatProgress(item: UnifiedUploadItem): string {
		if (!item.progress) return '';
		return `${item.progress.percentage.toFixed(0)}%`;
	}

	// Handle resume
	async function handleResume(item: UnifiedUploadItem) {
		if (item.source === 'session') {
			// Session uploads are managed by the automation
			toast.info($_('videoUploader.sessionManaged') || 'This upload is managed by the current session');
			return;
		}

		resumingId = item.id;
		try {
			await pendingUploadsStore.resumeUpload(item.id);
			toast.success($_('videoUploader.resumed') || 'Upload resumed');
		} catch (error) {
			toast.error(
				$_('videoUploader.resumeFailed') || 'Failed to resume upload',
				{ description: error instanceof Error ? error.message : String(error) }
			);
		} finally {
			resumingId = null;
		}
	}

	// Handle cancel
	async function handleCancel(item: UnifiedUploadItem) {
		if (item.source === 'session') {
			toast.info($_('videoUploader.sessionManaged') || 'This upload is managed by the current session');
			return;
		}

		try {
			await pendingUploadsStore.cancelUpload(item.id);
			toast.success($_('videoUploader.cancelled') || 'Upload cancelled');
		} catch (error) {
			toast.error($_('videoUploader.cancelFailed') || 'Failed to cancel upload');
		}
	}

	// Handle resume all
	async function handleResumeAll() {
		resumingId = 'all';
		try {
			await pendingUploadsStore.resumeAll();
			toast.success($_('videoUploader.allResumed') || 'All uploads resumed');
		} catch (error) {
			toast.error($_('videoUploader.resumeAllFailed') || 'Failed to resume uploads');
		} finally {
			resumingId = null;
		}
	}

	// Get resumable items count (not from session, not uploading)
	$: resumableCount = $unifiedUploads.filter(
		u => u.source === 'event' && (u.status === 'paused' || u.status === 'failed' || u.status === 'pending')
	).length;
</script>

{#if $hasUploads || showYouTubeWarning}
	<Card className="p-3">
		<div class="space-y-3">
			<!-- Header -->
			<div class="flex items-center justify-between">
				<h4 class="text-xs font-medium text-muted-foreground uppercase tracking-wide flex items-center gap-2">
					{$_('videoUploader.title') || 'Video Uploader'}
					{#if $uploadCount > 0}
						<Badge variant="secondary" className="text-xs">{$uploadCount}</Badge>
					{/if}
				</h4>
			</div>

			<!-- YouTube not connected warning -->
			{#if showYouTubeWarning}
				<Alert variant="warning" className="py-2">
					<Youtube class="h-4 w-4" />
					<AlertTitle className="text-xs">
						{$_('videoUploader.youtubeNotConnected') || 'YouTube Not Connected'}
					</AlertTitle>
					<AlertDescription className="text-xs">
						{$_('videoUploader.connectToUpload') || 'Connect to YouTube to upload recordings.'}
					</AlertDescription>
				</Alert>
			{/if}

			<!-- Upload list -->
			{#if $hasUploads}
				<div class="space-y-2">
					{#each $unifiedUploads as item (item.id)}
						{@const StatusIcon = getStatusIcon(item.status)}
						<div class="space-y-1.5 p-2 bg-muted/50 rounded-md">
							<!-- Title row with link -->
							<div class="flex items-start gap-2">
								<Youtube class="h-4 w-4 text-muted-foreground shrink-0 mt-0.5" />
								<div class="flex-1 min-w-0">
									<a
										href="/events/{item.eventId}"
										class="text-sm font-medium hover:text-primary transition-colors group flex items-center gap-1"
									>
										<span class="line-clamp-2">{item.calculatedTitle}</span>
										<ExternalLink class="h-3 w-3 opacity-0 group-hover:opacity-100 transition-opacity shrink-0" />
									</a>
								</div>
							</div>

							<!-- Status row -->
							<div class="flex items-center gap-2 pl-6">
								<Badge variant={getStatusVariant(item.status)} className="text-[10px] px-1.5 py-0 flex items-center gap-1">
									<StatusIcon class="h-3 w-3 {shouldAnimate(item.status) ? 'animate-spin' : ''}" />
									{$_(`videoUploader.status.${item.status}`) || item.status}
									{#if item.progress && item.status === 'uploading'}
										<span>{formatProgress(item)}</span>
									{/if}
								</Badge>

								<!-- Action buttons -->
								<div class="flex gap-1 ml-auto">
									{#if item.source === 'event' && (item.status === 'paused' || item.status === 'failed' || item.status === 'pending')}
										<Button
											buttonSize="icon"
											buttonVariant="ghost"
											className="h-6 w-6"
											onclick={() => handleResume(item)}
											disabled={resumingId !== null || !isYouTubeConnected}
											title={$_('videoUploader.resume') || 'Resume'}
										>
											{#if resumingId === item.id}
												<RefreshCw class="h-3 w-3 animate-spin" />
											{:else}
												<Play class="h-3 w-3" />
											{/if}
										</Button>
									{/if}
									{#if item.source === 'event'}
										<Button
											buttonSize="icon"
											buttonVariant="ghost"
											className="h-6 w-6 text-muted-foreground hover:text-destructive"
											onclick={() => handleCancel(item)}
											disabled={resumingId !== null}
											title={$_('videoUploader.cancel') || 'Cancel'}
										>
											<X class="h-3 w-3" />
										</Button>
									{/if}
								</div>
							</div>

							<!-- Progress bar (if uploading) -->
							{#if item.progress && item.status === 'uploading'}
								<div class="pl-6">
									<div class="h-1 bg-muted rounded-full overflow-hidden">
										<div
											class="h-full bg-primary transition-all duration-300"
											style="width: {item.progress.percentage}%"
										></div>
									</div>
								</div>
							{/if}

							<!-- Error message -->
							{#if item.error}
								<p class="text-xs text-destructive flex items-center gap-1 pl-6">
									<AlertCircle class="h-3 w-3 shrink-0" />
									<span class="line-clamp-1">{item.error}</span>
								</p>
							{/if}
						</div>
					{/each}
				</div>

				<!-- Resume All button -->
				{#if resumableCount > 1}
					<Button
						buttonVariant="outline"
						buttonSize="sm"
						className="w-full text-xs"
						onclick={handleResumeAll}
						disabled={resumingId !== null || !isYouTubeConnected}
					>
						{#if resumingId === 'all'}
							<RefreshCw class="h-3 w-3 mr-1 animate-spin" />
						{:else}
							<RefreshCw class="h-3 w-3 mr-1" />
						{/if}
						{$_('videoUploader.resumeAll') || 'Resume All'}
					</Button>
				{/if}
			{/if}
		</div>
	</Card>
{/if}
