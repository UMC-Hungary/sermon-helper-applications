<script lang="ts">
  import { onMount } from 'svelte';
  import { _ } from 'svelte-i18n';
  import { loadBundledUis, getActiveUi, setActiveUi } from '$lib/core-client/index.js';
  import type { BundledUi } from '$lib/core-client/index.js';

  let uis = $state<BundledUi[]>([]);
  let active = $state('');

  // Written by scripts/build-ui.mjs next to the bundled UIs. A build with one UI
  // still writes it, so the absence of the file just means "nothing to choose".
  onMount(async () => {
    const manifest = await loadBundledUis();
    if (!manifest) return; // Dev server, or a build with nothing to choose.
    uis = manifest.uis;
    active = getActiveUi(manifest.active);
  });

  function choose(id: string) {
    active = id;
    setActiveUi(id);
  }
</script>

{#if uis.length > 1}
  <section>
    <h2>{$_('appSettings.activeUi.title')}</h2>
    <p class="note">{$_('appSettings.activeUi.description')}</p>

    {#each uis as ui (ui.id)}
      <label class="ui-option">
        <input
          type="radio"
          name="active-ui"
          value={ui.id}
          checked={active === ui.id}
          onchange={() => choose(ui.id)}
        />
        <span>
          <strong>{ui.displayName}</strong>
          {#if ui.description}<span class="ui-description">{ui.description}</span>{/if}
        </span>
      </label>
    {/each}

    <p class="note">{$_('appSettings.activeUi.appliesOnRestart')}</p>
  </section>
{/if}

<style>
  .ui-option {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
    padding: 0.375rem 0;
  }

  .ui-description {
    display: block;
    font-size: 0.8125rem;
    color: var(--text-secondary);
  }

  .note {
    font-size: 0.8125rem;
    color: var(--text-secondary);
  }
</style>
