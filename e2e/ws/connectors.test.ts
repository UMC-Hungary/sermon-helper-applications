/**
 * E2E tests for Connectors WebSocket commands.
 */

import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { WsTestClient } from '../helpers/ws-client.js';

const isLive = !!process.env.TAURI_TEST_TOKEN;

describe.skipIf(!isLive)('Connectors WebSocket Commands', () => {
  let ws: WsTestClient;

  beforeAll(async () => {
    ws = new WsTestClient();
    await ws.waitForConnect();
  });

  afterAll(() => {
    ws.close();
  });

  it('connectors.status → connectors.status response', async () => {
    const msg = await ws.command({ type: 'connectors.status' }, 'connectors.status');
    expect(msg['type']).toBe('connectors.status');
    expect(msg).toHaveProperty('obs');
    expect(msg).toHaveProperty('vmix');
    expect(msg).toHaveProperty('youtube');
    expect(msg).toHaveProperty('facebook');
  });

  it('connectors.state → connectors.state response', async () => {
    const msg = await ws.command({ type: 'connectors.state' }, 'connectors.state');
    expect(msg['type']).toBe('connectors.state');
    expect(msg).toHaveProperty('obs');
  });
});
