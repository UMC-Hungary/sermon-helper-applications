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
	import {
		listCronJobs,
		createCronJob,
		updateCronJob,
		deleteCronJob
	} from '$lib/api/cron-jobs.js';
	import type { CronJob } from '$lib/api/cron-jobs.js';

	let currentMode: AppMode | null = $state(null);
	let resetting = $state(false);
	let errorMessage = $state('');

	// Cron Jobs state
	let cronJobs: CronJob[] = $state([]);
	let cronForm = $state({ name: '', cronExpression: '', enabled: true, pullYoutube: false });
	let editingCronId: string | null = $state(null);
	let cronSaving = $state(false);
	let cronError = $state('');

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
			pullYoutube: job.pullYoutube
		};
	}

	function cancelEditCron() {
		editingCronId = null;
		cronForm = { name: '', cronExpression: '', enabled: true, pullYoutube: false };
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
          {lang.flag} {lang.name}
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

  <!-- ── Connectors ──────────────────────────────────────────────── -->
  <h2 class="section-heading">{$_('appSettings.connectors.title')}</h2>

  <ConnectorSettingsBlock connectorId="obs" />
  <ConnectorSettingsBlock connectorId="youtube" />
  <ConnectorSettingsBlock connectorId="facebook" />
  <ConnectorSettingsBlock connectorId="vmix" />
  <ConnectorSettingsBlock connectorId="atem" />
  <ConnectorSettingsBlock connectorId="discord" />

  <!-- Cron Jobs -->
  <h2 class="section-heading">Cron Jobs</h2>
  <section>
    {#if cronJobs.length > 0}
      <table class="cron-table">
        <thead>
          <tr>
            <th scope="col">Name</th>
            <th scope="col">Expression</th>
            <th scope="col">Enabled</th>
            <th scope="col">Pull YouTube</th>
            <th scope="col"><span class="sr-only">Actions</span></th>
          </tr>
        </thead>
        <tbody>
          {#each cronJobs as job (job.id)}
            <tr>
              <td>{job.name}</td>
              <td><code>{job.cronExpression}</code></td>
              <td>{job.enabled ? '✓' : '✗'}</td>
              <td>{job.pullYoutube ? '✓' : '✗'}</td>
              <td class="cron-actions">
                <button class="btn-secondary btn-sm" onclick={() => startEditCron(job)}>Edit</button>
                <button class="btn-danger btn-sm" onclick={() => removeCronJob(job.id)}>Delete</button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {:else}
      <p class="note">No cron jobs configured yet.</p>
    {/if}

    <h3 class="cron-form-title">{editingCronId ? 'Edit' : 'Add'} Cron Job</h3>

    <div class="form-grid">
      <div class="field field--full">
        <label for="cron-name">Name</label>
        <input id="cron-name" type="text" bind:value={cronForm.name} placeholder="e.g. YouTube poll" />
      </div>
      <div class="field field--full">
        <label for="cron-expr">Cron Expression</label>
        <input
          id="cron-expr"
          type="text"
          bind:value={cronForm.cronExpression}
          placeholder="e.g. 0 */5 * * * * (every 5 min)"
        />
      </div>
    </div>

    <div class="form-row">
      <label class="checkbox-label">
        <input type="checkbox" bind:checked={cronForm.enabled} />
        Enabled
      </label>
    </div>
    <div class="form-row">
      <label class="checkbox-label">
        <input type="checkbox" bind:checked={cronForm.pullYoutube} />
        Pull YouTube Live Events
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
        {cronSaving ? 'Saving…' : 'Save'}
      </button>
      {#if editingCronId}
        <button class="btn-secondary" onclick={cancelEditCron}>Cancel</button>
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
    border: 1px solid #e5e7eb;
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
    color: #374151;
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
    border: 1px solid #d1d5db;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    width: 100%;
    box-sizing: border-box;
  }

  input[type='text']:focus {
    outline: 2px solid #2563eb;
    outline-offset: 1px;
    border-color: #2563eb;
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
    color: #374151;
    border: 1px solid #d1d5db;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    cursor: pointer;
  }

  .lang-btn:hover {
    background: #f3f4f6;
  }

  .lang-btn.active {
    background: #1d4ed8;
    color: #fff;
    border-color: #1d4ed8;
  }

  .note {
    font-size: 0.875rem;
    color: #6b7280;
    margin: 0.5rem 0 1rem;
  }

  .error {
    color: #dc2626;
    font-size: 0.875rem;
    margin: 0 0 0.75rem;
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

  .btn-secondary {
    padding: 0.5rem 1rem;
    background: transparent;
    color: #1d4ed8;
    border: 1px solid #1d4ed8;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    cursor: pointer;
  }

  .btn-secondary:hover:not(:disabled) {
    background: #eff6ff;
  }

  .btn-danger {
    padding: 0.5rem 1rem;
    background: #dc2626;
    color: #fff;
    border: none;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    cursor: pointer;
  }

  .btn-danger:hover:not(:disabled) {
    background: #b91c1c;
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
    border-bottom: 1px solid #e5e7eb;
  }

  .cron-table th {
    font-weight: 600;
    color: #374151;
  }

  .cron-table code {
    font-family: ui-monospace, monospace;
    font-size: 0.8125rem;
    background: #f3f4f6;
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
