<script lang="ts">
  import type { Recording } from '$lib/types/recording.js';

  interface Props {
    recordings: Recording[];
    loading: boolean;
  }

  let { recordings, loading }: Props = $props();

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function formatDuration(seconds: number): string {
    const m = Math.floor(seconds / 60);
    const s = Math.floor(seconds % 60);
    return `${m}:${s.toString().padStart(2, '0')}`;
  }
</script>

{#if loading}
  <p>Loading recordings…</p>
{:else if recordings.length === 0}
  <p class="empty">No recordings for this event.</p>
{:else}
  <ul class="list">
    {#each recordings as rec (rec.id)}
      <li class="item">
        <div class="item__name">{rec.customTitle ?? rec.fileName}</div>
        <div class="item__meta">
          {formatSize(rec.fileSize)} &middot; {formatDuration(rec.durationSeconds)}
          {#if rec.uploaded}
            &middot; <span class="badge--uploaded">Uploaded</span>
          {/if}
          {#if rec.videoUrl}
            &middot; <a href={rec.videoUrl} target="_blank" rel="noopener noreferrer">Watch</a>
          {/if}
        </div>
      </li>
    {/each}
  </ul>
{/if}

<style>
  .list { list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 0.5rem; }

  .item {
    padding: 0.75rem;
    border: 1px solid #e5e7eb;
    border-radius: 0.375rem;
  }

  .item__name { font-weight: 500; margin-bottom: 0.25rem; }

  .item__meta { font-size: 0.875rem; color: #6b7280; }

  .badge--uploaded { color: #065f46; font-weight: 500; }

  .empty { color: #6b7280; }
</style>
