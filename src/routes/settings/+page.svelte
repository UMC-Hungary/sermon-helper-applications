<script lang="ts">
  import { _ } from 'svelte-i18n';
  import ConnectorSettingsBlock from '$lib/components/connectors/ConnectorSettingsBlock.svelte';
  import LanguageSettings from '$lib/components/settings/LanguageSettings.svelte';
  import AppModeSettings from '$lib/components/settings/AppModeSettings.svelte';
  import ObsBadgeSettings from '$lib/components/settings/ObsBadgeSettings.svelte';
  import CronJobsSettings from '$lib/components/settings/CronJobsSettings.svelte';
  import AppVersionSettings from '$lib/components/settings/AppVersionSettings.svelte';
  import { useWebPresenter } from '$lib/stores/presenter.js';
  import { sendWsCommand } from '$lib/ws/client.js';

  function handleWebPresenterToggle(e: Event) {
    const enabled = (e.target as HTMLInputElement).checked;
    sendWsCommand('presentation.set_use_web_presenter', { enabled });
  }
</script>

<div class="settings-container">
  <h1>{$_('appSettings.title')}</h1>

  <LanguageSettings />
  <AppModeSettings />

  <h2 class="section-heading">{$_('appSettings.connectors.title')}</h2>
  <ConnectorSettingsBlock connectorId="obs" />
  <ConnectorSettingsBlock connectorId="youtube" />
  <ConnectorSettingsBlock connectorId="facebook" />
  <ConnectorSettingsBlock connectorId="vmix" />
  <ConnectorSettingsBlock connectorId="atem" />
  <ConnectorSettingsBlock connectorId="broadlink" />
  <ConnectorSettingsBlock connectorId="discord" />

  <h2 class="section-heading">Presentations</h2>
  <section>
    <p class="note">
      Use the built-in web presenter to parse and display slides in the browser
      instead of opening Keynote or PowerPoint. Only <code>.pptx</code> files are supported.
    </p>
    <label class="toggle-label">
      <input type="checkbox" checked={$useWebPresenter} onchange={handleWebPresenterToggle} />
      Use web presenter
    </label>
  </section>

  <ObsBadgeSettings />

  <CronJobsSettings />
  <AppVersionSettings />
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

  .note {
    font-size: 0.875rem;
    color: var(--text-secondary);
    margin: 0.5rem 0 1rem;
  }

  code {
    font-family: monospace;
    font-size: 0.875em;
    background: var(--content-bg);
    padding: 0.1em 0.3em;
    border-radius: 0.25rem;
  }

  .toggle-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
    cursor: pointer;
  }

  .toggle-label input[type='checkbox'] {
    width: 1rem;
    height: 1rem;
    cursor: pointer;
  }
</style>
