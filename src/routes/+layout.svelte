<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { appMode } from '$lib/stores/mode.js';
  import { serverUrl, serverPort, authToken } from '$lib/stores/server-url.js';
  import { connectWs, disconnectWs } from '$lib/ws/client.js';
  import type { AppMode } from '$lib/stores/mode.js';

  let { children } = $props();

  onMount(async () => {
    try {
      const mode = await invoke<string>('get_app_mode');
      appMode.set(mode as AppMode);

      if (mode === 'server') {
        const token = await invoke<string>('get_token');
        const port = await invoke<number>('get_server_port');
        authToken.set(token);
        serverPort.set(port);
        serverUrl.set(`http://localhost:${port}`);
      }

      connectWs();
    } catch (e) {
      console.error('Layout init error:', e);
    }
  });

  onDestroy(() => {
    disconnectWs();
  });
</script>

<div class="app">
  <nav>
    <a href="/">Dashboard</a>
    <a href="/events">Events</a>
    <a href="/connect">Connect</a>
  </nav>
  <main>
    {@render children()}
  </main>
</div>

<style>
  .app {
    font-family: system-ui, sans-serif;
    max-width: 1200px;
    margin: 0 auto;
    padding: 1rem;
  }

  nav {
    display: flex;
    gap: 1rem;
    padding: 0.75rem 0;
    border-bottom: 1px solid #e5e7eb;
    margin-bottom: 1.5rem;
  }

  nav a {
    color: #374151;
    text-decoration: none;
    font-weight: 500;
  }

  nav a:hover {
    color: #1d4ed8;
  }
</style>
