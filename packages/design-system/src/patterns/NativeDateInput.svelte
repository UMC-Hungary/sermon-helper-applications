<script lang="ts">
  interface Props {
    /** The visible label — the reference shows the formatted value here. */
    label: string;
    /** The mono affordance on the right. Passed in, so it can be translated. */
    hint: string;
    type?: 'date' | 'time' | 'datetime-local';
    value?: string;
    disabled?: boolean;
    id?: string;
    /** Names the control for assistive technology, which the visible value alone does not. */
    accessibleName: string;
  }

  let {
    label,
    hint,
    type = 'date',
    value = $bindable(''),
    disabled = false,
    id = `sanctum-native-date-${crypto.randomUUID()}`,
    accessibleName,
  }: Props = $props();
</script>

<div class="native">
  <label for={id}>
    <span aria-hidden="true">{label}</span>
    <em aria-hidden="true">{hint}</em>
    <span class="visually-hidden">{accessibleName}</span>
  </label>
  <input {id} {type} {disabled} bind:value />
</div>

<style>
  .native {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--ui-stack);
    min-height: var(--ui-target-min);
  }

  label {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--ui-stack);
    width: 100%;
    cursor: pointer;
    font-family: var(--type-body-strong-family);
    font-size: var(--type-body-strong-size);
    color: var(--text-primary);
    font-weight: var(--type-body-strong-weight);
    letter-spacing: var(--type-body-strong-track);
  }

  em {
    font-family: var(--type-label-sm-family);
    font-size: var(--type-label-sm-size);
    letter-spacing: var(--type-label-sm-track);
    color: var(--text-muted);
    text-transform: var(--type-label-sm-transform);
    font-style: normal;
  }

  /*
   * The reference hides the input entirely, which leaves the control unreachable by keyboard.
   * It stays transparent and full-bleed so the whole row is the target, but keeps its box so
   * the focus indicator has somewhere to land.
   */
  input {
    position: absolute;
    inset: 0;
    opacity: 0;
    cursor: pointer;
    width: 100%;
    border: 0;
    padding: 0;
    margin: 0;
  }

  input:focus-visible {
    outline: var(--ui-focus-width) solid var(--accent);
    outline-offset: var(--ui-focus-offset);
    opacity: 1;
  }

  .visually-hidden {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    overflow: hidden;
    clip-path: inset(50%);
    white-space: nowrap;
    border: 0;
  }
</style>
