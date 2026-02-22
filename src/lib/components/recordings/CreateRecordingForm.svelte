<script lang="ts">
  import { createRecording } from '$lib/api/recordings.js';
  import type { Recording } from '$lib/types/recording.js';

  interface Props {
    eventId: string;
    oncreated?: (recording: Recording) => void;
  }

  let { eventId, oncreated }: Props = $props();

  let filePath = $state('');
  let fileName = $state('');
  let customTitle = $state('');
  let submitting = $state(false);
  let error = $state('');

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    submitting = true;
    error = '';

    try {
      const created = await createRecording(eventId, {
        file_path: filePath,
        file_name: fileName || filePath.split('/').pop() || 'recording',
        custom_title: customTitle || undefined,
      });
      oncreated?.(created);
      filePath = '';
      fileName = '';
      customTitle = '';
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
    <label for="file-path">File Path *</label>
    <input id="file-path" type="text" bind:value={filePath} required placeholder="/path/to/recording.mp4" />
  </div>

  <div class="form__field">
    <label for="file-name">File Name</label>
    <input id="file-name" type="text" bind:value={fileName} placeholder="recording.mp4" />
  </div>

  <div class="form__field">
    <label for="custom-title">Custom Title</label>
    <input id="custom-title" type="text" bind:value={customTitle} />
  </div>

  <button type="submit" disabled={submitting}>
    {submitting ? 'Adding…' : 'Add Recording'}
  </button>
</form>

<style>
  .form { display: flex; flex-direction: column; gap: 1rem; max-width: 500px; }
  .form__field { display: flex; flex-direction: column; gap: 0.25rem; }
  .form__field label { font-size: 0.875rem; font-weight: 500; }
  .form__field input {
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
