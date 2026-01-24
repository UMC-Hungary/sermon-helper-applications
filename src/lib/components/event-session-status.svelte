<script lang="ts">
	import { _ } from 'svelte-i18n';
	import { derived } from 'svelte/store';
	import Badge from '$lib/components/ui/badge.svelte';
	import Button from '$lib/components/ui/button.svelte';
	import Card from '$lib/components/ui/card.svelte';
	import Alert from '$lib/components/ui/alert.svelte';
	import AlertTitle from '$lib/components/ui/alert-title.svelte';
	import AlertDescription from '$lib/components/ui/alert-description.svelte';
	import { currentSession, sessionState } from '$lib/stores/event-session-store';
	import { eventList } from '$lib/stores/event-store';
	import { sessionIntegration } from '$lib/services/session-integration';
	import {
		Activity,
		RefreshCw,
		CheckCircle2,
		PauseCircle,
		AlertCircle,
		Loader2,
		Clock,
		Play,
		ExternalLink
	} from 'lucide-svelte';
	import type { EventSessionState } from '$lib/types/event-session';

	// Get the event associated with the current session
	const sessionEvent = derived(
		[currentSession, eventList],
		([$session, $events]) => {
			if (!$session) return null;
			return $events.find(e => e.id === $session.eventId) ?? null;
		}
	);

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
				return RefreshCw;
			case 'COMPLETED':
				return CheckCircle2;
			case 'PAUSED':
				return PauseCircle;
			default:
				return Clock;
		}
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

			<!-- Event title with link -->
			{#if $sessionEvent}
				<a
					href="/events/{$sessionEvent.id}"
					class="flex items-center gap-2 text-sm font-medium hover:text-primary transition-colors group"
				>
					<span class="truncate">{$sessionEvent.title}</span>
					<ExternalLink class="h-3 w-3 opacity-0 group-hover:opacity-100 transition-opacity shrink-0" />
				</a>
				<p class="text-xs text-muted-foreground -mt-2">
					{$sessionEvent.date} • {$sessionEvent.time}
				</p>
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
