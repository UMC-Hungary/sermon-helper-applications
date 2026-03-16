/**
 * E2E tests for Stream WebSocket commands.
 */

import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { WsTestClient } from '../helpers/ws-client.js';

const isLive = !!process.env.TAURI_TEST_TOKEN;

describe.skipIf(!isLive)('Stream WebSocket Commands', () => {
  let ws: WsTestClient;

  beforeAll(async () => {
    ws = new WsTestClient();
    await ws.waitForConnect();
  });

  afterAll(() => {
    ws.close();
  });

  it('stream.stats → stream.stats response', async () => {
    const msg = await ws.command({ type: 'stream.stats' }, 'stream.stats');
    expect(msg['type']).toBe('stream.stats');
    const stats = msg['stats'] as Record<string, boolean | number | string[]>;
    expect(stats).toHaveProperty('ready');
    expect(stats).toHaveProperty('bytesReceived');
    expect(stats).toHaveProperty('bytesSent');
    expect(stats).toHaveProperty('readers');
    expect(stats).toHaveProperty('tracks');
  });
});
