<script lang="ts">
  import { createEvent } from '$lib/api/events.js';
  import type { Event } from '$lib/types/event.js';

  interface Props {
    oncreated?: (event: Event) => void;
  }

  let { oncreated }: Props = $props();

  let title = $state('');
  let dateTime = $state('');
  let speaker = $state('');
  let description = $state('');
  let submitting = $state(false);
  let error = $state('');

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    submitting = true;
    error = '';

    try {
      const created = await createEvent({
        title,
        date_time: new Date(dateTime).toISOString(),
        speaker: speaker || undefined,
        description: description || undefined,
      });
      oncreated?.(created);
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      submitting = false;
    }
  }
</script>

<form onsubmit={handleSubmit} class="form">
  {#if error}
    <p class="form__error" role="alert">{error}</p>
  {/if}

  <div class="form__field">
    <label for="title">Title *</label>
    <input id="title" type="text" bind:value={title} required />
  </div>

  <div class="form__field">
    <label for="date-time">Date &amp; Time *</label>
    <input id="date-time" type="datetime-local" bind:value={dateTime} required />
  </div>

  <div class="form__field">
    <label for="speaker">Speaker</label>
    <input id="speaker" type="text" bind:value={speaker} />
  </div>

  <div class="form__field">
    <label for="description">Description</label>
    <textarea id="description" bind:value={description} rows={3}></textarea>
  </div>

  <button type="submit" disabled={submitting}>
    {submitting ? 'Creating…' : 'Create Event'}
  </button>
</form>

<style>
  .form { display: flex; flex-direction: column; gap: 1rem; max-width: 500px; }

  .form__field { display: flex; flex-direction: column; gap: 0.25rem; }

  .form__field label { font-size: 0.875rem; font-weight: 500; }

  .form__field input,
  .form__field textarea {
    padding: 0.5rem 0.75rem;
    border: 1px solid #d1d5db;
    border-radius: 0.375rem;
    font-size: 1rem;
  }

  .form__error {
    padding: 0.75rem;
    background: #fee2e2;
    color: #991b1b;
    border-radius: 0.375rem;
    font-size: 0.875rem;
  }

  button {
    padding: 0.625rem 1.25rem;
    background: #2563eb;
    color: white;
    border: none;
    border-radius: 0.375rem;
    font-size: 1rem;
    cursor: pointer;
  }

  button:disabled { opacity: 0.6; cursor: not-allowed; }
</style>
