<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { getEvent } from '$lib/api/events.js';
  import { listRecordings } from '$lib/api/recordings.js';
  import type { Event } from '$lib/types/event.js';
  import type { Recording } from '$lib/types/recording.js';
  import RecordingList from '$lib/components/recordings/RecordingList.svelte';
  import CreateRecordingForm from '$lib/components/recordings/CreateRecordingForm.svelte';

  let event = $state<Event | null>(null);
  let recordings = $state<Recording[]>([]);
  let loadingEvent = $state(true);
  let loadingRecordings = $state(true);
  let error = $state('');
  let showAddRecording = $state(false);

  const id = page.params.id ?? '';

  onMount(async () => {
    try {
      [event, recordings] = await Promise.all([
        getEvent(id),
        listRecordings(id),
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
</script>

<svelte:head>
  <title>{event?.title ?? 'Event'} — Sermon Helper</title>
</svelte:head>

<a href="/events" class="back">&larr; Events</a>

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
    <dt>YouTube Privacy</dt>
    <dd>{event.youtubePrivacyStatus}</dd>
  </dl>

  <section class="recordings">
    <div class="recordings__header">
      <h2>Recordings</h2>
      <button onclick={() => { showAddRecording = !showAddRecording; }}>
        {showAddRecording ? 'Cancel' : '+ Add Recording'}
      </button>
    </div>

    {#if showAddRecording}
      <div class="recordings__form">
        <CreateRecordingForm eventId={id} oncreated={handleRecordingCreated} />
      </div>
    {/if}

    <RecordingList {recordings} loading={loadingRecordings} />
  </section>
{/if}

<style>
  .back { display: inline-block; margin-bottom: 1rem; color: #6b7280; text-decoration: none; font-size: 0.875rem; }
  .back:hover { color: #374151; }

  h1 { margin-bottom: 1.5rem; }

  .meta { display: grid; grid-template-columns: max-content 1fr; gap: 0.5rem 1rem; margin-bottom: 2rem; }
  .meta dt { font-weight: 600; color: #374151; }
  .meta dd { margin: 0; color: #6b7280; }

  .recordings__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 1rem;
  }
  .recordings__header h2 { margin: 0; }
  .recordings__header button {
    padding: 0.5rem 1rem;
    background: #2563eb;
    color: white;
    border: none;
    border-radius: 0.375rem;
    cursor: pointer;
    font-size: 0.875rem;
  }
  .recordings__header button:hover { background: #1d4ed8; }

  .recordings__form {
    padding: 1rem;
    background: #f9fafb;
    border-radius: 0.5rem;
    margin-bottom: 1rem;
  }

  .error { padding: 0.75rem; background: #fee2e2; color: #991b1b; border-radius: 0.375rem; }
</style>
