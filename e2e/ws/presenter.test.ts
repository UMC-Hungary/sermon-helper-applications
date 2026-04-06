/**
 * E2E tests for Presenter WebSocket commands.
 */

import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { WsTestClient } from '../helpers/ws-client.js';

const isLive = !!process.env.TAURI_TEST_TOKEN;

describe.skipIf(!isLive)('Presenter WebSocket Commands', () => {
  let ws: WsTestClient;

  beforeAll(async () => {
    ws = new WsTestClient();
    await ws.waitForConnect();
  });

  afterAll(() => {
    ws.close();
  });

  it('presenter.status → presenter.state response', async () => {
    const msg = await ws.command({ type: 'presenter.status' }, 'presenter.state');
    expect(msg['type']).toBe('presenter.state');
    expect(msg['state']).toHaveProperty('loaded');
    expect(msg['state']).toHaveProperty('currentSlide');
    expect(msg['state']).toHaveProperty('totalSlides');
    expect(Array.isArray((msg['state'] as { slides: unknown[] }).slides)).toBe(true);
  });

  it('presenter.load with non-existent file → error response', async () => {
    const msg = await ws.command(
      { type: 'presenter.load', file_path: '/tmp/e2e-nonexistent.pptx' },
      'error',
    );
    expect(msg['type']).toBe('error');
    expect(typeof msg['message']).toBe('string');
  });

  it('presenter.next on unloaded state → no crash (no-op)', async () => {
    // First ensure unloaded state
    await ws.command({ type: 'presenter.unload' }, 'presenter.state');
    // next on an empty presenter should silently no-op; request status to verify
    ws.send({ type: 'presenter.next' });
    const msg = await ws.command({ type: 'presenter.status' }, 'presenter.state');
    expect((msg['state'] as { loaded: boolean }).loaded).toBe(false);
  });
});
