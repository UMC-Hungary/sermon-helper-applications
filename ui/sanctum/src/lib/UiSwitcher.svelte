<script lang="ts">
  import { onMount } from 'svelte';
  import { _ } from 'svelte-i18n';
  import { loadBundledUis, getActiveUi, setActiveUi } from '@metocast/core-client';
  import type { BundledUi } from '@metocast/core-client';

  let uis = $state<BundledUi[]>([]);
  let active = $state('');

  onMount(async () => {
    const manifest = await loadBundledUis();
    if (!manifest) return;
    uis = manifest.uis;
    active = getActiveUi(manifest.active);
  });

  // Persist over the shared key, then load the chosen UI immediately.
  function choose(ui: BundledUi) {
    setActiveUi(ui.id);
    window.location.assign(ui.path);
  }
</script>

{#if uis.length > 1}
  <section>
    <h2>{$_('ui.activeInterface')}</h2>
    <div class="list">
      {#each uis as ui (ui.id)}
        <button class:on={active === ui.id} onclick={() => choose(ui)}>
          <strong>{ui.displayName}</strong>
          {#if ui.description}<span>{ui.description}</span>{/if}
        </button>
      {/each}
    </div>
  </section>
{/if}

<style>
  h2 {
    font-family: var(--font-label);
    font-size: var(--type-label-size, 0.7rem);
    letter-spacing: 1.4px;
    text-transform: uppercase;
    color: var(--text-muted);
    margin: 0 0 var(--space-8, 0.5rem);
  }
  .list {
    display: grid;
    gap: var(--border-1, 1px);
    border: var(--border-1, 1px) solid color-mix(in srgb, var(--text-primary) 12%, transparent);
  }
  button {
    display: block;
    width: 100%;
    text-align: left;
    padding: var(--space-12, 0.75rem) var(--space-14, 0.9rem);
    border: 0;
    border-left: var(--border-2, 2px) solid transparent;
    background: transparent;
    color: inherit;
    cursor: pointer;
    font: inherit;
  }
  button.on {
    background: var(--surface-raised);
    border-left-color: var(--text-primary);
  }
  strong {
    display: block;
    font-family: var(--font-body);
  }
  span {
    display: block;
    font-size: var(--type-caption-size, 0.8rem);
    color: var(--text-muted);
  }
</style>
