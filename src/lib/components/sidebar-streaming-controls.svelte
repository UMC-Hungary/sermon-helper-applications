<script lang="ts">
	import { Radio, Circle, Play, Loader2, Square, Youtube } from 'lucide-svelte';
	import { cn } from '$lib/utils.js';
	import Button from '$lib/components/ui/button.svelte';
	import { _ } from 'svelte-i18n';
	import { systemStore } from '$lib/stores/system-store';
	import {
		isStreaming,
		isRecording,
		isStreamTransitioningStore,
		isRecordTransitioningStore,
		streamTimecode,
		recordTimecode,
		streamControls,
		recordControls
	} from '$lib/stores/streaming-store';
	import { youtubeApi } from '$lib/utils/youtube-api';
	import { refreshStore } from '$lib/stores/refresh-store';
	import { toast } from '$lib/utils/toast';
	import type { ServiceEvent, YouTubeLifeCycleStatus } from '$lib/types/event';

	// Props
	export let event: ServiceEvent | null = null;

	// Local state for YouTube Go Live
	let isGoingLive = false;

	// Computed states
	$: isOBSConnected = $systemStore.obs;
	$: hasYoutubeBroadcast = event?.youtubeScheduledId != null;
	$: youtubeStatus = event?.youtubeLifeCycleStatus as YouTubeLifeCycleStatus | undefined;

	// YouTube status checks
	$: isYoutubeLive = youtubeStatus === 'live';
	$: isYoutubeComplete = youtubeStatus === 'complete';
	$: isYoutubeReady = youtubeStatus === 'ready' || youtubeStatus === 'testing';
	$: canGoLive = hasYoutubeBroadcast && isYoutubeReady && !isYoutubeLive && !isYoutubeComplete;

	// Handle stream toggle
	async function handleStreamToggle() {
		try {
			await streamControls.toggle();
		} catch (error) {
			toast({
				title: $_('toasts.error.title'),
				description: error instanceof Error ? error.message : $_('streaming.errors.streamFailed'),
				variant: 'error'
			});
		}
	}

	// Handle record toggle
	async function handleRecordToggle() {
		try {
			await recordControls.toggle();
		} catch (error) {
			toast({
				title: $_('toasts.error.title'),
				description: error instanceof Error ? error.message : $_('streaming.errors.recordFailed'),
				variant: 'error'
			});
		}
	}

	// Handle YouTube Go Live
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
			// Sync YouTube status immediately after going live
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

	// Get YouTube status display text
	function getYoutubeStatusText(status: YouTubeLifeCycleStatus | undefined): string {
		switch (status) {
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

<div class="space-y-3">
	<!-- Stream and Record Controls -->
	<div class="flex gap-2">
		<!-- Stream Button -->
		<div class="flex-1 flex flex-col items-center">
			<Button
				buttonVariant={$isStreaming ? 'destructive' : 'outline'}
				buttonSize="sm"
				className={cn(
					'w-full relative',
					$isStreaming && 'bg-red-600 hover:bg-red-700'
				)}
				onclick={handleStreamToggle}
				disabled={!isOBSConnected || $isStreamTransitioningStore}
			>
				{#if $isStreamTransitioningStore}
					<Loader2 class="h-4 w-4 animate-spin" />
				{:else if $isStreaming}
					<span class="relative flex h-3 w-3 mr-1">
						<span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-white opacity-75"></span>
						<span class="relative inline-flex rounded-full h-3 w-3 bg-white"></span>
					</span>
					<Square class="h-3 w-3" />
				{:else}
					<Radio class="h-4 w-4" />
				{/if}
				<span class="text-xs">{$_('streaming.stream')}</span>
			</Button>
			{#if $isStreaming}
				<span class="text-xs text-muted-foreground mt-1 font-mono">{$streamTimecode}</span>
			{/if}
		</div>

		<!-- Record Button -->
		<div class="flex-1 flex flex-col items-center">
			<Button
				buttonVariant={$isRecording ? 'destructive' : 'outline'}
				buttonSize="sm"
				className={cn(
					'w-full relative',
					$isRecording && 'bg-red-600 hover:bg-red-700'
				)}
				onclick={handleRecordToggle}
				disabled={!isOBSConnected || $isRecordTransitioningStore}
			>
				{#if $isRecordTransitioningStore}
					<Loader2 class="h-4 w-4 animate-spin" />
				{:else if $isRecording}
					<span class="relative flex h-3 w-3 mr-1">
						<span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-white opacity-75"></span>
						<span class="relative inline-flex rounded-full h-3 w-3 bg-white"></span>
					</span>
					<Square class="h-3 w-3" />
				{:else}
					<Circle class="h-4 w-4 fill-current" />
				{/if}
				<span class="text-xs">{$_('streaming.record')}</span>
			</Button>
			{#if $isRecording}
				<span class="text-xs text-muted-foreground mt-1 font-mono">{$recordTimecode}</span>
			{/if}
		</div>
	</div>

	<!-- YouTube Status and Controls (only shown when broadcast is scheduled) -->
	{#if hasYoutubeBroadcast && $systemStore.youtubeLoggedIn}
		<div class="flex items-center gap-2 pt-2 border-t border-border">
			<Youtube class="h-4 w-4 text-red-500 shrink-0" />

			{#if isYoutubeLive}
				<!-- Live indicator -->
				<div class="flex items-center gap-2 flex-1">
					<span class="relative flex h-2 w-2">
						<span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-red-500 opacity-75"></span>
						<span class="relative inline-flex rounded-full h-2 w-2 bg-red-500"></span>
					</span>
					<span class="text-sm font-medium text-red-500">{$_('streaming.youtube.status.live')}</span>
				</div>
			{:else if isYoutubeComplete}
				<!-- Completed indicator -->
				<span class="text-sm text-muted-foreground flex-1">{$_('streaming.youtube.status.complete')}</span>
			{:else}
				<!-- Status + Go Live button -->
				<span class="text-sm text-muted-foreground flex-1">{getYoutubeStatusText(youtubeStatus)}</span>
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
			{/if}
		</div>
	{/if}

	<!-- OBS Not Connected Warning -->
	{#if !isOBSConnected}
		<p class="text-xs text-muted-foreground text-center">
			{$_('streaming.obsNotConnected')}
		</p>
	{/if}
</div>
