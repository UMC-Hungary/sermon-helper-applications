<script lang="ts" module>
  export type ToastTone = 'live' | 'ok' | 'warn' | 'error';

  export interface ToastAction {
    label: string;
    primary?: boolean;
    onclick: () => void;
  }
</script>

<script lang="ts">
  import type { Snippet } from 'svelte';
  import Dot from '../primitives/Dot.svelte';

  interface Props {
    /** The mono kind label above the source name — "Connector", "Upload". */
    kind: string;
    /** Who is speaking — "YouTube", "OBS". */
    source: string;
    title: string;
    body?: string;
    /** A short state chip beside the source — "reconnecting", "offline". */
    state?: string;
    tone?: ToastTone;
    /** Renders the body in the mono face, for log lines and identifiers. */
    mono?: boolean;
    actions?: ToastAction[];
    dismissLabel?: string;
    ondismiss?: () => void;
    /** The tile that identifies the source, usually a `Glyph`. */
    mark?: Snippet;
    /** The numbered "why" list, shown when the caller expands it. */
    detail?: Snippet;
  }

  let {
    kind,
    source,
    title,
    body = '',
    state,
    tone = 'ok',
    mono = false,
    actions = [],
    dismissLabel = 'Dismiss',
    ondismiss,
    mark,
    detail,
  }: Props = $props();

  const accents: Record<ToastTone, string> = {
    live: 'var(--status-live)',
    ok: 'var(--status-ok)',
    warn: 'var(--status-warn)',
    error: 'var(--status-error)',
  };
</script>

<article style:--sanctum-toast-accent={accents[tone]}>
  <i aria-hidden="true"></i>
  <div class="content">
    <header>
      {#if mark}{@render mark()}{/if}
      <p class="who">
        <small>{kind}</small>
        <strong>{source}</strong>
      </p>
      {#if state}
        <span class="state">
          <Dot color={accents[tone]} size={4} pulse={tone === 'live'} />
          {state}
        </span>
      {/if}
      {#if ondismiss}
        <button class="dismiss" type="button" aria-label={dismissLabel} onclick={ondismiss}>
          ×
        </button>
      {/if}
    </header>
    <h3>{title}</h3>
    {#if body}<p class="body" class:mono>{body}</p>{/if}
    {#if detail}{@render detail()}{/if}
    {#if actions.length > 0}
      <footer>
        {#each actions as action (action.label)}
          <button type="button" class:primary={action.primary} onclick={action.onclick}>
            {action.label}
          </button>
        {/each}
      </footer>
    {/if}
  </div>
</article>

<style>
  article {
    display: grid;
    grid-template-columns: var(--c-toast-accent-width) 1fr;
    background: var(--surface-raised);
    border: var(--ui-border-hairline) solid var(--border-control);
    width: 100%;
  }

  i {
    background: var(--sanctum-toast-accent);
  }

  .content {
    padding: var(--c-toast-padding-block) var(--ui-gutter-inset);
    min-width: 0;
  }

  header {
    display: flex;
    align-items: center;
    gap: var(--c-toast-header-gap);
    margin-bottom: var(--c-toast-header-margin);
  }

  .who {
    flex: 1;
    min-width: 0;
    margin: 0;
  }

  small {
    display: block;
    font-family: var(--type-label-xs-family);
    font-size: var(--type-label-xs-size);
    letter-spacing: var(--type-label-xs-track);
    text-transform: var(--type-label-xs-transform);
    color: var(--text-muted);
  }

  strong {
    display: block;
    font-size: var(--c-toast-source-size);
    font-weight: var(--type-body-sm-weight);
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .state {
    display: inline-flex;
    align-items: center;
    gap: var(--c-toast-state-gap);
    padding: var(--c-toast-state-padding-block) var(--c-toast-state-padding-inline);
    border: var(--ui-border-hairline) solid var(--sanctum-toast-accent);
    color: var(--sanctum-toast-accent);
    font-family: var(--type-label-xs-family);
    font-size: var(--type-label-xs-size);
    letter-spacing: var(--c-toast-state-track);
    text-transform: var(--type-label-xs-transform);
  }

  .dismiss {
    min-width: var(--ui-target-min);
    min-height: var(--ui-target-min);
    background: transparent;
    border: 0;
    cursor: pointer;
    color: var(--text-muted);
    font-family: var(--type-label-family);
    font-size: var(--type-caption-size);
  }

  button:focus-visible {
    outline: var(--ui-focus-width) solid var(--accent);
    outline-offset: calc(var(--ui-focus-offset) * -1);
  }

  h3 {
    margin: 0;
    font-family: var(--type-heading-family);
    font-size: var(--type-heading-size);
    font-weight: var(--type-heading-weight);
    color: var(--text-primary);
    line-height: var(--type-heading-leading);
  }

  .body {
    margin: var(--c-toast-body-gap) 0 0;
    font-size: var(--type-caption-size);
    color: var(--text-secondary);
    line-height: var(--type-caption-leading);
  }

  .mono {
    font-family: var(--type-label-family);
    font-size: var(--c-toast-mono-size);
  }

  footer {
    display: flex;
    gap: var(--c-toast-footer-gap);
    margin-top: var(--c-toast-footer-margin);
    flex-wrap: wrap;
    align-items: center;
  }

  footer button {
    min-height: var(--ui-target-min);
    border: var(--ui-border-hairline) solid var(--sanctum-toast-accent);
    background: transparent;
    color: var(--sanctum-toast-accent);
    padding: 0 var(--c-toast-action-padding);
    cursor: pointer;
    font-family: var(--type-label-family);
    font-size: var(--c-toast-action-size);
    letter-spacing: var(--c-toast-action-track);
    text-transform: var(--type-label-transform);
  }

  footer .primary {
    background: var(--sanctum-toast-accent);
    color: var(--surface-raised);
  }
</style>
