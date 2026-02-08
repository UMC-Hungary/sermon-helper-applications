<script lang="ts">
	import { Youtube, XCircle, Loader2, LogIn, Play, ExternalLink } from 'lucide-svelte';
	import Button from '$lib/components/ui/button.svelte';
	import { _ } from 'svelte-i18n';
	import { systemStore } from '$lib/stores/system-store';
	import { youtubeApi } from '$lib/utils/youtube-api';
	import { scheduleYoutubeBroadcast } from '$lib/utils/youtube-helpers';
	import { refreshStore } from '$lib/stores/refresh-store';
	import { toast } from '$lib/utils/toast';
	import { isTauriApp } from '$lib/utils/storage-helpers';
	import type { ServiceEvent, YouTubeLifeCycleStatus } from '$lib/types/event';

	// Props
	export let event: ServiceEvent;
	export let onLoginRequired: (() => void) | undefined = undefined;

	// Local state
	let isGoingLive = false;

	// Computed
	$: youtubeLoggedIn = $systemStore.youtubeLoggedIn;
	$: hasScheduledId = event.youtubeScheduledId != null;
	$: isScheduling = event.isBroadcastScheduling ?? false;
	$: status = event.youtubeLifeCycleStatus as YouTubeLifeCycleStatus | undefined;

	// Lifecycle state checks
	$: isLive = status === 'live';
	$: isComplete = status === 'complete';
	$: isReady = status === 'ready' || status === 'testing';
	$: canGoLive = hasScheduledId && isReady && !isLive && !isComplete;

	// Schedule broadcast
	async function handleSchedule() {
		if (!event || event.youtubeScheduledId) return;
		await scheduleYoutubeBroadcast(event);
	}

	// Go live
	async function handleGoLive() {
		if (!event?.youtubeScheduledId || !canGoLive) return;

		isGoingLive = true;
		try {
			await youtubeApi.goLive(event.youtubeScheduledId);
			toast({
				title: $_('streaming.youtube.liveSuccess'),
				description: $_('streaming.youtube.liveSuccessDescription'),
				variant: 'success'
			});
			await refreshStore.triggerSync();
		} catch (error) {
			toast({
				title: $_('toasts.error.title'),
				description: error instanceof Error ? error.message : $_('streaming.youtube.liveFailed'),
				variant: 'error'
			});
		} finally {
			isGoingLive = false;
		}
	}

	// Open YouTube Studio
	async function openYoutubeStudio() {
		if (!event?.youtubeScheduledId) return;
		const url = youtubeApi.getYoutubeStudioUrl(event.youtubeScheduledId);
		if (isTauriApp()) {
			try {
				const { openUrl } = await import('@tauri-apps/plugin-opener');
				await openUrl(url);
			} catch (err) {
				console.error('Failed to open URL with Tauri opener:', err);
				window.open(url, '_blank');
			}
		} else {
			window.open(url, '_blank');
		}
	}

	// Status display text
	function getStatusText(s: YouTubeLifeCycleStatus | undefined): string {
		switch (s) {
			case 'created':
				return $_('streaming.youtube.status.created');
			case 'ready':
				return $_('streaming.youtube.status.ready');
			case 'testing':
				return $_('streaming.youtube.status.testing');
			case 'live':
				return $_('streaming.youtube.status.live');
			case 'complete':
				return $_('streaming.youtube.status.complete');
			default:
				return $_('streaming.youtube.status.unknown');
		}
	}
</script>

<div class="pt-2 border-t border-border">
	{#if !youtubeLoggedIn}
		<!-- Not logged in -->
		<div class="space-y-1">
			<div class="flex items-center gap-2">
				<XCircle class="h-4 w-4 text-muted-foreground shrink-0" />
				<span class="text-sm text-muted-foreground">{$_('sidebar.upcomingEvent.youtube.notScheduled')}</span>
			</div>
			{#if onLoginRequired}
				<Button
					buttonVariant="outline"
					buttonSize="sm"
					className="w-full"
					onclick={onLoginRequired}
				>
					<LogIn class="h-4 w-4 mr-2" />
					{$_('sidebar.upcomingEvent.youtube.loginFirst')}
				</Button>
			{/if}
		</div>
	{:else if !hasScheduledId}
		<!-- Logged in but not scheduled -->
		<div class="space-y-1">
			<div class="flex items-center gap-2">
				<XCircle class="h-4 w-4 text-muted-foreground shrink-0" />
				<span class="text-sm text-muted-foreground">{$_('sidebar.upcomingEvent.youtube.notScheduled')}</span>
			</div>
			<Button
				buttonVariant="outline"
				buttonSize="sm"
				className="w-full"
				onclick={handleSchedule}
				disabled={isScheduling}
			>
				{#if isScheduling}
					<Loader2 class="h-4 w-4 mr-2 animate-spin" />
					{$_('sidebar.upcomingEvent.youtube.scheduling')}
				{:else}
					<Youtube class="h-4 w-4 mr-2" />
					{$_('sidebar.upcomingEvent.youtube.schedule')}
				{/if}
			</Button>
		</div>
	{:else if isLive}
		<!-- Live -->
		<div class="flex items-center gap-2">
			<Youtube class="h-4 w-4 text-red-500 shrink-0" />
			<span class="relative flex h-2 w-2">
				<span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-red-500 opacity-75"></span>
				<span class="relative inline-flex rounded-full h-2 w-2 bg-red-500"></span>
			</span>
			<span class="text-sm font-medium text-red-500">{$_('streaming.youtube.status.live')}</span>
		</div>
	{:else if isComplete}
		<!-- Complete -->
		<div class="flex items-center gap-2">
			<Youtube class="h-4 w-4 text-muted-foreground shrink-0" />
			<span class="text-sm text-muted-foreground">{$_('streaming.youtube.status.complete')}</span>
		</div>
	{:else if isReady}
		<!-- Ready/Testing - show Go Live button -->
		<div class="flex items-center gap-2">
			<Youtube class="h-4 w-4 text-red-500 shrink-0" />
			<span class="text-sm text-muted-foreground flex-1">{getStatusText(status)}</span>
			{#if canGoLive}
				<Button
					buttonVariant="outline"
					buttonSize="sm"
					onclick={handleGoLive}
					disabled={isGoingLive}
				>
					{#if isGoingLive}
						<Loader2 class="h-4 w-4 animate-spin" />
					{:else}
						<Play class="h-4 w-4 fill-red-500 text-red-500" />
					{/if}
					<span class="text-xs">{$_('streaming.youtube.goLive')}</span>
				</Button>
			{/if}
		</div>
	{:else}
		<!-- Created / other status - show link to YouTube Studio -->
		<button
			type="button"
			onclick={openYoutubeStudio}
			class="flex items-center gap-2 w-full text-left hover:bg-muted/50 rounded p-1 -m-1 transition-colors"
		>
			<Youtube class="h-4 w-4 text-red-500 shrink-0" />
			<span class="text-sm text-green-600">{getStatusText(status)}</span>
			<ExternalLink class="h-3 w-3 text-muted-foreground shrink-0" />
		</button>
	{/if}
</div>
