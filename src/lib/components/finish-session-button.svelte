<script lang="ts">
	import { Loader2, CheckSquare } from 'lucide-svelte';
	import Button from '$lib/components/ui/button.svelte';
	import { _ } from 'svelte-i18n';
	import { isStreaming, isRecording } from '$lib/stores/streaming-store';
	import { uploaderIntegration } from '$lib/services/uploader-integration';
	import { toast } from '$lib/utils/toast';
	import type { ServiceEvent } from '$lib/types/event';

	// Props
	export let event: ServiceEvent;

	// Local state
	let isFinishing = false;

	// Visible when session is ACTIVE and not streaming/recording
	$: canFinishSession = event.sessionState === 'ACTIVE' && !$isStreaming && !$isRecording;

	async function handleFinishSession() {
		if (isFinishing) return;
		isFinishing = true;
		try {
			await uploaderIntegration.triggerPostEventAutomation();
		} catch (error) {
			toast({
				title: $_('toasts.error.title'),
				description: error instanceof Error ? error.message : 'Failed to finish session',
				variant: 'error'
			});
		} finally {
			isFinishing = false;
		}
	}
</script>

<pre>{event.sessionState} - {!$isStreaming} - {!$isRecording}</pre>
{#if canFinishSession}
	<div class="pt-2 border-t border-border">
		<Button
			buttonVariant="outline"
			buttonSize="sm"
			className="w-full"
			onclick={handleFinishSession}
			disabled={isFinishing}
		>
			{#if isFinishing}
				<Loader2 class="h-4 w-4 animate-spin" />
			{:else}
				<CheckSquare class="h-4 w-4" />
			{/if}
			<span class="text-xs">{$_('eventSession.finishSession')}</span>
		</Button>
	</div>
{/if}
