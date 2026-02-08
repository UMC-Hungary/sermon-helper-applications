<script lang="ts">
	import { Radio, Circle, Loader2 } from 'lucide-svelte';
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
	import { toast } from '$lib/utils/toast';
	import FinishSessionButton from '$lib/components/finish-session-button.svelte';
	import YoutubeStatus from '$lib/components/youtube-status.svelte';
	import type { ServiceEvent } from '$lib/types/event';

	// Props
	export let event: ServiceEvent | null = null;

	// Computed states
	$: isOBSConnected = $systemStore.obs;

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

	<!-- YouTube Status -->
	{#if event}
		<YoutubeStatus {event} />
	{/if}

	<!-- Finish Session Button -->
	{#if event}
		<FinishSessionButton {event} />
	{/if}

	<!-- OBS Not Connected Warning -->
	{#if !isOBSConnected}
		<p class="text-xs text-muted-foreground text-center">
			{$_('streaming.obsNotConnected')}
		</p>
	{/if}
</div>
