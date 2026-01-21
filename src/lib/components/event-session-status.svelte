<script lang="ts">
	import { _ } from 'svelte-i18n';
	import Badge from '$lib/components/ui/badge.svelte';
	import Button from '$lib/components/ui/button.svelte';
	import Card from '$lib/components/ui/card.svelte';
	import Alert from '$lib/components/ui/alert.svelte';
	import AlertTitle from '$lib/components/ui/alert-title.svelte';
	import AlertDescription from '$lib/components/ui/alert-description.svelte';
	import { currentSession, sessionState, uploadProgress } from '$lib/stores/event-session-store';
	import { sessionIntegration } from '$lib/services/session-integration';
	import {
		Activity,
		Upload,
		CheckCircle2,
		PauseCircle,
		AlertCircle,
		Loader2,
		Clock,
		Play
	} from 'lucide-svelte';
	import type { EventSessionState, PlatformUploadProgress } from '$lib/types/event-session';

	// Get badge variant for state
	function getStateVariant(
		state: EventSessionState
	): 'default' | 'secondary' | 'success' | 'warning' | 'destructive' {
		switch (state) {
			case 'IDLE':
				return 'secondary';
			case 'PREPARING':
				return 'default';
			case 'ACTIVE':
				return 'success';
			case 'FINALIZING':
				return 'default';
			case 'COMPLETED':
				return 'success';
			case 'PAUSED':
				return 'warning';
			default:
				return 'secondary';
		}
	}

	// Get icon for state
	function getStateIcon(state: EventSessionState) {
		switch (state) {
			case 'IDLE':
				return Clock;
			case 'PREPARING':
				return Loader2;
			case 'ACTIVE':
				return Activity;
			case 'FINALIZING':
				return Upload;
			case 'COMPLETED':
				return CheckCircle2;
			case 'PAUSED':
				return PauseCircle;
			default:
				return Clock;
		}
	}

	// Format progress percentage
	function formatProgress(progress: PlatformUploadProgress): string {
		if (progress.status === 'completed') return '100%';
		if (progress.status === 'pending') return '0%';
		return `${progress.percentage.toFixed(0)}%`;
	}

	// Handle manual trigger of post-event automation
	async function handleTriggerAutomation() {
		await sessionIntegration.triggerPostEventAutomation();
	}

	$: StateIcon = getStateIcon($sessionState);
	$: stateVariant = getStateVariant($sessionState);
	$: isAnimated = $sessionState === 'PREPARING' || $sessionState === 'FINALIZING';
</script>

{#if $currentSession}
	<Card className="p-3">
		<div class="space-y-3">
			<!-- Header with state badge -->
			<div class="flex items-center justify-between">
				<h4 class="text-xs font-medium text-muted-foreground uppercase tracking-wide">
					{$_('eventSession.title') || 'Session'}
				</h4>
				<Badge variant={stateVariant} className="text-xs">
					<svelte:component
						this={StateIcon}
						class="h-3 w-3 mr-1 {isAnimated ? 'animate-spin' : ''}"
					/>
					{$_(`eventSession.states.${$sessionState.toLowerCase()}`) || $sessionState}
				</Badge>
			</div>

			<!-- Upload Progress (when finalizing) -->
			{#if $sessionState === 'FINALIZING' && $uploadProgress.length > 0}
				<div class="space-y-2">
					{#each $uploadProgress as platform}
						<div class="space-y-1">
							<div class="flex items-center justify-between text-xs">
								<span class="text-muted-foreground">{platform.name}</span>
								<span class="font-medium">{formatProgress(platform)}</span>
							</div>
							<div class="h-1.5 bg-muted rounded-full overflow-hidden">
								<div
									class="h-full bg-primary transition-all duration-300"
									style="width: {platform.percentage}%"
								></div>
							</div>
						</div>
					{/each}
				</div>
			{/if}

			<!-- Paused state warning -->
			{#if $sessionState === 'PAUSED'}
				<Alert variant="warning" className="py-2">
					<AlertCircle class="h-4 w-4" />
					<AlertTitle className="text-xs">
						{$_('eventSession.connection.lost') || 'Connection Lost'}
					</AlertTitle>
					<AlertDescription className="text-xs">
						{$currentSession.pauseReason || $_('eventSession.connection.reconnecting')}
					</AlertDescription>
				</Alert>
			{/if}

			<!-- Completed state -->
			{#if $sessionState === 'COMPLETED'}
				<div class="flex items-center gap-2 text-xs text-green-600">
					<CheckCircle2 class="h-4 w-4" />
					<span>{$_('eventSession.automation.completed') || 'Automation completed'}</span>
				</div>
			{/if}

			<!-- Error state -->
			{#if $currentSession.completionError}
				<Alert variant="destructive" className="py-2">
					<AlertCircle class="h-4 w-4" />
					<AlertTitle className="text-xs">{$_('common.error') || 'Error'}</AlertTitle>
					<AlertDescription className="text-xs">
						{$currentSession.completionError}
					</AlertDescription>
				</Alert>
			{/if}

			<!-- Manual trigger button (for ACTIVE state when all outputs stopped) -->
			{#if $sessionState === 'ACTIVE' && $currentSession.recordEndedAt}
				<Button
					buttonVariant="outline"
					buttonSize="sm"
					className="w-full text-xs"
					onclick={handleTriggerAutomation}
				>
					<Play class="h-3 w-3 mr-1" />
					{$_('eventSession.automation.triggerManually') || 'Start Post-Event Processing'}
				</Button>
			{/if}
		</div>
	</Card>
{/if}
