<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { openUrl } from '@tauri-apps/plugin-opener';

  interface Props {
    connector: 'youtube' | 'facebook';
    onclose: () => void;
  }

  let { connector, onclose }: Props = $props();

  const names: Record<'youtube' | 'facebook', string> = {
    youtube: 'YouTube',
    facebook: 'Facebook',
  };

  let opening = $state(false);
  let error = $state('');

  async function reLogin() {
    opening = true;
    error = '';
    try {
      const command = connector === 'youtube' ? 'get_youtube_auth_url' : 'get_facebook_auth_url';
      const url = await invoke<string>(command);
      await openUrl(url);
      onclose();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      opening = false;
    }
  }
</script>

<!-- Overlay -->
<div
  class="overlay"
  role="presentation"
  onclick={onclose}
  onkeydown={(e) => { if (e.key === 'Escape') onclose(); }}
></div>

<!-- Dialog -->
<div class="modal" role="alertdialog" aria-modal="true" aria-labelledby="relogin-title" aria-describedby="relogin-desc">
  <h2 id="relogin-title" class="modal__title">Session Expired</h2>
  <p id="relogin-desc" class="modal__body">
    Your {names[connector]} session has expired. Please re-login to continue automatic
    event scheduling.
  </p>

  {#if error}
    <p class="modal__error" role="alert">{error}</p>
  {/if}

  <div class="modal__actions">
    <button class="btn-primary" onclick={reLogin} disabled={opening}>
      {opening ? 'Opening…' : `Re-login with ${names[connector]}`}
    </button>
    <button class="btn-secondary" onclick={onclose}>Dismiss</button>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.45);
    z-index: 100;
  }

  .modal {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    z-index: 101;
    background: #fff;
    border-radius: 0.75rem;
    padding: 2rem;
    max-width: 400px;
    width: calc(100% - 2rem);
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.15);
  }

  .modal__title {
    font-size: 1.125rem;
    font-weight: 600;
    margin: 0 0 0.75rem;
    color: #111827;
  }

  .modal__body {
    font-size: 0.9rem;
    color: #4b5563;
    margin: 0 0 1.25rem;
    line-height: 1.5;
  }

  .modal__error {
    color: #dc2626;
    font-size: 0.875rem;
    margin: 0 0 0.75rem;
  }

  .modal__actions {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .btn-primary {
    padding: 0.5rem 1rem;
    background: #1d4ed8;
    color: #fff;
    border: none;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    cursor: pointer;
  }

  .btn-primary:hover:not(:disabled) {
    background: #1e40af;
  }

  .btn-primary:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .btn-secondary {
    padding: 0.5rem 1rem;
    background: transparent;
    color: #374151;
    border: 1px solid #d1d5db;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    cursor: pointer;
  }

  .btn-secondary:hover {
    background: #f3f4f6;
  }
</style>
