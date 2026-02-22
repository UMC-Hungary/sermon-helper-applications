<script lang="ts">
  import { goto } from '$app/navigation';
  import CreateEventForm from '$lib/components/events/CreateEventForm.svelte';
  import type { Event } from '$lib/types/event.js';

  function handleCreated(event: Event) {
    goto(`/events/${event.id}`);
  }
</script>

<svelte:head>
  <title>New Event — Sermon Helper</title>
</svelte:head>

<a href="/events" class="back">&larr; Events</a>
<h1>Create Event</h1>
<CreateEventForm oncreated={handleCreated} />

<style>
  .back { display: inline-block; margin-bottom: 1rem; color: #6b7280; text-decoration: none; font-size: 0.875rem; }
  .back:hover { color: #374151; }
  h1 { margin-bottom: 1.5rem; }
</style>
