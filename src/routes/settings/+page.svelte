<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { invoke } from '@tauri-apps/api/core';
  import { appMode } from '$lib/stores/mode.js';
  import type { AppMode } from '$lib/stores/mode.js';
  import { locale, _ } from 'svelte-i18n';
  import { setLocale, availableLocales } from '$lib/i18n';
  import { authToken } from '$lib/stores/server-url.js';
  import ConnectorSettingsBlock from '$lib/components/connectors/ConnectorSettingsBlock.svelte';
  import MediamtxDownloadManager from '$lib/components/MediamtxDownloadManager.svelte';
  import { listCronJobs, createCronJob, updateCronJob, deleteCronJob } from '$lib/api/cron-jobs.js';
  import type { CronJob } from '$lib/api/cron-jobs.js';
  import { installBadge, getObsScenes, createBadgeSources } from '$lib/api/badge.js';
  import { obsBadgeConfig } from '$lib/stores/connectors.js';
  import { obsState } from '$lib/stores/connectors.js';

  let currentMode: AppMode | null = $state(null);
  let resetting = $state(false);
  let errorMessage = $state('');

  // Cron Jobs state
  let cronJobs: CronJob[] = $state([]);
  let cronForm = $state({ name: '', cronExpression: '', enabled: true, pullYoutube: false, autoUpload: false });
  let editingCronId: string | null = $state(null);
  let cronSaving = $state(false);
  let cronError = $state('');

  // OBS Badge state
  let badgeInstalling = $state(false);
  let badgeSaving = $state(false);
  let badgeError = $state('');
  let badgeSuccess = $state(false);
  let badgeScenes: { name: string }[] = $state([]);
  let badgeLoadingScenes = $state(false);
  let badgeInstallResult = $state<{
    shaderfilter_installed: boolean;
    shader_installed: boolean;
  } | null>(null);

  async function loadBadgeScenes() {
    if ($obsState.connection === 'connected') {
      badgeLoadingScenes = true;
      try {
        badgeScenes = await getObsScenes();
      } catch (e) {
        console.error('Failed to load scenes:', e);
      } finally {
        badgeLoadingScenes = false;
      }
    }
  }

  $effect(() => {
    if ($obsState.connection === 'connected') {
      loadBadgeScenes();
    }
  });

  onMount(async () => {
    try {
      const mode = await invoke<string | null>('get_app_mode');
      currentMode = (mode as AppMode) ?? null;
    } catch (e) {
      console.error('Settings load error:', e);
    }
  });

  // Load cron jobs once the auth token is available (set async by layout onMount).
  $effect(() => {
    if ($authToken) {
      loadCronJobs();
    }
  });

  async function loadCronJobs() {
    try {
      cronJobs = await listCronJobs();
    } catch {
      // server might not be running in client mode — silently ignore
    }
  }

  function startEditCron(job: CronJob) {
    editingCronId = job.id;
    cronForm = {
      name: job.name,
      cronExpression: job.cronExpression,
      enabled: job.enabled,
      pullYoutube: job.pullYoutube,
      autoUpload: job.autoUpload,
    };
  }

  function cancelEditCron() {
    editingCronId = null;
    cronForm = { name: '', cronExpression: '', enabled: true, pullYoutube: false, autoUpload: false };
    cronError = '';
  }

  async function saveCronJob() {
    cronSaving = true;
    cronError = '';
    try {
      if (editingCronId) {
        await updateCronJob(editingCronId, cronForm);
      } else {
        await createCronJob(cronForm);
      }
      cancelEditCron();
      await loadCronJobs();
    } catch (e) {
      cronError = String(e);
    } finally {
      cronSaving = false;
    }
  }

  async function removeCronJob(id: string) {
    cronError = '';
    try {
      await deleteCronJob(id);
      await loadCronJobs();
    } catch (e) {
      cronError = String(e);
    }
  }

  async function changeMode() {
    resetting = true;
    errorMessage = '';
    try {
      await invoke('reset_setup');
      appMode.set('server');
      await goto('/setup');
    } catch (e) {
      errorMessage = String(e);
      resetting = false;
    }
  }
</script>

<div class="settings-container">
  <h1>{$_('appSettings.title')}</h1>

  <section>
    <h2>{$_('appSettings.language')}</h2>
    <div class="lang-buttons">
      {#each availableLocales as lang}
        <button
          class="lang-btn"
          class:active={$locale === lang.code}
          onclick={() => setLocale(lang.code)}
        >
          {lang.flag}
          {lang.name}
        </button>
      {/each}
    </div>
  </section>

  <section>
    <h2>{$_('appSettings.appMode.title')}</h2>
    <p>
      {$_('appSettings.appMode.current')}: <strong>{currentMode ?? '—'}</strong>
    </p>
    <p class="note">
      {$_('appSettings.appMode.note')}
    </p>

    {#if errorMessage}
      <p class="error" role="alert">{errorMessage}</p>
    {/if}

    <button onclick={changeMode} disabled={resetting}>
      {resetting ? $_('appSettings.appMode.changing') : $_('appSettings.appMode.changeMode')}
    </button>
  </section>

  <!-- ── Dependencies ────────────────────────────────────────────── -->
  <h2 class="section-heading">{$_('appSettings.dependencies.title')}</h2>
  <section>
    <p class="note">{$_('appSettings.dependencies.note')}</p>
    <MediamtxDownloadManager />
  </section>

  <!-- ── Connectors ──────────────────────────────────────────────── -->
  <h2 class="section-heading">{$_('appSettings.connectors.title')}</h2>

  <ConnectorSettingsBlock connectorId="obs" />
  <ConnectorSettingsBlock connectorId="youtube" />
  <ConnectorSettingsBlock connectorId="facebook" />
  <ConnectorSettingsBlock connectorId="vmix" />
  <ConnectorSettingsBlock connectorId="atem" />
  <ConnectorSettingsBlock connectorId="broadlink" />
  <ConnectorSettingsBlock connectorId="discord" />

  <!-- OBS Badge -->
  <h2 class="section-heading">{$_('appSettings.obsBadge.title')}</h2>
  <section>
    <p class="note">{$_('appSettings.obsBadge.description')}</p>

    {#if $obsState.connection !== 'connected'}
      <p class="error" role="alert">{$_('appSettings.obsBadge.obsNotConnected')}</p>
    {:else}
      <!-- Step 1: Install plugin & shader -->
      <button
        class="btn-primary"
        onclick={async () => {
          badgeInstalling = true;
          badgeError = '';
          try {
            badgeInstallResult = await installBadge();
          } catch (e) {
            badgeError = String(e);
          } finally {
            badgeInstalling = false;
          }
        }}
        disabled={badgeInstalling}
      >
        {badgeInstalling ? $_('appSettings.obsBadge.installing') : $_('appSettings.obsBadge.installButton')}
      </button>

      {#if badgeInstallResult}
        <p class="note" style="margin-top: 0.5rem;">
          {badgeInstallResult.shaderfilter_installed ? $_('appSettings.obsBadge.pluginInstalled') : $_('appSettings.obsBadge.pluginNotInstalled')}<br />
          {badgeInstallResult.shader_installed ? $_('appSettings.obsBadge.shaderInstalled') : $_('appSettings.obsBadge.shaderNotInstalled')}
        </p>
      {/if}

      <hr style="margin: 1rem 0;" />

      <!-- Step 2: Create OBS sources -->
      <label class="field">
        <span>{$_('appSettings.obsBadge.targetScene')}</span>
        <select bind:value={$obsBadgeConfig.sceneName}>
          <option value="">{$_('appSettings.obsBadge.selectScene')}</option>
          {#each badgeScenes as scene}
            <option value={scene.name}>{scene.name}</option>
          {/each}
        </select>
        {#if badgeLoadingScenes}
          <span class="note">{$_('appSettings.obsBadge.loadingScenes')}</span>
        {/if}
      </label>

      <div class="button-row" style="margin-top: 0.5rem;">
        <button
          class="btn-primary"
          onclick={async () => {
            if (!$obsBadgeConfig.sceneName) {
              badgeError = $_('appSettings.obsBadge.selectSceneError');
              return;
            }
            badgeSaving = true;
            badgeError = '';
            badgeSuccess = false;
            try {
              await createBadgeSources($obsBadgeConfig.sceneName);
              $obsBadgeConfig.enabled = true;
              badgeSuccess = true;
            } catch (e) {
              badgeError = String(e);
            } finally {
              badgeSaving = false;
            }
          }}
          disabled={badgeSaving || !$obsBadgeConfig.sceneName}
        >
          {badgeSaving ? $_('appSettings.obsBadge.creating') : $_('appSettings.obsBadge.createButton')}
        </button>
      </div>
    {/if}

    {#if badgeSuccess}
      <p class="note" role="status" style="margin-top: 0.5rem; color: var(--status-ok-text, green);">
        {$_('appSettings.obsBadge.sourcesCreated', { values: { sceneName: $obsBadgeConfig.sceneName } })}
      </p>
    {/if}

    {#if badgeError}
      <p class="error" role="alert">{badgeError}</p>
    {/if}
  </section>

  <!-- Cron Jobs -->
  <h2 class="section-heading">{$_('appSettings.cronJobs.title')}</h2>
  <section>
    {#if cronJobs.length > 0}
      <table class="cron-table">
        <thead>
          <tr>
            <th scope="col">{$_('appSettings.cronJobs.colName')}</th>
            <th scope="col">{$_('appSettings.cronJobs.colExpression')}</th>
            <th scope="col">{$_('appSettings.cronJobs.colEnabled')}</th>
            <th scope="col">{$_('appSettings.cronJobs.colPullYoutube')}</th>
            <th scope="col">Auto Upload</th>
            <th scope="col"><span class="sr-only">{$_('appSettings.cronJobs.colActions')}</span></th>
          </tr>
        </thead>
        <tbody>
          {#each cronJobs as job (job.id)}
            <tr>
              <td>{job.name}</td>
              <td><code>{job.cronExpression}</code></td>
              <td>{job.enabled ? '✓' : '✗'}</td>
              <td>{job.pullYoutube ? '✓' : '✗'}</td>
              <td>{job.autoUpload ? '✓' : '✗'}</td>
              <td class="cron-actions">
                <button class="btn-secondary btn-sm" onclick={() => startEditCron(job)}>{$_('appSettings.cronJobs.editButton')}</button
                >
                <button class="btn-danger btn-sm" onclick={() => removeCronJob(job.id)}
                  >{$_('appSettings.cronJobs.deleteButton')}</button
                >
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {:else}
      <p class="note">{$_('appSettings.cronJobs.noneConfigured')}</p>
    {/if}

    <h3 class="cron-form-title">{editingCronId ? $_('appSettings.cronJobs.editTitle') : $_('appSettings.cronJobs.addTitle')}</h3>

    <div class="form-grid">
      <div class="field field--full">
        <label for="cron-name">{$_('appSettings.cronJobs.colName')}</label>
        <input
          id="cron-name"
          type="text"
          bind:value={cronForm.name}
          placeholder={$_('appSettings.cronJobs.namePlaceholder')}
        />
      </div>
      <div class="field field--full">
        <label for="cron-expr">{$_('appSettings.cronJobs.expressionLabel')}</label>
        <input
          id="cron-expr"
          type="text"
          bind:value={cronForm.cronExpression}
          placeholder={$_('appSettings.cronJobs.expressionPlaceholder')}
        />
      </div>
    </div>

    <div class="form-row">
      <label class="checkbox-label">
        <input type="checkbox" bind:checked={cronForm.enabled} />
        {$_('appSettings.cronJobs.enabledLabel')}
      </label>
    </div>
    <div class="form-row">
      <label class="checkbox-label">
        <input type="checkbox" bind:checked={cronForm.pullYoutube} />
        {$_('appSettings.cronJobs.pullYoutubeLabel')}
      </label>
    </div>
    <div class="form-row">
      <label class="checkbox-label">
        <input type="checkbox" bind:checked={cronForm.autoUpload} />
        Auto-upload flagged recordings
      </label>
    </div>

    {#if cronError}
      <p class="error" role="alert">{cronError}</p>
    {/if}

    <div class="button-row">
      <button
        class="btn-primary"
        onclick={saveCronJob}
        disabled={cronSaving || !cronForm.name || !cronForm.cronExpression}
      >
        {cronSaving ? $_('appSettings.cronJobs.saving') : $_('appSettings.cronJobs.save')}
      </button>
      {#if editingCronId}
        <button class="btn-secondary" onclick={cancelEditCron}>{$_('appSettings.cronJobs.cancel')}</button>
      {/if}
    </div>
  </section>
</div>

<style>
  .settings-container {
    max-width: 600px;
  }

  h1 {
    margin: 0 0 1.5rem;
    font-size: 1.5rem;
  }

  h2 {
    font-size: 1.125rem;
    margin: 0 0 0.75rem;
  }

  .section-heading {
    margin-top: 1.5rem;
  }

  section {
    padding: 1.25rem;
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    margin-bottom: 1rem;
  }

  .form-row {
    margin-bottom: 0.75rem;
  }

  .form-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.75rem;
    margin-bottom: 0.75rem;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .field--full {
    grid-column: 1 / -1;
  }

  .field label,
  .checkbox-label {
    font-size: 0.875rem;
    color: var(--text-primary);
    font-weight: 500;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
  }

  input[type='text'] {
    padding: 0.375rem 0.625rem;
    border: 1px solid var(--input-border);
    border-radius: 0.375rem;
    font-size: 0.875rem;
    width: 100%;
    box-sizing: border-box;
  }

  input[type='text']:focus {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
    border-color: var(--accent);
  }

  .button-row {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .lang-buttons {
    display: flex;
    gap: 0.5rem;
  }

  .lang-btn {
    padding: 0.5rem 1rem;
    background: transparent;
    color: var(--text-primary);
    border: 1px solid var(--border);
    border-radius: 0.375rem;
    font-size: 0.875rem;
    cursor: pointer;
  }

  .lang-btn:hover {
    background: var(--nav-item-hover);
  }

  .lang-btn.active {
    background: var(--accent);
    color: #fff;
    border-color: var(--accent);
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

  .btn-secondary {
    padding: 0.5rem 1rem;
    background: transparent;
    color: var(--accent);
    border: 1px solid var(--accent);
    border-radius: 0.375rem;
    font-size: 0.875rem;
    cursor: pointer;
  }

  .btn-secondary:hover:not(:disabled) {
    background: var(--accent-subtle);
  }

  .btn-danger {
    padding: 0.5rem 1rem;
    background: var(--status-err-dot);
    color: #fff;
    border: none;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    cursor: pointer;
  }

  .btn-danger:hover:not(:disabled) {
    filter: brightness(0.9);
  }

  button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .cron-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.875rem;
    margin-bottom: 1.25rem;
  }

  .cron-table th,
  .cron-table td {
    text-align: left;
    padding: 0.375rem 0.5rem;
    border-bottom: 1px solid var(--border);
  }

  .cron-table th {
    font-weight: 600;
    color: var(--text-primary);
  }

  .cron-table code {
    font-family: ui-monospace, monospace;
    font-size: 0.8125rem;
    background: var(--content-bg);
    padding: 0.125rem 0.25rem;
    border-radius: 0.25rem;
  }

  .cron-actions {
    display: flex;
    gap: 0.375rem;
  }

  .btn-sm {
    padding: 0.25rem 0.625rem;
    font-size: 0.8125rem;
  }

  .cron-form-title {
    font-size: 0.9375rem;
    margin: 0 0 0.75rem;
  }

  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }
</style>
