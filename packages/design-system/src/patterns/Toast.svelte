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
    /** Numbered remediation steps, disclosed by the "why" toggle in the footer. */
    remediation?: string[];
    /** The disclosure toggle's labels — closed then open. */
    whyLabel?: string;
    hideWhyLabel?: string;
    /** Extra content between the body and footer, e.g. a folded group of sources. */
    detail?: Snippet;
  }

  let {
    kind,
    source,
    title,
    body = '',
    state: stateLabel,
    tone = 'ok',
    mono = false,
    actions = [],
    dismissLabel = 'Dismiss',
    ondismiss,
    mark,
    remediation,
    whyLabel = 'Why?',
    hideWhyLabel = 'Hide why',
    detail,
  }: Props = $props();

  let expanded = $state(false);
  const numeral = (i: number) => String(i + 1).padStart(2, '0');

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
      {#if stateLabel}
        <span class="state">
          <Dot color={accents[tone]} size={4} pulse={stateLabel === 'reconnecting'} />
          {stateLabel}
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
    {#if expanded && remediation?.length}
      <ol class="why">
        {#each remediation as step, i (i)}
          <li><code>{numeral(i)}</code>{step}</li>
        {/each}
      </ol>
    {/if}
    {#if actions.length > 0 || remediation?.length}
      <footer>
        {#each actions as action (action.label)}
          <button type="button" class:primary={action.primary} onclick={action.onclick}>
            {action.label}
          </button>
        {/each}
        {#if remediation?.length}
          <button type="button" class="toggle" aria-expanded={expanded} onclick={() => (expanded = !expanded)}>
            {expanded ? hideWhyLabel : whyLabel}
          </button>
        {/if}
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
    min-width: 340px;
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

  .why {
    margin: var(--c-toast-why-margin) 0 0;
    padding: var(--c-toast-why-padding-block) var(--c-toast-why-padding-inline);
    background: var(--surface-sunken);
    border-left: var(--ui-border-emphasis) solid var(--sanctum-toast-accent);
    list-style: none;
  }

  .why li {
    display: flex;
    gap: var(--c-toast-why-step-gap);
    margin-bottom: var(--c-toast-why-step-margin);
    font-family: var(--type-quote-family);
    font-style: italic;
    font-size: var(--c-toast-why-step-size);
    color: var(--text-secondary);
  }

  .why li:last-child {
    margin-bottom: 0;
  }

  .why code {
    font-family: var(--type-label-family);
    color: var(--text-muted);
  }

  footer {
    display: flex;
    gap: var(--c-toast-footer-gap);
    margin-top: var(--c-toast-footer-margin);
    flex-wrap: wrap;
    align-items: center;
  }

  footer button {
    border: var(--ui-border-hairline) solid var(--sanctum-toast-accent);
    background: transparent;
    color: var(--sanctum-toast-accent);
    padding: var(--c-toast-action-padding-block) var(--c-toast-action-padding);
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

  footer .toggle {
    margin-left: auto;
    border: 0;
    color: var(--text-muted);
    padding-left: 0;
    padding-right: 0;
  }
</style>
