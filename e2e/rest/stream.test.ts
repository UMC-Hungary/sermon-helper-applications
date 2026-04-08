/**
 * E2E tests for Stream REST API.
 */

import { describe, it, expect } from 'vitest';
import { apiClient } from '../helpers/client.js';

interface StreamStats {
  ready: boolean;
  bytesReceived: number;
  bytesSent: number;
  readers: number;
  tracks: string[];
}

const isLive = !!process.env.TAURI_TEST_TOKEN;

describe.skipIf(!isLive)('Stream REST API', () => {
  it('GET /api/stream/stats → 200 with correct shape', async () => {
    const res = await apiClient.get<StreamStats>('/api/stream/stats');
    expect(res.status).toBe(200);
    expect(typeof res.body.ready).toBe('boolean');
    expect(typeof res.body.bytesReceived).toBe('number');
    expect(typeof res.body.bytesSent).toBe('number');
    expect(typeof res.body.readers).toBe('number');
    expect(Array.isArray(res.body.tracks)).toBe(true);
  });
});
