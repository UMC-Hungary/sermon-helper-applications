<script lang="ts">
  import { appMode } from '$lib/stores/mode.js';
  import { serverUrl, serverPort, authToken } from '$lib/stores/server-url.js';
  import { wsStatus } from '$lib/stores/ws.js';
</script>

<svelte:head>
  <title>Sermon Helper</title>
</svelte:head>

<h1>Sermon Helper v2</h1>

<section class="status-grid">
  <div class="card">
    <h2>Mode</h2>
    <p class="badge">{$appMode}</p>
  </div>

  <div class="card">
    <h2>WebSocket</h2>
    <p class="badge badge--{$wsStatus}">{$wsStatus}</p>
  </div>

  {#if $appMode === 'server'}
    <div class="card">
      <h2>Server URL</h2>
      <p><code>{$serverUrl}</code> (port {$serverPort})</p>
    </div>
  {/if}
</section>

<section class="quick-links">
  <h2>Quick Links</h2>
  <ul>
    <li><a href="/events">View all events</a></li>
    <li><a href="/events/new">Create new event</a></li>
    <li><a href="/connect">Connection info &amp; token</a></li>
  </ul>
</section>

{#if $appMode === 'server' && $authToken}
  <section class="token-preview">
    <h2>Auth Token</h2>
    <p>Use this token in the <code>Authorization: Bearer &lt;token&gt;</code> header.</p>
    <a href="/connect">Manage token &rarr;</a>
  </section>
{/if}

<style>
  h1 { margin-bottom: 1.5rem; }

  .status-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 1rem;
    margin-bottom: 2rem;
  }

  .card {
    padding: 1rem;
    border: 1px solid #e5e7eb;
    border-radius: 0.5rem;
  }

  .card h2 { margin: 0 0 0.5rem; font-size: 0.875rem; color: #6b7280; }

  .badge {
    display: inline-block;
    padding: 0.25rem 0.75rem;
    border-radius: 9999px;
    font-size: 0.875rem;
    background: #f3f4f6;
  }

  .badge--connected { background: #d1fae5; color: #065f46; }
  .badge--connecting { background: #fef3c7; color: #92400e; }
  .badge--disconnected, .badge--error { background: #fee2e2; color: #991b1b; }

  .quick-links { margin-bottom: 2rem; }
  .quick-links ul { padding-left: 1.25rem; }
  .quick-links li { margin-bottom: 0.5rem; }

  .token-preview {
    padding: 1rem;
    background: #eff6ff;
    border-radius: 0.5rem;
  }
</style>
