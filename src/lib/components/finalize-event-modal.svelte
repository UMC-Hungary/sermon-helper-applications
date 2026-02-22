<script lang="ts">
	import { _ } from 'svelte-i18n';
	import Button from '$lib/components/ui/button.svelte';
	import { Loader2, CheckSquare } from 'lucide-svelte';
	import { eventStore } from '$lib/stores/event-store';
	import { toast } from '$lib/utils/toast';
	import { createEventRecording, type RecordingFile, type ServiceEvent } from '$lib/types/event';
	import { youtubeApi } from '$lib/utils/youtube-api';
	import FinalizeRecordingScan from './finalize-recording-scan.svelte';
	import FinalizeManualAdd from './finalize-manual-add.svelte';
	import RecordingsStatus from './recordings-status.svelte';

	interface Props {
		open: boolean;
		event: ServiceEvent;
		onClose: () => void;
	}

	let { open, event, onClose }: Props = $props();

	let dialogElement: HTMLDialogElement;
	let isSubmitting = $state(false);

	// Known paths from all events (for deduplication)
	let knownPaths = $derived(eventStore.getAllRecordingPaths());

	// Dialog open/close effect
	$effect(() => {
		if (open && dialogElement) {
			dialogElement.showModal();
		} else if (!open && dialogElement) {
			dialogElement.close();
		}
	});

	async function completeLiveBroadcast() {
		if (!event.youtubeScheduledId) return;
		if (event.youtubeLifeCycleStatus === 'complete') return;

		try {
			await youtubeApi.endBroadcast(event.youtubeScheduledId);
			eventStore.updateEvent(event.id, { youtubeLifeCycleStatus: 'complete' });
		} catch (error) {
			console.error('[FinalizeModal] Failed to complete live broadcast:', error);
		}
	}

	function handleRecordingFound(file: RecordingFile) {
		if (knownPaths.has(file.path)) return;
		const recording = createEventRecording(file);
		eventStore.addRecording(event.id, recording);
		eventStore.pushSessionActivity(event.id, 'RECORDING_MANUALLY_ADDED', file.name);
	}

	async function handleFinalize() {
		if (isSubmitting) return;
		isSubmitting = true;

		try {
			// Log scan activities
			eventStore.pushSessionActivity(event.id, 'RECORDING_SCAN_STARTED');
			eventStore.pushSessionActivity(event.id, 'RECORDING_SCAN_COMPLETED');

			// Complete live broadcast
			await completeLiveBroadcast();

			// Finalize session
			eventStore.setSessionFinalized(event.id);

			toast({
				title: $_('finalize.toast.finalized'),
				description: $_('finalize.toast.finalizedDescription'),
				variant: 'success'
			});

			onClose();
		} catch (error) {
			console.error('[FinalizeModal] Finalize failed:', error);
			toast({
				title: $_('toasts.error.title'),
				description: error instanceof Error ? error.message : 'Failed to finalize event',
				variant: 'error'
			});
		} finally {
			isSubmitting = false;
		}
	}

	function handleDialogClose() {
		onClose();
	}
</script>

<dialog
	bind:this={dialogElement}
	class="fixed z-50 bg-background border rounded-lg shadow-lg max-w-2xl w-full backdrop:bg-black/50 backdrop:backdrop-blur-sm p-0"
	onclose={handleDialogClose}
>
	<div class="p-6 space-y-6">
		<!-- Header -->
		<div class="space-y-1">
			<h2 class="text-lg font-semibold">{$_('finalize.title')}</h2>
		</div>

		<!-- Scan section -->
		<FinalizeRecordingScan
			{event}
			onRecordingFound={handleRecordingFound}
			{knownPaths}
		/>

		<!-- Manual add section -->
		<FinalizeManualAdd onFileAdded={handleRecordingFound} />

		<!-- Existing recordings -->
		<div class="space-y-3">
			<h3 class="text-sm font-medium">{$_('finalize.recordings.title')}</h3>
			<RecordingsStatus {event} />
		</div>

		<!-- Footer -->
		<div class="flex justify-end gap-2 pt-4 border-t">
			<Button buttonVariant="ghost" onclick={onClose} disabled={isSubmitting}>
				{$_('common.cancel')}
			</Button>
			<Button onclick={handleFinalize} disabled={isSubmitting}>
				{#if isSubmitting}
					<Loader2 class="h-4 w-4 animate-spin" />
				{:else}
					<CheckSquare class="h-4 w-4" />
				{/if}
				{$_('finalize.submit')}
			</Button>
		</div>
	</div>
</dialog>
