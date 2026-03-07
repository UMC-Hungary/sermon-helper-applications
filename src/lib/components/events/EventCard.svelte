<script lang="ts">
  import type { EventSummary } from '$lib/schemas/event.js';

  interface Props {
    event: EventSummary;
  }

  let { event }: Props = $props();

  function formatDate(iso: string): string {
    return new Date(iso).toLocaleDateString(undefined, {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  }
</script>

<a class="card" class:card--completed={event.isCompleted} href="/events/{event.id}">
  <div class="card__header">
    <h3>{event.title}</h3>
    <div class="card__badges">
      {#if event.isCompleted}
        <span class="badge badge--completed">Completed</span>
      {/if}
      <span class="badge"
        >{event.recordingCount} recording{event.recordingCount !== 1 ? 's' : ''}</span
      >
    </div>
  </div>
  <p class="card__meta">
    {#if event.speaker}
      <span>{event.speaker}</span> &middot;
    {/if}
    <span>{formatDate(event.dateTime)}</span>
  </p>
</a>

<style>
  .card {
    display: block;
    padding: 1rem;
    border: 1px solid #e5e7eb;
    border-radius: 0.5rem;
    text-decoration: none;
    color: inherit;
    transition: border-color 0.15s;
  }

  .card:hover {
    border-color: #93c5fd;
  }

  .card--completed {
    opacity: 0.65;
  }

  .card__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    margin-bottom: 0.5rem;
  }

  .card__header h3 {
    margin: 0;
    font-size: 1rem;
  }

  .card__badges {
    display: flex;
    gap: 0.375rem;
    align-items: center;
  }

  .badge {
    font-size: 0.75rem;
    padding: 0.125rem 0.5rem;
    border-radius: 9999px;
    background: #e0e7ff;
    color: #3730a3;
    white-space: nowrap;
  }

  .badge--completed {
    background: #d1fae5;
    color: #065f46;
  }

  .card__meta {
    margin: 0;
    font-size: 0.875rem;
    color: #6b7280;
  }
</style>
