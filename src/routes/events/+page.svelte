<script lang="ts">
	import { _ } from 'svelte-i18n';
	import { goto } from '$app/navigation';
	import Card from '$lib/components/ui/card.svelte';
	import Button from '$lib/components/ui/button.svelte';
	import Badge from '$lib/components/ui/badge.svelte';
	import { toast } from '$lib/utils/toast';
	import { eventStore, upcomingEvents, pastEvents, todayEvent } from '$lib/stores/event-store';
	import { formatEventDate, generateCalculatedTitle, type ServiceEvent } from '$lib/types/event';
	import { Plus, Calendar, Clock, User, BookOpen, Edit, Trash2, ChevronDown, ChevronUp } from 'lucide-svelte';

	// UI state
	let showPastEvents = $state(false);

	// Navigate to new event form
	function handleAdd() {
		goto('/events/new');
	}

	// Navigate to edit event form
	function handleEdit(event: ServiceEvent) {
		goto(`/events/${event.id}`);
	}

	// Delete event
	async function handleDelete(event: ServiceEvent) {
		if (!confirm($_('events.confirmDelete'))) return;

		try {
			await eventStore.deleteEvent(event.id);
			toast({
				title: $_('events.toasts.deleted.title'),
				description: $_('events.toasts.deleted.description'),
				variant: 'success'
			});
		} catch (error) {
			toast({
				title: $_('events.toasts.error.title'),
				description: error instanceof Error ? error.message : 'Unknown error',
				variant: 'error'
			});
		}
	}

	// Get badge variant for event
	function getEventBadge(event: ServiceEvent): { variant: 'default' | 'secondary' | 'success'; label: string } {
		const today = new Date().toISOString().split('T')[0];
		if (event.date === today) {
			return { variant: 'success', label: $_('events.badges.today') };
		} else if (event.date > today) {
			return { variant: 'default', label: $_('events.badges.upcoming') };
		}
		return { variant: 'secondary', label: $_('events.badges.past') };
	}
</script>

<!-- Page Header -->
<div class="mt-12 lg:mt-0 flex items-center justify-between">
	<div>
		<h2 class="text-3xl font-bold tracking-tight">{$_('events.title')}</h2>
		<p class="text-muted-foreground">{$_('events.subtitle')}</p>
	</div>
	<Button onclick={handleAdd}>
		<Plus class="mr-2 h-4 w-4" />
		{$_('events.addEvent')}
	</Button>
</div>

<!-- Today's Event -->
{#if $todayEvent}
	<Card className="border-green-500 border-2">
		<svelte:fragment slot="header">
			<div class="flex items-center justify-between w-full">
				<div class="flex items-center gap-2 flex-1 min-w-0">
					<Badge variant="success" className="shrink-0">{$_('events.badges.today')}</Badge>
					<span class="font-semibold text-sm truncate">{generateCalculatedTitle($todayEvent)}</span>
				</div>
				<div class="flex gap-2">
					<Button buttonVariant="outline" buttonSize="sm" onclick={() => handleEdit($todayEvent!)}>
						<Edit class="h-4 w-4" />
					</Button>
					<Button buttonVariant="outline" buttonSize="sm" onclick={() => handleDelete($todayEvent!)}>
						<Trash2 class="h-4 w-4 text-destructive" />
					</Button>
				</div>
			</div>
		</svelte:fragment>
		<svelte:fragment slot="content">
			<div class="grid gap-4 md:grid-cols-2">
				<div class="flex items-center gap-2 text-sm text-muted-foreground">
					<Clock class="h-4 w-4" />
					<span>{$todayEvent.time || $_('events.noTime')}</span>
				</div>
				<div class="flex items-center gap-2 text-sm text-muted-foreground">
					<User class="h-4 w-4" />
					<span>{$todayEvent.speaker || $_('events.noSpeaker')}</span>
				</div>
				{#if $todayEvent.textus}
					<div class="flex items-center gap-2 text-sm">
						<BookOpen class="h-4 w-4 text-muted-foreground" />
						<Badge variant="outline">{$_('events.textusLabel')}: {$todayEvent.textus}</Badge>
					</div>
				{/if}
				{#if $todayEvent.leckio}
					<div class="flex items-center gap-2 text-sm">
						<BookOpen class="h-4 w-4 text-muted-foreground" />
						<Badge variant="outline">{$_('events.leckioLabel')}: {$todayEvent.leckio}</Badge>
					</div>
				{/if}
			</div>
		</svelte:fragment>
	</Card>
{/if}

<!-- Upcoming Events -->
{#if $upcomingEvents.length > 0}
	<div class="space-y-4">
		<h3 class="text-xl font-semibold">{$_('events.upcomingTitle')}</h3>
		<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
			{#each $upcomingEvents.filter(e => e.id !== $todayEvent?.id) as event (event.id)}
				{@const badge = getEventBadge(event)}
				<Card>
					<svelte:fragment slot="header">
						<div class="flex items-center justify-between w-full">
							<Badge variant={badge.variant}>{badge.label}</Badge>
							<div class="flex gap-1">
								<Button buttonVariant="ghost" buttonSize="icon" onclick={() => handleEdit(event)}>
									<Edit class="h-4 w-4" />
								</Button>
								<Button buttonVariant="ghost" buttonSize="icon" onclick={() => handleDelete(event)}>
									<Trash2 class="h-4 w-4 text-destructive" />
								</Button>
							</div>
						</div>
					</svelte:fragment>
					<svelte:fragment slot="title">
						<span class="text-sm">{generateCalculatedTitle(event)}</span>
					</svelte:fragment>
					<svelte:fragment slot="content">
						<div class="space-y-2 text-sm">
							{#if event.time}
								<div class="flex items-center gap-2 text-muted-foreground">
									<Clock class="h-4 w-4" />
									<span>{event.time}</span>
								</div>
							{/if}
							{#if event.speaker}
								<div class="flex items-center gap-2 text-muted-foreground">
									<User class="h-4 w-4" />
									<span>{event.speaker}</span>
								</div>
							{/if}
						</div>
					</svelte:fragment>
				</Card>
			{/each}
		</div>
	</div>
{/if}

<!-- Empty state -->
{#if $upcomingEvents.length === 0}
	<Card>
		<svelte:fragment slot="content">
			<div class="text-center py-8">
				<Calendar class="h-12 w-12 mx-auto text-muted-foreground mb-4" />
				<h3 class="text-lg font-semibold mb-2">{$_('events.empty.title')}</h3>
				<p class="text-muted-foreground mb-4">{$_('events.empty.description')}</p>
				<Button onclick={handleAdd}>
					<Plus class="mr-2 h-4 w-4" />
					{$_('events.addEvent')}
				</Button>
			</div>
		</svelte:fragment>
	</Card>
{/if}

<!-- Past Events (collapsible) -->
{#if $pastEvents.length > 0}
	<div class="space-y-4">
		<button
			class="flex items-center gap-2 text-xl font-semibold text-muted-foreground hover:text-foreground transition-colors"
			onclick={() => (showPastEvents = !showPastEvents)}
		>
			{#if showPastEvents}
				<ChevronUp class="h-5 w-5" />
			{:else}
				<ChevronDown class="h-5 w-5" />
			{/if}
			{$_('events.pastTitle')} ({$pastEvents.length})
		</button>

		{#if showPastEvents}
			<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
				{#each $pastEvents as event (event.id)}
					<Card className="opacity-60">
						<svelte:fragment slot="header">
							<div class="flex items-center justify-between w-full">
								<Badge variant="secondary">{$_('events.badges.past')}</Badge>
								<Button buttonVariant="ghost" buttonSize="icon" onclick={() => handleDelete(event)}>
									<Trash2 class="h-4 w-4 text-destructive" />
								</Button>
							</div>
						</svelte:fragment>
						<svelte:fragment slot="title">
							<span class="text-sm">{generateCalculatedTitle(event)}</span>
						</svelte:fragment>
						<svelte:fragment slot="content">
							<div class="space-y-2 text-sm text-muted-foreground">
								{#if event.speaker}
									<div class="flex items-center gap-2">
										<User class="h-4 w-4" />
										<span>{event.speaker}</span>
									</div>
								{/if}
							</div>
						</svelte:fragment>
					</Card>
				{/each}
			</div>
		{/if}
	</div>
{/if}
