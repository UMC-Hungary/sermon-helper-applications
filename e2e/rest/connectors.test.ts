/**
 * E2E tests for Connectors REST API.
 */

import { describe, it, expect } from 'vitest';
import { apiClient } from '../helpers/client.js';

interface ConnectorStatus {
  type: 'disconnected' | 'connecting' | 'connected' | 'error';
  message?: string;
}

interface ConnectorStatuses {
  obs: ConnectorStatus;
  vmix: ConnectorStatus;
  youtube: ConnectorStatus;
  facebook: ConnectorStatus;
}

interface ConnectorState {
  obs: { isStreaming: boolean; isRecording: boolean } | null;
}

const isLive = !!process.env.TAURI_TEST_TOKEN;

describe.skipIf(!isLive)('Connectors REST API', () => {
  it('GET /api/connectors/status → 200 with all connectors', async () => {
    const res = await apiClient.get<ConnectorStatuses>('/api/connectors/status');
    expect(res.status).toBe(200);
    expect(res.body).toHaveProperty('obs');
    expect(res.body).toHaveProperty('vmix');
    expect(res.body).toHaveProperty('youtube');
    expect(res.body).toHaveProperty('facebook');
  });

  it('GET /api/connectors/state → 200 with obs state', async () => {
    const res = await apiClient.get<ConnectorState>('/api/connectors/state');
    expect(res.status).toBe(200);
    expect(res.body).toHaveProperty('obs');
  });
});
