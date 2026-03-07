<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';

  let installed = $state(false);
  let downloading = $state(false);
  let progress = $state({ downloaded: 0, total: 0 });
  let errorMessage = $state('');

  let unlisten: (() => void) | null = null;

  onMount(async () => {
    installed = await invoke<boolean>('get_mediamtx_status');
    unlisten = await listen<{ downloaded: number; total: number }>('mediamtx://progress', (e) => {
      progress = e.payload;
    });
  });

  onDestroy(() => {
    unlisten?.();
  });

  async function download() {
    downloading = true;
    errorMessage = '';
    progress = { downloaded: 0, total: 0 };
    try {
      await invoke('download_mediamtx');
      installed = true;
    } catch (e) {
      errorMessage = String(e);
    } finally {
      downloading = false;
    }
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const units = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`;
  }

  const percent = $derived(
    progress.total > 0 ? Math.round((progress.downloaded / progress.total) * 100) : null
  );
</script>

<div class="dep-row">
  <div class="dep-info">
    <span class="dep-name">mediamtx</span>
    <span class="dep-desc">Required for stream preview and multi-stream relay</span>
  </div>

  <div class="dep-status">
    {#if installed}
      <span class="badge badge--ok">Installed</span>
    {:else if downloading}
      <div class="progress-wrap" role="progressbar" aria-valuenow={percent ?? 0} aria-valuemin={0} aria-valuemax={100}>
        <div class="progress-bar">
          <div class="progress-fill" style:width="{percent ?? 0}%"></div>
        </div>
        <span class="progress-label">
          {#if percent !== null}
            {percent}% · {formatBytes(progress.downloaded)} / {formatBytes(progress.total)}
          {:else}
            {formatBytes(progress.downloaded)} downloaded…
          {/if}
        </span>
      </div>
    {:else}
      <span class="badge badge--missing">Not installed</span>
      <button class="btn-download" onclick={download}>Download</button>
    {/if}
  </div>
</div>

{#if errorMessage}
  <p class="dep-error" role="alert">{errorMessage}</p>
{/if}

<style>
  .dep-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    flex-wrap: wrap;
  }

  .dep-info {
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
  }

  .dep-name {
    font-weight: 600;
    font-size: 0.9375rem;
    font-family: ui-monospace, monospace;
  }

  .dep-desc {
    font-size: 0.8125rem;
    color: #6b7280;
  }

  .dep-status {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-shrink: 0;
  }

  .badge {
    font-size: 0.75rem;
    font-weight: 600;
    padding: 0.2rem 0.5rem;
    border-radius: 0.375rem;
  }

  .badge--ok {
    background: #d1fae5;
    color: #065f46;
  }

  .badge--missing {
    background: #fee2e2;
    color: #991b1b;
  }

  .btn-download {
    padding: 0.375rem 0.75rem;
    background: #1d4ed8;
    color: #fff;
    border: none;
    border-radius: 0.375rem;
    font-size: 0.8125rem;
    cursor: pointer;
  }

  .btn-download:hover {
    background: #1e40af;
  }

  .progress-wrap {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    min-width: 200px;
  }

  .progress-bar {
    height: 6px;
    background: #e5e7eb;
    border-radius: 3px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: #1d4ed8;
    border-radius: 3px;
    transition: width 0.2s ease;
  }

  .progress-label {
    font-size: 0.75rem;
    color: #6b7280;
  }

  .dep-error {
    font-size: 0.8125rem;
    color: #dc2626;
    margin: 0.5rem 0 0;
  }
</style>
