<script lang="ts">
  import { _ } from 'svelte-i18n';
  import { installBadge, getObsScenes, createBadgeSources } from '$lib/api/badge.js';
  import { obsBadgeConfig, obsState } from '$lib/stores/connectors.js';

  let installing = $state(false);
  let saving = $state(false);
  let error = $state('');
  let success = $state(false);
  let scenes: { name: string }[] = $state([]);
  let loadingScenes = $state(false);
  let installResult = $state<{
    shaderfilter_installed: boolean;
    shader_installed: boolean;
  } | null>(null);

  $effect(() => {
    if ($obsState.connection === 'connected') {
      loadScenes();
    }
  });

  async function loadScenes() {
    loadingScenes = true;
    try {
      scenes = await getObsScenes();
    } catch (e) {
      console.error('Failed to load scenes:', e);
    } finally {
      loadingScenes = false;
    }
  }

  async function install() {
    installing = true;
    error = '';
    try {
      installResult = await installBadge();
    } catch (e) {
      error = String(e);
    } finally {
      installing = false;
    }
  }

  async function createSources() {
    if (!$obsBadgeConfig.sceneName) {
      error = $_('appSettings.obsBadge.selectSceneError');
      return;
    }
    saving = true;
    error = '';
    success = false;
    try {
      await createBadgeSources($obsBadgeConfig.sceneName);
      $obsBadgeConfig.enabled = true;
      success = true;
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }
</script>

<section>
  <h2>{$_('appSettings.obsBadge.title')}</h2>
  <p class="note">{$_('appSettings.obsBadge.description')}</p>

  {#if $obsState.connection !== 'connected'}
    <p class="error" role="alert">{$_('appSettings.obsBadge.obsNotConnected')}</p>
  {:else}
    <button class="btn-primary" onclick={install} disabled={installing}>
      {installing ? $_('appSettings.obsBadge.installing') : $_('appSettings.obsBadge.installButton')}
    </button>

    {#if installResult}
      <p class="note" style="margin-top: 0.5rem;">
        {installResult.shaderfilter_installed
          ? $_('appSettings.obsBadge.pluginInstalled')
          : $_('appSettings.obsBadge.pluginNotInstalled')}<br />
        {installResult.shader_installed
          ? $_('appSettings.obsBadge.shaderInstalled')
          : $_('appSettings.obsBadge.shaderNotInstalled')}
      </p>
    {/if}

    <hr style="margin: 1rem 0;" />

    <label class="field">
      <span>{$_('appSettings.obsBadge.targetScene')}</span>
      <select bind:value={$obsBadgeConfig.sceneName}>
        <option value="">{$_('appSettings.obsBadge.selectScene')}</option>
        {#each scenes as scene}
          <option value={scene.name}>{scene.name}</option>
        {/each}
      </select>
      {#if loadingScenes}
        <span class="note">{$_('appSettings.obsBadge.loadingScenes')}</span>
      {/if}
    </label>

    <div class="button-row" style="margin-top: 0.5rem;">
      <button
        class="btn-primary"
        onclick={createSources}
        disabled={saving || !$obsBadgeConfig.sceneName}
      >
        {saving ? $_('appSettings.obsBadge.creating') : $_('appSettings.obsBadge.createButton')}
      </button>
    </div>
  {/if}

  {#if success}
    <p class="note" role="status" style="margin-top: 0.5rem; color: var(--status-ok-text, green);">
      {$_('appSettings.obsBadge.sourcesCreated', {
        values: { sceneName: $obsBadgeConfig.sceneName },
      })}
    </p>
  {/if}

  {#if error}
    <p class="error" role="alert">{error}</p>
  {/if}
</section>

<style>
  section {
    padding: 1.25rem;
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    margin-bottom: 1rem;
  }

  h2 {
    font-size: 1.125rem;
    margin: 0 0 0.75rem;
  }

  .note {
    font-size: 0.875rem;
    color: var(--text-secondary);
    margin: 0.5rem 0 1rem;
  }

  .error {
    color: var(--status-err-text);
    font-size: 0.875rem;
    margin: 0 0 0.75rem;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    font-size: 0.875rem;
    color: var(--text-primary);
    font-weight: 500;
  }

  .button-row {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .btn-primary {
    padding: 0.5rem 1rem;
    background: var(--accent);
    color: #fff;
    border: none;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    cursor: pointer;
  }

  .btn-primary:hover:not(:disabled) {
    filter: brightness(0.9);
  }

  .btn-primary:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
</style>
