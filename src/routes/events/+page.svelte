<script lang="ts">
	import { _ } from 'svelte-i18n';
	import { goto } from '$app/navigation';
	import Card from '$lib/components/ui/card.svelte';
	import Button from '$lib/components/ui/button.svelte';
	import Badge from '$lib/components/ui/badge.svelte';
	import { toast } from '$lib/utils/toast';
	import { eventStore, eventList, upcomingEvents, pastEvents, todayEvent } from '$lib/stores/event-store';
	import { isYouTubeConnected } from '$lib/stores/youtube-store';
	import { uploadManager } from '$lib/services/upload/upload-manager';
	import {
		formatEventDate,
		formatEventTime,
		formatDuration,
		generateCalculatedTitle,
		getRecordingStatus,
		getYouTubeVideoUrl,
		getEventDate,
		getLocalToday,
		type ServiceEvent,
		type EventRecording,
		type EventRecordingStatus
	} from '$lib/types/event';
	import { Plus, Calendar, Clock, User, BookOpen, Edit, Trash2, ChevronDown, ChevronUp, Video, CheckCircle, XCircle, Upload, Globe, Link, Lock, ExternalLink, Loader2 } from 'lucide-svelte';

	// UI state
	let showPastEvents = $state(false);
	let showRecordings = $state(false);

	// Upload state
	let uploadingRecordingId = $state<string | null>(null);
	let uploadProgress = $state(0);

	// Flat list of all recordings across all events, sorted by most recent first
	let allRecordings = $derived(
		$eventList
			.flatMap((event) =>
				(event.recordings ?? []).map((recording) => ({ recording, event }))
			)
			.sort((a, b) => b.recording.file.createdAt - a.recording.file.createdAt)
	);

	// Pending recordings (not uploaded, eligible), sorted oldest first for sequential upload
	let pendingRecordings = $derived(
		allRecordings.filter(({ recording }) => {
			const s = getRecordingItemStatus(recording);
			return s === 'pending' || s === 'uploading';
		})
	);

	// Whether YouTube is connected (from shared store)
	let youtubeConnected = $derived($isYouTubeConnected);

	// Check if a recording is the next in line to upload (first pending)
	function isNextToUpload(recordingId: string): boolean {
		if (pendingRecordings.length === 0) return false;
		// Last in the list = oldest (sorted most recent first), so the next to upload is the last pending
		return pendingRecordings[pendingRecordings.length - 1].recording.id === recordingId;
	}

	// Start a fresh upload for a recording
	async function handleUpload(event: ServiceEvent, recording: EventRecording) {
		if (uploadingRecordingId) return;

		// If this recording has a saved session, resume instead
		if (recording.uploadSession) {
			return handleResumeUpload(event, recording);
		}

		uploadingRecordingId = recording.id;
		uploadProgress = 0;

		try {
			const result = await uploadManager.uploadRecordingManual(
				event.id,
				recording,
				(progress) => {
					uploadProgress = progress.percentage;
				}
			);

			if (result) {
				toast({
					title: $_('toasts.success.title'),
					description: $_('recordings.table.uploadSuccess'),
					variant: 'success'
				});
			} else {
				await eventStore.clearRecordingUploading(event.id, recording.id);
			}
		} catch (error) {
			await eventStore.clearRecordingUploading(event.id, recording.id);
			toast({
				title: $_('toasts.error.title'),
				description: $_('recordings.table.uploadFailed'),
				variant: 'error'
			});
		} finally {
			uploadingRecordingId = null;
			uploadProgress = 0;
		}
	}

	// Resume an interrupted upload from persisted session
	async function handleResumeUpload(event: ServiceEvent, recording: EventRecording) {
		if (uploadingRecordingId) return;

		uploadingRecordingId = recording.id;
		// Start from last persisted progress
		const savedSession = recording.uploadSession;
		uploadProgress = savedSession ? (savedSession.bytesUploaded / savedSession.fileSize) * 100 : 0;

		try {
			const result = await uploadManager.resumeUploadRecording(
				event.id,
				recording,
				(progress) => {
					uploadProgress = progress.percentage;
				}
			);

			if (result) {
				toast({
					title: $_('toasts.success.title'),
					description: $_('recordings.table.uploadSuccess'),
					variant: 'success'
				});
			} else {
				await eventStore.clearRecordingUploading(event.id, recording.id);
			}
		} catch (error) {
			// Resume failed — keep session data so user can retry, but clear in-progress flag
			await eventStore.clearRecordingUploading(event.id, recording.id);
			toast({
				title: $_('toasts.error.title'),
				description: $_('recordings.table.uploadFailed'),
				variant: 'error'
			});
		} finally {
			uploadingRecordingId = null;
			uploadProgress = 0;
		}
	}

	// Get recording-level status for a single recording
	function getRecordingItemStatus(recording: EventRecording): EventRecordingStatus {
		if (recording.uploaded) return 'uploaded';
		if (recording.uploadSession) return 'uploading';
		const meetsMinDuration = recording.file.duration >= 120; // 2 minutes
		if (meetsMinDuration || recording.whitelisted) return 'pending';
		return 'none';
	}

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
		const today = getLocalToday();
		const eventDate = getEventDate(event);
		if (eventDate === today) {
			return { variant: 'success', label: $_('events.badges.today') };
		} else if (eventDate > today) {
			return { variant: 'default', label: $_('events.badges.upcoming') };
		}
		return { variant: 'secondary', label: $_('events.badges.past') };
	}

	// Get recording status badge variant
	function getRecordingBadgeVariant(status: EventRecordingStatus): 'default' | 'secondary' | 'success' | 'destructive' {
		switch (status) {
			case 'uploaded':
				return 'success';
			case 'failed':
				return 'destructive';
			case 'uploading':
			// TODO: add case 'paused':
			case 'pending':
				return 'default';
			default:
				return 'secondary';
		}
	}

	// Get privacy icon component
	function getPrivacyIcon(privacy: string): typeof Globe {
		switch (privacy) {
			case 'public':
				return Globe;
			case 'unlisted':
				return Link;
			case 'private':
				return Lock;
			default:
				return Globe;
		}
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
					<span>{formatEventTime($todayEvent.dateTime) || $_('events.noTime')}</span>
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
							{#if formatEventTime(event.dateTime)}
								<div class="flex items-center gap-2 text-muted-foreground">
									<Clock class="h-4 w-4" />
									<span>{formatEventTime(event.dateTime)}</span>
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
					{@const recordingStatus = getRecordingStatus(event)}
					{@const videoUrl = getYouTubeVideoUrl(event)}
					<Card className="opacity-80 hover:opacity-100 transition-opacity">
						<svelte:fragment slot="header">
							<div class="flex items-center justify-between w-full">
								<div class="flex items-center gap-2">
									<Badge variant="secondary">{$_('events.badges.past')}</Badge>
									<!-- Recording status badge -->
									{#if recordingStatus !== 'none'}
										<Badge variant={getRecordingBadgeVariant(recordingStatus)}>
											{#if recordingStatus === 'uploaded'}
												<CheckCircle class="h-3 w-3 mr-1" />
											{:else if recordingStatus === 'failed'}
												<XCircle class="h-3 w-3 mr-1" />
											{:else if recordingStatus === 'uploading'}
												<Upload class="h-3 w-3 mr-1" />
											{:else}
												<Video class="h-3 w-3 mr-1" />
											{/if}
											{$_(`events.form.recording.status.${recordingStatus}`)}
										</Badge>
									{/if}
								</div>
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
							<div class="space-y-2 text-sm text-muted-foreground">
								{#if event.speaker}
									<div class="flex items-center gap-2">
										<User class="h-4 w-4" />
										<span>{event.speaker}</span>
									</div>
								{/if}
								<!-- Watch Recording link for uploaded events -->
								{#if recordingStatus === 'uploaded' && videoUrl}
									<div class="flex items-center gap-2 pt-1">
										<Button
											buttonVariant="outline"
											buttonSize="sm"
											href={videoUrl}
											target="_blank"
										>
											<ExternalLink class="h-3 w-3 mr-1" />
											{$_('events.form.recording.watchRecording')}
										</Button>
										<!-- Visibility badge -->
										<Badge variant="secondary" className="text-xs">
											{#if event.uploadPrivacyStatus === 'public'}
												<Globe class="h-3 w-3 mr-1" />
											{:else if event.uploadPrivacyStatus === 'unlisted'}
												<Link class="h-3 w-3 mr-1" />
											{:else}
												<Lock class="h-3 w-3 mr-1" />
											{/if}
											{$_(`events.form.privacyOptions.${event.uploadPrivacyStatus || 'public'}`)}
										</Badge>
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

<!-- Recordings Table (collapsible) -->
{#if allRecordings.length > 0}
	<div class="space-y-4">
		<button
			class="flex items-center gap-2 text-xl font-semibold text-muted-foreground hover:text-foreground transition-colors"
			onclick={() => (showRecordings = !showRecordings)}
		>
			{#if showRecordings}
				<ChevronUp class="h-5 w-5" />
			{:else}
				<ChevronDown class="h-5 w-5" />
			{/if}
			{$_('recordings.table.title')} ({allRecordings.length})
		</button>

		{#if showRecordings}
			<div class="overflow-x-auto rounded-lg border border-border">
				<table class="w-full text-sm">
					<thead>
						<tr class="border-b border-border bg-muted/50">
							<th class="px-4 py-3 text-left font-medium text-muted-foreground">{$_('recordings.table.recording')}</th>
							<th class="px-4 py-3 text-left font-medium text-muted-foreground">{$_('recordings.table.duration')}</th>
							<th class="px-4 py-3 text-left font-medium text-muted-foreground">{$_('recordings.table.event')}</th>
							<th class="px-4 py-3 text-left font-medium text-muted-foreground">{$_('recordings.table.date')}</th>
							<th class="px-4 py-3 text-left font-medium text-muted-foreground">{$_('recordings.table.speaker')}</th>
							<th class="px-4 py-3 text-left font-medium text-muted-foreground">{$_('recordings.table.status')}</th>
							<th class="px-4 py-3 text-left font-medium text-muted-foreground">{$_('recordings.table.youtube')}</th>
							<th class="px-4 py-3 text-left font-medium text-muted-foreground">{$_('recordings.table.actions')}</th>
						</tr>
					</thead>
					<tbody>
						{#each allRecordings as { recording, event } (recording.id)}
							{@const status = getRecordingItemStatus(recording)}
							<tr class="border-b border-border last:border-b-0 hover:bg-muted/30 transition-colors">
								<td class="px-4 py-3 font-medium max-w-[200px] truncate" title={recording.file.name}>
									{recording.file.name}
								</td>
								<td class="px-4 py-3 text-muted-foreground whitespace-nowrap">
									{formatDuration(recording.file.duration)}
								</td>
								<td class="px-4 py-3 max-w-[250px] truncate" title={generateCalculatedTitle(event)}>
									{generateCalculatedTitle(event)}
								</td>
								<td class="px-4 py-3 text-muted-foreground whitespace-nowrap">
									{formatEventDate(event.dateTime)}
								</td>
								<td class="px-4 py-3 text-muted-foreground">
									{event.speaker || '-'}
								</td>
								<td class="px-4 py-3">
									<Badge variant={getRecordingBadgeVariant(status)}>
										{#if status === 'uploaded'}
											<CheckCircle class="h-3 w-3 mr-1" />
										{:else if status === 'uploading'}
											<Upload class="h-3 w-3 mr-1" />
										{:else if status === 'pending'}
											<Video class="h-3 w-3 mr-1" />
										{/if}
										{$_(`events.form.recording.status.${status}`)}
									</Badge>
								</td>
								<td class="px-4 py-3">
									{#if recording.videoUrl}
										<a
											href={recording.videoUrl}
											target="_blank"
											rel="noopener noreferrer"
											class="text-primary hover:text-primary/80 transition-colors"
											title={$_('recordings.viewOnYoutube')}
										>
											<ExternalLink class="h-4 w-4" />
										</a>
									{:else}
										<span class="text-muted-foreground">-</span>
									{/if}
								</td>
								<td class="px-4 py-3">
									<div class="flex items-center gap-2">
										{#if (status === 'pending' || status === 'uploading') && youtubeConnected}
											<Button
												buttonVariant={isNextToUpload(recording.id) ? 'default' : 'outline'}
												buttonSize="sm"
												disabled={uploadingRecordingId !== null && uploadingRecordingId !== recording.id}
												onclick={() => handleUpload(event, recording)}
											>
												{#if uploadingRecordingId === recording.id}
													<Loader2 class="h-3 w-3 mr-1 animate-spin" />
													{uploadProgress > 0 ? `${Math.round(uploadProgress)}%` : $_('recordings.table.uploading')}
												{:else}
													<Upload class="h-3 w-3 mr-1" />
													{$_('recordings.table.upload')}
												{/if}
											</Button>
										{/if}
										<Button buttonVariant="outline" buttonSize="sm" onclick={() => handleEdit(event)}>
											<Edit class="h-3 w-3 mr-1" />
											{$_('recordings.table.editEvent')}
										</Button>
									</div>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/if}
	</div>
{/if}
