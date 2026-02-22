<script lang="ts">
	import { CheckSquare } from 'lucide-svelte';
	import Button from '$lib/components/ui/button.svelte';
	import { _ } from 'svelte-i18n';
	import { isStreaming, isRecording } from '$lib/stores/streaming-store';
	import { deriveSessionState, type ServiceEvent } from '$lib/types/event';
	import FinalizeEventModal from './finalize-event-modal.svelte';

	interface Props {
		event: ServiceEvent;
	}

	let { event }: Props = $props();

	let showModal = $state(false);

	// Visible when session is not COMPLETED (always available to finalize)
	let sessionState = $derived(deriveSessionState(event.activities));
	let canFinishSession = $derived(sessionState !== 'COMPLETED' && !$isStreaming && !$isRecording);
</script>

{#if canFinishSession}
	<div class="pt-2 border-t border-border">
		<Button
			buttonVariant="outline"
			buttonSize="sm"
			className="w-full"
			onclick={() => (showModal = true)}
		>
			<CheckSquare class="h-4 w-4" />
			<span class="text-xs">{$_('eventSession.finishSession')}</span>
		</Button>
	</div>

	<FinalizeEventModal
		open={showModal}
		{event}
		onClose={() => (showModal = false)}
	/>
{/if}
