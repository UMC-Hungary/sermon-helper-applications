<script lang="ts">
  import { onMount } from 'svelte';
  import { listEvents } from '$lib/api/events.js';
  import { events, eventsLoading } from '$lib/stores/events.js';
  import { lastWsMessage } from '$lib/stores/ws.js';
  import EventList from '$lib/components/events/EventList.svelte';

  onMount(async () => {
    eventsLoading.set(true);
    try {
      const data = await listEvents();
      events.set(data);
    } finally {
      eventsLoading.set(false);
    }
  });

  $effect(() => {
    const msg = $lastWsMessage;
    if (msg?.type === 'event.changed' && msg.data.operation === 'INSERT') {
      // EventList will re-render from the ws/client.ts store update
    }
  });
</script>

<svelte:head>
  <title>Events — Sermon Helper</title>
</svelte:head>

<div class="page-header">
  <h1>Events</h1>
  <a href="/events/new" class="btn">+ New Event</a>
</div>

<EventList events={$events} loading={$eventsLoading} />

<style>
  .page-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 1.5rem;
  }

  .page-header h1 { margin: 0; }

  .btn {
    padding: 0.5rem 1rem;
    background: #2563eb;
    color: white;
    text-decoration: none;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    font-weight: 500;
  }
</style>
