<script lang="ts">
  import type { ConnectorStatus } from '$lib/stores/connectors.js';

  interface Props {
    name: string;
    status: ConnectorStatus;
  }

  let { name, status }: Props = $props();

  const statusLabels: Record<ConnectorStatus, string> = {
    connected: 'Connected',
    connecting: 'Connecting',
    disconnected: 'Disconnected',
    error: 'Error'
  };
</script>

<span class="badge badge--{status}">
  <span class="dot" aria-hidden="true"></span>
  <span>{name}</span>
  <span class="status-label">{statusLabels[status]}</span>
</span>

<style>
  .badge {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.2rem 0.6rem;
    border-radius: 9999px;
    font-size: 0.8rem;
    font-weight: 500;
    background: #f3f4f6;
    color: #374151;
  }

  .dot {
    width: 0.5rem;
    height: 0.5rem;
    border-radius: 50%;
    background: #9ca3af;
    flex-shrink: 0;
  }

  .status-label {
    color: inherit;
    opacity: 0.75;
  }

  /* Connected — green */
  .badge--connected {
    background: #d1fae5;
    color: #065f46;
  }
  .badge--connected .dot {
    background: #10b981;
  }

  /* Connecting — yellow */
  .badge--connecting {
    background: #fef3c7;
    color: #92400e;
  }
  .badge--connecting .dot {
    background: #f59e0b;
  }

  /* Disconnected — grey */
  .badge--disconnected {
    background: #f3f4f6;
    color: #6b7280;
  }
  .badge--disconnected .dot {
    background: #9ca3af;
  }

  /* Error — red */
  .badge--error {
    background: #fee2e2;
    color: #991b1b;
  }
  .badge--error .dot {
    background: #ef4444;
  }
</style>
