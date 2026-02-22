<script lang="ts">
  import { appMode } from '$lib/stores/mode.js';
  import { serverUrl, serverPort } from '$lib/stores/server-url.js';
  import TokenDisplay from '$lib/components/connect/TokenDisplay.svelte';
  import ConnectionGuide from '$lib/components/connect/ConnectionGuide.svelte';
</script>

<svelte:head>
  <title>Connect — Sermon Helper</title>
</svelte:head>

<h1>Connection Info</h1>

<section class="info-card">
  <h2>Server</h2>
  <dl class="meta">
    <dt>Mode</dt>
    <dd>{$appMode}</dd>
    <dt>URL</dt>
    <dd><code>{$serverUrl}</code></dd>
    <dt>Port</dt>
    <dd>{$serverPort}</dd>
  </dl>
</section>

{#if $appMode === 'server'}
  <section>
    <h2>Auth Token</h2>
    <TokenDisplay />
  </section>

  <section>
    <ConnectionGuide />
  </section>
{:else}
  <p>In client mode. Connect to a server to manage events.</p>
{/if}

<style>
  h1 { margin-bottom: 1.5rem; }

  section { margin-bottom: 2rem; }

  h2 { margin: 0 0 1rem; }

  .info-card {
    padding: 1.25rem;
    background: #f9fafb;
    border: 1px solid #e5e7eb;
    border-radius: 0.5rem;
    max-width: 600px;
  }

  .meta { display: grid; grid-template-columns: max-content 1fr; gap: 0.5rem 1rem; margin: 0; }
  .meta dt { font-weight: 600; }
  .meta dd { margin: 0; }
</style>
