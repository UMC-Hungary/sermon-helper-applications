<script lang="ts" module>
  export interface SlideResult {
    id: string;
    /** The deck or folder the slide belongs to. */
    group: string;
    title: string;
    /** A short position marker — "12 / 40". */
    position?: string;
  }
</script>

<script lang="ts">
  interface Props {
    results: SlideResult[];
    /** Names the search region. */
    label: string;
    /** The mono line above the current selection in the trigger. */
    triggerLabel: string;
    /** What is selected now, shown in the trigger. */
    selection: string;
    emptyMessage: string;
    openLabel: string;
    queueLabel: string;
    queueDisabled?: boolean;
    ontrigger?: () => void;
    onopen?: (result: SlideResult) => void;
    onqueue?: (result: SlideResult) => void;
  }

  let {
    results,
    label,
    triggerLabel,
    selection,
    emptyMessage,
    openLabel,
    queueLabel,
    queueDisabled = false,
    ontrigger,
    onopen,
    onqueue,
  }: Props = $props();
</script>

<section class="search" aria-label={label}>
  <button class="trigger" type="button" onclick={ontrigger}>
    <span>
      <em>{triggerLabel}</em>
      <strong>{selection}</strong>
    </span>
  </button>

  {#if results.length === 0}
    <p class="empty">{emptyMessage}</p>
  {:else}
    <ul class="results">
      {#each results as result (result.id)}
        <li>
          <button
            class="open"
            type="button"
            aria-label={`${openLabel}: ${result.title}`}
            onclick={() => onopen?.(result)}
          >
            <code>{result.position ?? ''}</code>
            <span>{result.title}</span>
            <em>{result.group}</em>
          </button>
          <button
            class="queue"
            type="button"
            disabled={queueDisabled}
            aria-label={`${queueLabel}: ${result.title}`}
            onclick={() => onqueue?.(result)}
          >
            +
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .trigger {
    width: 100%;
    min-height: var(--ui-target-min);
    display: flex;
    align-items: center;
    gap: var(--ui-gutter-tight);
    padding: var(--ui-gutter-inset);
    background: var(--surface-raised);
    border: var(--ui-border-hairline) solid var(--border-control);
    color: var(--text-primary);
    cursor: pointer;
    text-align: left;
  }

  .trigger span {
    flex: 1;
    min-width: 0;
  }

  em,
  code {
    display: block;
    font-family: var(--type-label-sm-family);
    font-size: var(--type-label-sm-size);
    letter-spacing: var(--c-search-label-track);
    text-transform: var(--type-label-sm-transform);
    color: var(--text-muted);
    font-style: normal;
  }

  strong {
    display: block;
    margin-top: var(--c-search-title-gap);
    font-family: var(--type-body-family);
    font-size: var(--type-body-size);
    font-weight: var(--type-body-strong-weight);
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .results {
    margin: 0;
    padding: 0;
    list-style: none;
    border-left: var(--ui-border-hairline) solid var(--border-control);
    border-right: var(--ui-border-hairline) solid var(--border-control);
    background: var(--surface-raised);
  }

  li {
    display: grid;
    grid-template-columns: 1fr auto;
    border-bottom: var(--ui-border-hairline) solid var(--border-hairline);
  }

  .open {
    min-width: 0;
    min-height: var(--ui-target-min);
    display: grid;
    grid-template-columns: auto 1fr;
    gap: var(--c-search-item-row-gap) var(--c-search-item-column-gap);
    padding: var(--c-search-item-padding-block) var(--ui-gutter-inset);
    border: 0;
    background: transparent;
    color: var(--text-primary);
    text-align: left;
    cursor: pointer;
  }

  .open span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .open em {
    grid-column: 2;
  }

  .queue {
    min-width: var(--ui-target-min);
    padding: 0 var(--ui-gutter-tight);
    border: 0;
    border-left: var(--ui-border-hairline) solid var(--border-hairline);
    background: transparent;
    color: var(--text-primary);
    cursor: pointer;
  }

  .queue:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  button:focus-visible {
    outline: var(--ui-focus-width) solid var(--accent);
    outline-offset: calc(var(--ui-focus-offset) * -1);
  }

  .empty {
    margin: 0;
    padding: var(--c-search-empty-padding-block) var(--ui-gutter-inset);
    color: var(--text-muted);
    font-family: var(--type-quote-family);
    font-style: italic;
  }
</style>
