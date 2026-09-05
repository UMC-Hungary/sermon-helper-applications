/**
 * E2E tests for the slide output folder setting. The core stores the path
 * because the core is what writes the decks, so it — not the UI — is what has
 * to reject a folder that is not there.
 */

import { describe, it, expect, afterAll } from 'vitest';
import { mkdtemp, readdir, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { apiClient } from '../helpers/client.js';

interface SlideFolder {
  path: string;
}

const isLive = !!process.env.TAURI_TEST_TOKEN;

describe.skipIf(!isLive)('Slide folder REST API', () => {
  afterAll(async () => {
    await apiClient.put('/api/settings/slide-folder', { path: '' });
  });

  it('GET → 200 with an empty path when unset', async () => {
    await apiClient.put('/api/settings/slide-folder', { path: '' });
    const res = await apiClient.get<SlideFolder>('/api/settings/slide-folder');
    expect(res.status).toBe(200);
    expect(res.body.path).toBe('');
  });

  it('PUT round-trips a folder that exists', async () => {
    const dir = process.cwd();
    const put = await apiClient.put<SlideFolder>('/api/settings/slide-folder', { path: dir });
    expect(put.status).toBe(200);

    const get = await apiClient.get<SlideFolder>('/api/settings/slide-folder');
    expect(get.body.path).toBe(dir);
  });

  it('PUT → 400 for a folder that is not on the core’s machine', async () => {
    const res = await apiClient.put('/api/settings/slide-folder', {
      path: '/definitely/not/a/real/folder',
    });
    expect(res.status).toBe(400);
  });
});

describe.skipIf(!isLive)('Bible slide deck generation', () => {
  let dir = '';
  let eventId = '';

  afterAll(async () => {
    if (eventId) await apiClient.delete(`/api/events/${eventId}`);
    await apiClient.put('/api/settings/slide-folder', { path: '' });
    if (dir) await rm(dir, { recursive: true, force: true });
  });

  it('writes one deck per Bible reference into the configured folder', async () => {
    dir = await mkdtemp(join(tmpdir(), 'metocast-slides-'));
    await apiClient.put('/api/settings/slide-folder', { path: dir });

    const created = await apiClient.post<{ id: string }>('/api/events', {
      title: 'E2E Slide Event',
      date_time: new Date(Date.now() + 86400000).toISOString(),
      speaker: 'Test Speaker',
      description: '',
      bible_references: [
        {
          type: 'textus',
          reference: 'Jn 3,16',
          translation: 'RUF_v2',
          verses: [{ chapter: 3, verse: 16, text: 'Mert úgy szerette Isten a világot…' }],
        },
      ],
    });
    eventId = created.body.id;

    const res = await apiClient.post<{ files: string[] }>(`/api/events/${eventId}/slides`);
    expect(res.status).toBe(200);
    expect(res.body.files).toHaveLength(1);
    expect(await readdir(dir)).toEqual(['textus.pptx']);
  });

  it('POST → 400 when no folder is configured', async () => {
    await apiClient.put('/api/settings/slide-folder', { path: '' });
    const res = await apiClient.post(`/api/events/${eventId}/slides`);
    expect(res.status).toBe(400);
  });
});
