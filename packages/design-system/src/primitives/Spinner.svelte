<script lang="ts">
  interface Props {
    size?: number;
    /** Announced while the work is in progress. Required, so the wait is never silent. */
    label: string;
  }

  let { size = 20, label }: Props = $props();
</script>

<span class="spinner" role="status" aria-live="polite" style:--sanctum-spinner-size={`${size}px`}>
  <svg viewBox="0 0 24 24" aria-hidden="true" focusable="false">
    <circle cx="12" cy="12" r="9" class="track" />
    <path d="M21 12a9 9 0 0 0-9-9" class="head" />
  </svg>
  <span class="visually-hidden">{label}</span>
</span>

<style>
  .spinner {
    display: inline-flex;
    width: var(--sanctum-spinner-size);
    height: var(--sanctum-spinner-size);
    flex-shrink: 0;
  }

  svg {
    width: 100%;
    height: 100%;
    fill: none;
    stroke-width: var(--ui-border-emphasis);
    stroke-linecap: round;
    animation: sanctum-spin var(--motion-pulse) linear infinite;
  }

  .track {
    stroke: var(--border-control);
  }

  .head {
    stroke: var(--text-primary);
  }

  @keyframes sanctum-spin {
    to {
      transform: rotate(360deg);
    }
  }

  /*
   * Reduced motion stills the rotation through the duration token, which would leave a static
   * arc with no indication that anything is happening — so the arc pulses in opacity instead.
   */
  @media (prefers-reduced-motion: reduce) {
    svg {
      animation: sanctum-spinner-breathe 2s ease-in-out infinite;
    }
  }

  @keyframes sanctum-spinner-breathe {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.4;
    }
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
