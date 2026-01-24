<script lang="ts">
	import { _ } from 'svelte-i18n';
	import Button from '$lib/components/ui/button.svelte';
	import Badge from '$lib/components/ui/badge.svelte';
	import {
		pendingUploadsStore,
		pendingUploads,
		hasPendingUploads,
		pendingUploadCount
	} from '$lib/stores/pending-uploads-store';
	import { Play, X, RefreshCw, AlertCircle, Youtube, Calendar } from 'lucide-svelte';
	import { toast } from 'svelte-sonner';

	let resumingId: string | null = null;

	// Format time ago
	function formatTimeAgo(timestamp: number): string {
		const seconds = Math.floor((Date.now() - timestamp) / 1000);
		if (seconds < 60) return $_('upload.pending.justNow') || 'just now';
		const minutes = Math.floor(seconds / 60);
		if (minutes < 60) return `${minutes}m ${$_('upload.pending.ago') || 'ago'}`;
		const hours = Math.floor(minutes / 60);
		if (hours < 24) return `${hours}h ${$_('upload.pending.ago') || 'ago'}`;
		const days = Math.floor(hours / 24);
		return `${days}d ${$_('upload.pending.ago') || 'ago'}`;
	}

	// Format event date
	function formatEventDate(dateStr: string): string {
		const date = new Date(dateStr);
		return date.toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
	}

	// Get platform icon
	function getPlatformIcon(platform: string) {
		switch (platform) {
			case 'youtube':
				return Youtube;
			default:
				return RefreshCw;
		}
	}

	// Get status badge variant
	function getStatusVariant(
		status: string
	): 'default' | 'secondary' | 'destructive' | 'warning' | 'success' {
		switch (status) {
			case 'paused':
				return 'warning';
			case 'failed':
				return 'destructive';
			case 'uploading':
				return 'default';
			case 'pending':
				return 'secondary';
			default:
				return 'secondary';
		}
	}

	async function handleResume(sessionId: string) {
		resumingId = sessionId;
		try {
			await pendingUploadsStore.resumeUpload(sessionId);
			toast.success($_('upload.pending.resumed') || 'Upload resumed');
		} catch (error) {
			toast.error(
				$_('upload.pending.resumeFailed') || 'Failed to resume upload',
				{ description: error instanceof Error ? error.message : String(error) }
			);
		} finally {
			resumingId = null;
		}
	}

	async function handleCancel(sessionId: string) {
		try {
			await pendingUploadsStore.cancelUpload(sessionId);
			toast.success($_('upload.pending.cancelled') || 'Upload cancelled');
		} catch (error) {
			toast.error($_('upload.pending.cancelFailed') || 'Failed to cancel upload');
		}
	}

	async function handleResumeAll() {
		resumingId = 'all';
		try {
			await pendingUploadsStore.resumeAll();
			toast.success($_('upload.pending.allResumed') || 'All uploads resumed');
		} catch (error) {
			toast.error($_('upload.pending.resumeAllFailed') || 'Failed to resume uploads');
		} finally {
			resumingId = null;
		}
	}
</script>

{#if $hasPendingUploads}
	<div class="space-y-2">
		<div class="flex items-center justify-between">
			<h4 class="text-xs font-medium text-muted-foreground uppercase tracking-wide flex items-center gap-2">
				{$_('upload.pending.title') || 'Pending Uploads'}
				<Badge variant="secondary" className="text-xs">{$pendingUploadCount}</Badge>
			</h4>
		</div>

		<div class="space-y-2">
			{#each $pendingUploads as { event, session } (session.id)}
				{@const PlatformIcon = getPlatformIcon(session.platform)}
				<div class="flex items-center gap-2 p-2 bg-muted/50 rounded-md">
					<PlatformIcon class="h-4 w-4 text-muted-foreground shrink-0" />
					<div class="flex-1 min-w-0">
						<p class="text-sm font-medium truncate">{session.metadata.title}</p>
						<div class="flex items-center gap-2 text-xs text-muted-foreground">
							<Badge variant={getStatusVariant(session.status)} className="text-[10px] px-1.5 py-0">
								{$_(`upload.status.${session.status}`) || session.status}
							</Badge>
							<span class="flex items-center gap-1">
								<Calendar class="h-3 w-3" />
								{formatEventDate(event.date)}
							</span>
							<span>{formatTimeAgo(session.startedAt)}</span>
						</div>
						{#if session.error}
							<p class="text-xs text-destructive flex items-center gap-1 mt-1">
								<AlertCircle class="h-3 w-3" />
								{session.error}
							</p>
						{/if}
					</div>
					<div class="flex gap-1 shrink-0">
						{#if session.status === 'paused' || session.status === 'failed'}
							<Button
								buttonSize="icon"
								buttonVariant="ghost"
								className="h-7 w-7"
								onclick={() => handleResume(session.id)}
								disabled={resumingId !== null}
							>
								{#if resumingId === session.id}
									<RefreshCw class="h-4 w-4 animate-spin" />
								{:else}
									<Play class="h-4 w-4" />
								{/if}
							</Button>
						{/if}
						<Button
							buttonSize="icon"
							buttonVariant="ghost"
							className="h-7 w-7 text-muted-foreground hover:text-destructive"
							onclick={() => handleCancel(session.id)}
							disabled={resumingId !== null}
						>
							<X class="h-4 w-4" />
						</Button>
					</div>
				</div>
			{/each}
		</div>

		{#if $pendingUploadCount > 1}
			<Button
				buttonVariant="outline"
				buttonSize="sm"
				className="w-full text-xs"
				onclick={handleResumeAll}
				disabled={resumingId !== null}
			>
				{#if resumingId === 'all'}
					<RefreshCw class="h-3 w-3 mr-1 animate-spin" />
				{:else}
					<RefreshCw class="h-3 w-3 mr-1" />
				{/if}
				{$_('upload.pending.resumeAll') || 'Resume All'}
			</Button>
		{/if}
	</div>
{/if}
