<script lang="ts">
  import { appMode } from '$lib/stores/mode.js';
  import { serverUrl, serverPort, localNetworkUrl } from '$lib/stores/server-url.js';
  import TokenDisplay from '$lib/components/connect/TokenDisplay.svelte';
  import ConnectionGuide from '$lib/components/connect/ConnectionGuide.svelte';

  type Platform = 'linux-x86_64' | 'linux-arm64' | 'macos-arm64' | 'macos-x86_64';

  const platforms: { id: Platform; label: string }[] = [
    { id: 'linux-x86_64', label: 'Linux (x86_64)' },
    { id: 'linux-arm64', label: 'Linux ARM64 (RPi)' },
    { id: 'macos-arm64', label: 'macOS (Apple Silicon)' },
    { id: 'macos-x86_64', label: 'macOS (Intel)' }
  ];

  let copiedNetworkUrl = $state(false);
  let selectedPlatform = $state<Platform>('linux-arm64');
  let copiedInstall = $state(false);
  let copiedManual = $state(false);
  let autoStart = $state(false);

  const wsUrl = $derived(
    ($localNetworkUrl || $serverUrl).replace(/^http/, 'ws') + '/ws'
  );

  const installCommand = $derived(
    `curl -fsSL https://raw.githubusercontent.com/UMC-Hungary/sermon-helper-applications/main/presenter-receiver/install.sh | bash -s -- ${wsUrl}${autoStart ? ' --service' : ''}`
  );

  const binaryName: Record<Platform, string> = {
    'linux-x86_64': 'presenter-receiver-linux-x86_64',
    'linux-arm64': 'presenter-receiver-linux-arm64',
    'macos-arm64': 'presenter-receiver-macos-arm64',
    'macos-x86_64': 'presenter-receiver-macos-x86_64'
  };

  const manualCommand = $derived(
    `curl -fsSL https://github.com/UMC-Hungary/sermon-helper-applications/releases/latest/download/${binaryName[selectedPlatform]} -o presenter-receiver\nchmod +x presenter-receiver\n./presenter-receiver ${wsUrl}`
  );

  async function copyNetworkUrl() {
    await navigator.clipboard.writeText($localNetworkUrl);
    copiedNetworkUrl = true;
    setTimeout(() => { copiedNetworkUrl = false; }, 2000);
  }

  async function copyInstall() {
    await navigator.clipboard.writeText(installCommand);
    copiedInstall = true;
    setTimeout(() => { copiedInstall = false; }, 2000);
  }

  async function copyManual() {
    await navigator.clipboard.writeText(manualCommand);
    copiedManual = true;
    setTimeout(() => { copiedManual = false; }, 2000);
  }
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
    <dt>Local URL</dt>
    <dd><code>{$serverUrl}</code></dd>
    {#if $localNetworkUrl}
      <dt>Network URL</dt>
      <dd class="url-cell">
        <code>{$localNetworkUrl}</code>
        <button onclick={copyNetworkUrl}>{copiedNetworkUrl ? 'Copied!' : 'Copy'}</button>
      </dd>
    {/if}
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

  <section class="info-card">
    <h2>Presenter Receiver</h2>
    <p class="note">Install the presenter receiver on a display device. It connects to this server and renders the active slide on-screen.</p>

    <h3>One-line install</h3>
    <p class="note">Auto-detects platform (macOS arm64/x86_64, Linux x86_64/arm64). Installs and starts immediately.</p>
    <label class="autostart-toggle">
      <input type="checkbox" bind:checked={autoStart} />
      Auto-start on boot
      <span class="autostart-hint">(Linux / systemd only)</span>
    </label>
    <div class="url-cell">
      <code class="cmd">{installCommand}</code>
      <button onclick={copyInstall}>{copiedInstall ? 'Copied!' : 'Copy'}</button>
    </div>

    <h3>Manual download</h3>
    <div class="platform-tabs">
      {#each platforms as p}
        <button
          class="tab"
          class:active={selectedPlatform === p.id}
          onclick={() => (selectedPlatform = p.id)}
        >{p.label}</button>
      {/each}
    </div>
    <div class="url-cell">
      <code class="cmd multiline">{manualCommand}</code>
      <button onclick={copyManual}>{copiedManual ? 'Copied!' : 'Copy'}</button>
    </div>
  </section>
{:else}
  <p>In client mode. Connect to a server to manage events.</p>
{/if}

<style>
  h1 {
    margin-bottom: 1.5rem;
  }

  section {
    margin-bottom: 2rem;
  }

  h2 {
    margin: 0 0 1rem;
  }

  .info-card {
    padding: 1.25rem;
    background: var(--content-bg);
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    max-width: 600px;
  }

  .url-cell {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .url-cell button {
    padding: 0.2rem 0.6rem;
    border: 1px solid var(--border);
    border-radius: 0.375rem;
    background: var(--glass-card-bg);
    cursor: pointer;
    font-size: 0.8rem;
    white-space: nowrap;
  }

  .url-cell button:hover {
    background: var(--nav-item-hover);
  }

  .meta {
    display: grid;
    grid-template-columns: max-content 1fr;
    gap: 0.5rem 1rem;
    margin: 0;
  }
  .meta dt {
    font-weight: 600;
  }
  .meta dd {
    margin: 0;
  }

  h3 {
    margin: 1.25rem 0 0.25rem;
    font-size: 0.95rem;
  }

  .autostart-toggle {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.85rem;
    cursor: pointer;
    margin-bottom: 0.5rem;
    user-select: none;
  }

  .autostart-hint {
    color: var(--text-secondary, #666);
    font-size: 0.78rem;
  }

  .note {
    font-size: 0.85rem;
    color: var(--text-secondary, #666);
    margin: 0 0 0.75rem;
  }

  .cmd {
    flex: 1;
    font-size: 0.78rem;
    word-break: break-all;
    white-space: pre-wrap;
    font-family: monospace;
  }

  .cmd.multiline {
    white-space: pre;
  }

  .platform-tabs {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
    margin-bottom: 0.6rem;
  }

  .tab {
    padding: 0.25rem 0.7rem;
    border: 1px solid var(--border);
    border-radius: 0.375rem;
    background: var(--glass-card-bg);
    cursor: pointer;
    font-size: 0.78rem;
    color: var(--text-secondary, #666);
  }

  .tab:hover {
    background: var(--nav-item-hover);
  }

  .tab.active {
    background: var(--nav-item-active-bg);
    color: var(--nav-item-active-text);
    border-color: transparent;
  }
</style>
