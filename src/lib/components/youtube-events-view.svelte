<script lang="ts">
	import Card from '$lib/components/ui/card.svelte';
	import CardHeader from '$lib/components/ui/card-header.svelte';
	import CardContent from '$lib/components/ui/card-content.svelte';
	import CardTitle from '$lib/components/ui/card-title.svelte';
	import CardDescription from '$lib/components/ui/card-description.svelte';
	import Badge from '$lib/components/ui/badge.svelte';
	import Button from '$lib/components/ui/button.svelte';
	import ErrorMessages from '$lib/components/ui/error-messages.svelte';
	import { Calendar, Clock, User, Eye, Edit, Trash2 } from 'lucide-svelte';
	import { toast } from '$lib/utils/toast';
	import { systemStore } from '$lib/stores/system-store';
	import type { SystemStatus } from '$lib/stores/types';
	import type { YoutubeEvent } from '$lib/types';

	interface YoutubeEventsViewProps {
		systemStatus: SystemStatus;
		onRecheck: () => void;
	}

	let { systemStatus, onRecheck }: YoutubeEventsViewProps = $props();

	const mockEvents: YoutubeEvent[] = [
		{
			id: '1',
			title: 'Sunday Service - The Love of God',
			speaker: 'Pastor John Smith',
			date: '2025-01-05',
			time: '10:00',
			privacy: 'public',
			status: 'scheduled'
		},
		{
			id: '2',
			title: 'Wednesday Evening Service',
			speaker: 'Rev. Mary Johnson',
			date: '2025-01-08',
			time: '19:00',
			privacy: 'public',
			status: 'scheduled'
		},
		{
			id: '3',
			title: 'Youth Service - Faith in Action',
			speaker: 'Pastor David Lee',
			date: '2025-01-12',
			time: '18:30',
			privacy: 'unlisted',
			status: 'scheduled'
		}
	];

	const handleEdit = (eventId: string) => {
		toast({
			title: 'Edit Event',
			description: 'Opening event editor...'
		});
	};

	const handleDelete = (eventId: string) => {
		toast({
			title: 'Event Deleted',
			description: 'YouTube event has been removed',
			variant: 'error'
		});
	};

	const getStatusBadgeVariant = (status: YoutubeEvent['status']): 'default' | 'destructive' | 'secondary' => {
		const variants = {
			scheduled: 'default',
			live: 'destructive',
			completed: 'secondary'
		} as const;
		return variants[status];
	};

	const getPrivacyIcon = (privacy: YoutubeEvent['privacy']) => {
		return Eye;
	};
</script>

<div class="p-4 lg:p-8 space-y-6 pt-20 lg:pt-8">
	<ErrorMessages {onRecheck} />

	<div class="flex items-center justify-between">
		<div>
			<h2 class="text-3xl font-bold tracking-tight">YouTube Events</h2>
			<p class="text-muted-foreground">View and manage scheduled YouTube live events</p>
		</div>
		<Badge variant="outline" className="text-lg px-4 py-2">
			{mockEvents.length} Events
		</Badge>
	</div>

	<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
		{#each mockEvents as event (event.id)}
			<Card>
				<CardHeader>
					<div class="flex items-start justify-between">
						<CardTitle class="text-lg leading-tight">{event.title}</CardTitle>
						<Badge variant={getStatusBadgeVariant(event.status)}>{event.status}</Badge>
					</div>
					<CardDescription class="flex items-center gap-1 mt-2">
						<User class="h-3 w-3" />
						{event.speaker}
					</CardDescription>
				</CardHeader>
				<CardContent class="space-y-4">
					<div class="space-y-2 text-sm">
						<div class="flex items-center gap-2 text-muted-foreground">
							<Calendar class="h-4 w-4" />
							<span>
								{new Date(event.date).toLocaleDateString('en-US', {
									weekday: 'short',
									year: 'numeric',
									month: 'short',
									day: 'numeric'
								})}
							</span>
						</div>
						<div class="flex items-center gap-2 text-muted-foreground">
							<Clock class="h-4 w-4" />
							<span>{event.time}</span>
						</div>
						<div class="flex items-center gap-2 text-muted-foreground">
							<Eye class="h-4 w-4" />
							<span class="capitalize">{event.privacy}</span>
						</div>
					</div>

					<div class="flex gap-2">
						<Button
							buttonVariant="outline"
							buttonSize="sm"
							className="flex-1 bg-transparent"
							onclick={() => handleEdit(event.id)}
						>
							<Edit class="h-4 w-4 mr-1" />
							Edit
						</Button>
						<Button
							buttonVariant="outline"
							buttonSize="sm"
							className="flex-1 bg-transparent"
							onclick={() => handleDelete(event.id)}
						>
							<Trash2 class="h-4 w-4 mr-1" />
							Delete
						</Button>
					</div>
				</CardContent>
			</Card>
		{/each}
	</div>
</div>
