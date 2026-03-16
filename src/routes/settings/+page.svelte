<script lang="ts">
  import { _ } from 'svelte-i18n';
  import ConnectorSettingsBlock from '$lib/components/connectors/ConnectorSettingsBlock.svelte';
  import MediamtxDownloadManager from '$lib/components/MediamtxDownloadManager.svelte';
  import LanguageSettings from '$lib/components/settings/LanguageSettings.svelte';
  import AppModeSettings from '$lib/components/settings/AppModeSettings.svelte';
  import ObsBadgeSettings from '$lib/components/settings/ObsBadgeSettings.svelte';
  import CronJobsSettings from '$lib/components/settings/CronJobsSettings.svelte';
</script>

<div class="settings-container">
  <h1>{$_('appSettings.title')}</h1>

  <LanguageSettings />
  <AppModeSettings />

  <h2 class="section-heading">{$_('appSettings.dependencies.title')}</h2>
  <section>
    <p class="note">{$_('appSettings.dependencies.note')}</p>
    <MediamtxDownloadManager />
  </section>

  <h2 class="section-heading">{$_('appSettings.connectors.title')}</h2>
  <ConnectorSettingsBlock connectorId="obs" />
  <ConnectorSettingsBlock connectorId="youtube" />
  <ConnectorSettingsBlock connectorId="facebook" />
  <ConnectorSettingsBlock connectorId="vmix" />
  <ConnectorSettingsBlock connectorId="atem" />
  <ConnectorSettingsBlock connectorId="broadlink" />
  <ConnectorSettingsBlock connectorId="discord" />

  <ObsBadgeSettings />

  <CronJobsSettings />
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
</style>
