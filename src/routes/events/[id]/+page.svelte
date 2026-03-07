<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { getEvent, deleteEvent } from '$lib/api/events.js';
  import { listRecordings, deleteRecording } from '$lib/api/recordings.js';
  import { triggerYouTubeSchedule, triggerFacebookSchedule } from '$lib/api/connectors.js';
  import { listActivities, createActivity, deleteActivity } from '$lib/api/activities.js';
  import type { Event, EventActivity } from '$lib/schemas/event.js';
  import type { Recording } from '$lib/schemas/recording.js';
  import RecordingList from '$lib/components/recordings/RecordingList.svelte';
  import CreateRecordingForm from '$lib/components/recordings/CreateRecordingForm.svelte';
  import { youtubeStatus, facebookStatus } from '$lib/stores/connectors.js';

  let event = $state<Event | null>(null);
  let recordings = $state<Recording[]>([]);
  let activities = $state<EventActivity[]>([]);
  let loadingEvent = $state(true);
  let loadingRecordings = $state(true);
  let error = $state('');
  let showAddRecording = $state(false);
  let schedulingYt = $state(false);
  let schedulingFb = $state(false);
  let scheduleError = $state('');
  let togglingCompletion = $state(false);
  let deletingEvent = $state(false);

  const id = page.params.id ?? '';

  const isPast = $derived(event ? new Date(event.dateTime) <= new Date() : false);
  const isCompleted = $derived(activities.some((a) => a.activityType === 'completed'));
  const completedActivity = $derived(activities.find((a) => a.activityType === 'completed'));

  function conn(platform: string) {
    return event?.connections.find((c) => c.platform === platform);
  }

  onMount(async () => {
    try {
      [event, recordings, activities] = await Promise.all([
        getEvent(id),
        listRecordings(id),
        listActivities(id),
      ]);
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      loadingEvent = false;
      loadingRecordings = false;
    }
  });

  function formatDate(iso: string): string {
    return new Date(iso).toLocaleString(undefined, {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  }

  function handleRecordingCreated(rec: Recording) {
    recordings = [rec, ...recordings];
    showAddRecording = false;
  }

  async function scheduleYouTube() {
    if (!event) return;
    schedulingYt = true;
    scheduleError = '';
    try {
      await triggerYouTubeSchedule(event.id);
      // Reload event to get updated status
      event = await getEvent(id);
    } catch (e) {
      scheduleError = e instanceof Error ? e.message : String(e);
    } finally {
      schedulingYt = false;
    }
  }

  async function scheduleFacebook() {
    if (!event) return;
    schedulingFb = true;
    scheduleError = '';
    try {
      await triggerFacebookSchedule(event.id);
      event = await getEvent(id);
    } catch (e) {
      scheduleError = e instanceof Error ? e.message : String(e);
    } finally {
      schedulingFb = false;
    }
  }

  async function toggleCompletion() {
    togglingCompletion = true;
    try {
      if (isCompleted && completedActivity) {
        await deleteActivity(id, completedActivity.id);
        activities = activities.filter((a) => a.id !== completedActivity!.id);
      } else {
        const act = await createActivity(id, { activity_type: 'completed' });
        activities = [...activities, act];
      }
    } catch (e) {
      scheduleError = e instanceof Error ? e.message : String(e);
    } finally {
      togglingCompletion = false;
    }
  }

  async function handleDeleteEvent() {
    if (!confirm('Delete this event and all its recordings? This cannot be undone.')) return;
    deletingEvent = true;
    try {
      await deleteEvent(id);
      await goto('/events');
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      deletingEvent = false;
    }
  }

  async function handleDeleteRecording(recordingId: string, deleteFile: boolean) {
    await deleteRecording(id, recordingId, deleteFile);
    recordings = recordings.filter((r) => r.id !== recordingId);
  }
</script>

<svelte:head>
  <title>{event?.title ?? 'Event'} — Sermon Helper</title>
</svelte:head>

<div class="nav">
  <a href="/events" class="back">&larr; Events</a>
  {#if event}
    <div class="nav__actions">
      <a href="/events/{id}/edit" class="btn-edit">Edit</a>
      <button class="btn-danger" onclick={handleDeleteEvent} disabled={deletingEvent}>
        {deletingEvent ? 'Deleting…' : 'Delete Event'}
      </button>
    </div>
  {/if}
</div>

{#if error}
  <p class="error" role="alert">{error}</p>
{:else if loadingEvent}
  <p>Loading…</p>
{:else if event}
  <h1>{event.title}</h1>

  <dl class="meta">
    {#if event.speaker}
      <dt>Speaker</dt>
      <dd>{event.speaker}</dd>
    {/if}
    <dt>Date &amp; Time</dt>
    <dd>{formatDate(event.dateTime)}</dd>
    {#if event.description}
      <dt>Description</dt>
      <dd>{event.description}</dd>
    {/if}
    <dt>YouTube Visibility</dt>
    <dd>{conn('youtube')?.privacyStatus === 'public' ? 'Public' : conn('youtube')?.privacyStatus === 'unlisted' ? 'Unlisted' : 'Private'}</dd>
    <dt>Facebook Visibility</dt>
    <dd>{conn('facebook')?.privacyStatus === 'EVERYONE' ? 'Public' : conn('facebook')?.privacyStatus === 'FRIENDS' ? 'Friends' : 'Only Me'}</dd>
  </dl>

  <!-- Social connector status & actions -->
  <section class="social">
    <h2>Social Platforms</h2>

    {#if scheduleError}
      <p class="error" role="alert">{scheduleError}</p>
    {/if}

    <div class="social-grid">
      <!-- YouTube -->
      <div class="social-item">
        <div class="social-item__header">
          <span class="social-item__name">YouTube</span>
          <span class="badge badge--{conn('youtube')?.scheduleStatus === 'scheduled' || conn('youtube')?.externalId ? 'scheduled' : conn('youtube')?.scheduleStatus === 'failed' ? 'failed' : 'none'}">
            {conn('youtube')?.scheduleStatus === 'scheduled' || conn('youtube')?.externalId ? 'Scheduled' : conn('youtube')?.scheduleStatus === 'failed' ? 'Failed' : 'Not Scheduled'}
          </span>
        </div>
        {#if conn('youtube')?.streamUrl}
          <a href={conn('youtube')?.streamUrl} target="_blank" rel="noopener noreferrer" class="view-link">
            View on YouTube
          </a>
        {/if}
        {#if $youtubeStatus === 'connected' && conn('youtube')?.scheduleStatus !== 'scheduled' && !conn('youtube')?.externalId}
          <button
            class="btn-secondary"
            onclick={scheduleYouTube}
            disabled={schedulingYt || isPast}
            title={isPast ? 'Event date is in the past — edit the event to reschedule' : undefined}
          >
            {schedulingYt ? 'Scheduling…' : 'Schedule on YouTube'}
          </button>
          {#if isPast}
            <p class="past-warning">Event date has passed. Update the date to schedule.</p>
          {/if}
        {/if}
      </div>

      <!-- Facebook -->
      <div class="social-item">
        <div class="social-item__header">
          <span class="social-item__name">Facebook</span>
          <span class="badge badge--{conn('facebook')?.scheduleStatus === 'scheduled' ? 'scheduled' : conn('facebook')?.scheduleStatus === 'failed' ? 'failed' : 'none'}">
            {conn('facebook')?.scheduleStatus === 'scheduled' ? 'Scheduled' : conn('facebook')?.scheduleStatus === 'failed' ? 'Failed' : 'Not Scheduled'}
          </span>
        </div>
        {#if conn('facebook')?.eventUrl}
          <a href={conn('facebook')?.eventUrl} target="_blank" rel="noopener noreferrer" class="view-link">
            View on Facebook
          </a>
        {/if}
        {#if $facebookStatus === 'connected' && conn('facebook')?.scheduleStatus !== 'scheduled'}
          <button
            class="btn-secondary"
            onclick={scheduleFacebook}
            disabled={schedulingFb || isPast}
            title={isPast ? 'Event date is in the past — edit the event to reschedule' : undefined}
          >
            {schedulingFb ? 'Scheduling…' : 'Schedule on Facebook'}
          </button>
          {#if isPast}
            <p class="past-warning">Event date has passed. Update the date to schedule.</p>
          {/if}
        {/if}
      </div>
    </div>
  </section>

  <section class="status-section">
    <h2>Event Status</h2>
    <div class="status-row">
      <span class="status-label">
        {isCompleted ? 'This event is marked as completed.' : 'This event is not yet completed.'}
      </span>
      <button
        class="btn-status"
        class:btn-status--undo={isCompleted}
        onclick={toggleCompletion}
        disabled={togglingCompletion}
      >
        {#if togglingCompletion}
          …
        {:else if isCompleted}
          Undo Completion
        {:else}
          Mark as Completed
        {/if}
      </button>
    </div>
  </section>

  <section class="recordings">
    <div class="recordings__header">
      <h2>Recordings</h2>
      <button
        onclick={() => {
          showAddRecording = !showAddRecording;
        }}
      >
        {showAddRecording ? 'Cancel' : '+ Add Recording'}
      </button>
    </div>

    {#if showAddRecording}
      <div class="recordings__form">
        <CreateRecordingForm eventId={id} oncreated={handleRecordingCreated} />
      </div>
    {/if}

    <RecordingList {recordings} loading={loadingRecordings} ondelete={handleDeleteRecording} />
  </section>
{/if}

<style>
  .nav {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 1rem;
  }

  .nav__actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .back {
    color: #6b7280;
    text-decoration: none;
    font-size: 0.875rem;
  }
  .back:hover {
    color: #374151;
  }

  .btn-edit {
    padding: 0.375rem 0.875rem;
    background: transparent;
    color: #2563eb;
    border: 1px solid #2563eb;
    border-radius: 0.375rem;
    text-decoration: none;
    font-size: 0.875rem;
    font-weight: 500;
  }
  .btn-edit:hover {
    background: #eff6ff;
  }

  .btn-danger {
    padding: 0.375rem 0.875rem;
    background: transparent;
    color: #dc2626;
    border: 1px solid #dc2626;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
  }
  .btn-danger:hover:not(:disabled) {
    background: #fef2f2;
  }
  .btn-danger:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  h1 {
    margin-bottom: 1.5rem;
  }

  .meta {
    display: grid;
    grid-template-columns: max-content 1fr;
    gap: 0.5rem 1rem;
    margin-bottom: 2rem;
  }
  .meta dt {
    font-weight: 600;
    color: #374151;
  }
  .meta dd {
    margin: 0;
    color: #6b7280;
  }

  .recordings__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 1rem;
  }
  .recordings__header h2 {
    margin: 0;
  }
  .recordings__header button {
    padding: 0.5rem 1rem;
    background: #2563eb;
    color: white;
    border: none;
    border-radius: 0.375rem;
    cursor: pointer;
    font-size: 0.875rem;
  }
  .recordings__header button:hover {
    background: #1d4ed8;
  }

  .recordings__form {
    padding: 1rem;
    background: #f9fafb;
    border-radius: 0.5rem;
    margin-bottom: 1rem;
  }

  .error {
    padding: 0.75rem;
    background: #fee2e2;
    color: #991b1b;
    border-radius: 0.375rem;
  }

  .social {
    margin-bottom: 2rem;
  }

  .social h2 {
    margin-bottom: 1rem;
  }

  .social-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
  }

  .social-item {
    padding: 1rem;
    border: 1px solid #e5e7eb;
    border-radius: 0.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .social-item__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .social-item__name {
    font-weight: 600;
    color: #374151;
  }

  .badge {
    font-size: 0.75rem;
    padding: 0.2rem 0.5rem;
    border-radius: 9999px;
    font-weight: 500;
  }

  .badge--scheduled {
    background: #d1fae5;
    color: #065f46;
  }

  .badge--failed {
    background: #fee2e2;
    color: #991b1b;
  }

  .badge--none {
    background: #f3f4f6;
    color: #6b7280;
  }

  .view-link {
    font-size: 0.875rem;
    color: #2563eb;
    text-decoration: none;
  }

  .view-link:hover {
    text-decoration: underline;
  }

  .past-warning {
    font-size: 0.75rem;
    color: #92400e;
    background: #fef3c7;
    border: 1px solid #fcd34d;
    border-radius: 0.25rem;
    padding: 0.25rem 0.5rem;
    margin: 0;
  }

  .btn-secondary {
    padding: 0.375rem 0.875rem;
    background: transparent;
    color: #2563eb;
    border: 1px solid #2563eb;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    cursor: pointer;
    align-self: flex-start;
  }

  .btn-secondary:hover:not(:disabled) {
    background: #eff6ff;
  }

  .btn-secondary:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .status-section {
    margin-bottom: 2rem;
    padding: 1rem;
    border: 1px solid #e5e7eb;
    border-radius: 0.5rem;
  }

  .status-section h2 {
    margin: 0 0 0.75rem;
  }

  .status-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
  }

  .status-label {
    font-size: 0.875rem;
    color: #6b7280;
  }

  .btn-status {
    padding: 0.375rem 0.875rem;
    background: #2563eb;
    color: #fff;
    border: none;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    cursor: pointer;
    white-space: nowrap;
  }

  .btn-status:hover:not(:disabled) {
    background: #1d4ed8;
  }

  .btn-status--undo {
    background: transparent;
    color: #6b7280;
    border: 1px solid #d1d5db;
  }

  .btn-status--undo:hover:not(:disabled) {
    background: #f3f4f6;
  }

  .btn-status:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
</style>
