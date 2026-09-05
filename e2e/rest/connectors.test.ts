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
  atem: ConnectorStatus;
  broadlink: ConnectorStatus;
  youtube: ConnectorStatus;
  facebook: ConnectorStatus;
  discord: ConnectorStatus;
  szentiras: ConnectorStatus;
}

interface SzentirasConfig {
  enabled: boolean;
  apiKey: string;
  apiKeySet: boolean;
}

interface VmixConfig {
  enabled: boolean;
  host: string;
  port: number;
}

interface CameraSettings {
  recording: boolean;
  record: { supported: { supportedFormats: unknown[] } };
  storage: { slots: unknown[] };
  stream: { platforms: string[]; active: { platform: string } };
}

interface ConnectorState {
  obs: { isStreaming: boolean; isRecording: boolean } | null;
}

const isLive = !!process.env.TAURI_TEST_TOKEN;

describe.skipIf(!isLive)('Connectors REST API', () => {
  it('GET /api/connectors/status → 200 with all connectors', async () => {
    const res = await apiClient.get<ConnectorStatuses>('/api/connectors/status');
    expect(res.status).toBe(200);
    for (const name of [
      'obs',
      'vmix',
      'atem',
      'broadlink',
      'youtube',
      'facebook',
      'discord',
      'szentiras',
    ]) {
      expect(res.body).toHaveProperty(name);
    }
  });

  it('GET /api/connectors/state → 200 with obs state', async () => {
    const res = await apiClient.get<ConnectorState>('/api/connectors/state');
    expect(res.status).toBe(200);
    expect(res.body).toHaveProperty('obs');
  });

  it('GET /api/connectors/{name}/config → defaults when unset', async () => {
    const res = await apiClient.get<VmixConfig>('/api/connectors/vmix/config');
    expect(res.status).toBe(200);
    expect(res.body).toMatchObject({ enabled: expect.any(Boolean), host: expect.any(String) });
  });

  it('PUT then GET /api/connectors/{name}/config round-trips the stored value', async () => {
    const saved = await apiClient.put('/api/connectors/vmix/config', {
      enabled: false,
      host: 'e2e-vmix.local',
      port: 8099,
    });
    expect(saved.status).toBe(204);

    const res = await apiClient.get<VmixConfig>('/api/connectors/vmix/config');
    expect(res.body).toEqual({ enabled: false, host: 'e2e-vmix.local', port: 8099 });
  });

  it('PUT /api/connectors/{name}/config rejects a mismatched body → 400', async () => {
    const res = await apiClient.put('/api/connectors/vmix/config', { enabled: 'yes' });
    expect(res.status).toBe(400);
  });

  it('GET /api/connectors/nope/config on an unknown connector → 404', async () => {
    const res = await apiClient.get('/api/connectors/nope/config');
    expect(res.status).toBe(404);
  });

  it('never returns a stored secret, and keeps it when saved blank', async () => {
    const saved = await apiClient.put('/api/connectors/szentiras/config', {
      enabled: true,
      apiKey: 'e2e-secret-value',
    });
    expect(saved.status).toBe(204);

    // Reading it back must not leak the key, only that one is stored.
    const read = await apiClient.get<SzentirasConfig>('/api/connectors/szentiras/config');
    expect(read.status).toBe(200);
    expect(read.body.apiKey).toBe('');
    expect(read.body.apiKeySet).toBe(true);
    expect(JSON.stringify(read.body)).not.toContain('e2e-secret-value');

    // Saving the redacted form back must not wipe the key.
    await apiClient.put('/api/connectors/szentiras/config', {
      enabled: false,
      apiKey: '',
      apiKeySet: true,
    });
    const afterBlankSave = await apiClient.get<SzentirasConfig>('/api/connectors/szentiras/config');
    expect(afterBlankSave.body.enabled).toBe(false);
    expect(afterBlankSave.body.apiKeySet).toBe(true);

    // A non-empty value replaces it; an explicit clear needs its own step.
    await apiClient.put('/api/connectors/szentiras/config', {
      enabled: false,
      apiKey: 'e2e-replacement',
    });
    const replaced = await apiClient.get<SzentirasConfig>('/api/connectors/szentiras/config');
    expect(replaced.body.apiKeySet).toBe(true);
    expect(replaced.body.apiKey).toBe('');

    // Clearing is explicit: apiKeySet false wipes it.
    await apiClient.put('/api/connectors/szentiras/config', {
      enabled: false,
      apiKey: '',
      apiKeySet: false,
    });
    const cleared = await apiClient.get<SzentirasConfig>('/api/connectors/szentiras/config');
    expect(cleared.body.apiKeySet).toBe(false);
  });

  it('szentiras reports connected once an API key is stored', async () => {
    const saved = await apiClient.put('/api/connectors/szentiras/config', {
      enabled: true,
      apiKey: 'e2e-placeholder-key',
    });
    expect(saved.status).toBe(204);

    const statuses = await apiClient.get<ConnectorStatuses>('/api/connectors/status');
    expect(statuses.body.szentiras.type).toBe('connected');

    // Leave no key behind for the other tests.
    await apiClient.put('/api/connectors/szentiras/config', {
      enabled: false,
      apiKey: '',
      apiKeySet: false,
    });
    const cleared = await apiClient.get<ConnectorStatuses>('/api/connectors/status');
    expect(cleared.body.szentiras.type).toBe('disconnected');
  });

  it('GET /api/connectors/obs/stream-settings while OBS is offline → 409', async () => {
    const res = await apiClient.get('/api/connectors/obs/stream-settings');
    expect(res.status).toBe(409);
  });

  // A camera is rarely on the test network, so the interesting assertion is the
  // shape when one answers — and the 409 contract when none is connected.
  it('GET /api/connectors/blackmagic-camera/settings → 409, or the camera settings', async () => {
    const res = await apiClient.get<CameraSettings>('/api/connectors/blackmagic-camera/settings');
    if (res.status === 409) return;
    expect(res.status).toBe(200);
    expect(typeof res.body.recording).toBe('boolean');
    expect(res.body.record.supported.supportedFormats.length).toBeGreaterThan(0);
    expect(res.body.storage.slots.length).toBeGreaterThan(0);
    expect(res.body.stream.active.platform.length).toBeGreaterThan(0);
    expect(res.body.stream.platforms).toContain(res.body.stream.active.platform);
  });

  it('PUT /api/connectors/blackmagic-camera/settings with no camera → 409', async () => {
    const res = await apiClient.get('/api/connectors/blackmagic-camera/settings');
    if (res.status !== 409) return;
    const written = await apiClient.put('/api/connectors/blackmagic-camera/settings', {});
    expect(written.status).toBe(409);
  });
});
