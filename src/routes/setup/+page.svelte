<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  let selectedMode: 'server' | 'client' = 'server';
  let clientUrl = '';
  let clientToken = '';
  let errorMessage = '';
  let submitting = false;

  onMount(async () => {
    try {
      const mode = await invoke<string | null>('get_app_mode');
      if (mode !== null) {
        // Mode already configured — full reload so the layout initialises properly.
        window.location.href = '/';
      }
    } catch (e) {
      console.error('Setup mount error:', e);
    }
  });

  async function handleSubmit() {
    errorMessage = '';
    submitting = true;

    try {
      if (selectedMode === 'client') {
        const trimmedUrl = clientUrl.trim().replace(/\/$/, '');
        const trimmedToken = clientToken.trim();

        if (!trimmedUrl) {
          errorMessage = 'Server URL is required.';
          return;
        }
        if (!trimmedToken) {
          errorMessage = 'Auth token is required.';
          return;
        }

        // Ping the remote server before saving
        let response: Response;
        try {
          response = await fetch(`${trimmedUrl}/api/events`, {
            headers: { Authorization: `Bearer ${trimmedToken}` },
          });
        } catch {
          errorMessage = 'Server unreachable — check the URL and try again.';
          return;
        }

        if (response.status === 401) {
          errorMessage = 'Wrong auth token — check the token and try again.';
          return;
        }

        if (!response.ok) {
          errorMessage = `Unexpected response from server (${response.status}).`;
          return;
        }

        await invoke('complete_setup', {
          mode: 'client',
          serverUrl: trimmedUrl,
          clientToken: trimmedToken,
        });
      } else {
        await invoke('complete_setup', {
          mode: 'server',
          serverUrl: null,
          clientToken: null,
        });
      }

      // Full reload so the layout re-runs onMount with the persisted mode,
      // initialises stores, and connects the WebSocket from scratch.
      window.location.href = '/';
    } catch (e) {
      errorMessage = String(e);
    } finally {
      submitting = false;
    }
  }
</script>

<div class="setup-container">
  <h1>Welcome — Choose a Mode</h1>
  <p class="subtitle">Select how this app will operate on this device.</p>

  <form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
    <fieldset>
      <legend>App mode</legend>

      <label class="radio-label">
        <input
          type="radio"
          name="mode"
          value="server"
          bind:group={selectedMode}
        />
        <span class="radio-text">
          <strong>Server</strong>
          <span class="description">Run the backend locally (requires this machine to stay on).</span>
        </span>
      </label>

      <label class="radio-label">
        <input
          type="radio"
          name="mode"
          value="client"
          bind:group={selectedMode}
        />
        <span class="radio-text">
          <strong>Client</strong>
          <span class="description">Connect to a remote server. Get the URL and token from the server's Connect page.</span>
        </span>
      </label>
    </fieldset>

    {#if selectedMode === 'client'}
      <div class="client-fields">
        <label for="client-url">Server URL</label>
        <input
          id="client-url"
          type="url"
          placeholder="https://example.com"
          bind:value={clientUrl}
          autocomplete="off"
        />

        <label for="client-token">Auth Token</label>
        <textarea
          id="client-token"
          rows="3"
          placeholder="Paste the auth token here"
          bind:value={clientToken}
        ></textarea>
      </div>
    {/if}

    {#if errorMessage}
      <p class="error" role="alert">{errorMessage}</p>
    {/if}

    <button type="submit" disabled={submitting}>
      {submitting ? 'Setting up…' : 'Confirm'}
    </button>
  </form>
</div>

<style>
  .setup-container {
    max-width: 480px;
    margin: 4rem auto;
    padding: 2rem;
    border: 1px solid var(--border);
    border-radius: 0.5rem;
  }

  h1 {
    margin: 0 0 0.25rem;
    font-size: 1.5rem;
  }

  .subtitle {
    margin: 0 0 1.5rem;
    color: var(--text-secondary);
  }

  fieldset {
    border: none;
    padding: 0;
    margin: 0 0 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  legend {
    font-weight: 600;
    margin-bottom: 0.5rem;
  }

  .radio-label {
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
    cursor: pointer;
  }

  .radio-label input[type='radio'] {
    margin-top: 0.2rem;
    flex-shrink: 0;
  }

  .radio-text {
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
  }

  .description {
    font-size: 0.875rem;
    color: var(--text-secondary);
  }

  .client-fields {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    margin-bottom: 1.25rem;
  }

  .client-fields label {
    font-weight: 500;
    font-size: 0.875rem;
  }

  .client-fields input,
  .client-fields textarea {
    width: 100%;
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--input-border);
    border-radius: 0.375rem;
    font-size: 0.875rem;
    font-family: inherit;
    box-sizing: border-box;
  }

  .client-fields textarea {
    resize: vertical;
    font-family: monospace;
  }

  .error {
    color: var(--status-err-text);
    font-size: 0.875rem;
    margin: 0 0 1rem;
  }

  button {
    width: 100%;
    padding: 0.625rem 1rem;
    background: var(--accent);
    color: #fff;
    border: none;
    border-radius: 0.375rem;
    font-size: 1rem;
    cursor: pointer;
  }

  button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  button:not(:disabled):hover {
    filter: brightness(0.9);
  }
</style>
