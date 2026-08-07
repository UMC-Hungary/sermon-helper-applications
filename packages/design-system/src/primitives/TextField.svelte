<script lang="ts">
  interface Props {
    value?: string;
    type?: 'text' | 'email' | 'url' | 'tel' | 'search' | 'password' | 'number';
    placeholder?: string;
    disabled?: boolean;
    readonly?: boolean;
    required?: boolean;
    invalid?: boolean;
    /** Supplied by `FormField`, which owns the label and the notes. */
    id?: string;
    describedby?: string;
    /** Only when the field stands outside a `FormField`. */
    label?: string;
  }

  let {
    value = $bindable(''),
    type = 'text',
    placeholder = '',
    disabled = false,
    readonly = false,
    required = false,
    invalid = false,
    id,
    describedby,
    label,
  }: Props = $props();
</script>

<input
  {id}
  {type}
  {placeholder}
  {disabled}
  {readonly}
  {required}
  bind:value
  aria-label={label}
  aria-invalid={invalid ? 'true' : undefined}
  aria-describedby={describedby}
  class:invalid
/>

<style>
  input {
    width: 100%;
    min-height: var(--ui-target-min);
    padding: var(--c-text-field-padding-block) var(--ui-gutter-inset);
    background: var(--surface-sunken);
    border: var(--ui-border-hairline) solid var(--border-control);
    border-radius: var(--ui-radius-square);
    font-family: var(--type-body-strong-family);
    font-size: var(--type-body-strong-size);
    font-weight: var(--type-body-strong-weight);
    letter-spacing: var(--type-body-strong-track);
    color: var(--text-primary);
    caret-color: var(--text-primary);
  }

  input::placeholder {
    color: var(--text-faint);
  }

  input:focus-visible {
    outline: var(--ui-focus-width) solid var(--accent);
    outline-offset: var(--ui-focus-offset);
    border-color: var(--accent);
  }

  .invalid {
    border-color: var(--status-error);
  }

  input:disabled {
    color: var(--text-muted);
    cursor: not-allowed;
  }
</style>
