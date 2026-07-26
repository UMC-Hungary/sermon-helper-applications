/**
 * E2E tests for the job-queue REST API.
 *
 * Creating a future event enqueues a `youtube.upsert` job, which exercises the
 * migration, the dedup index and the claim/redrive endpoints end to end.
 */

import { describe, it, expect } from 'vitest';
import { apiClient } from '../helpers/client.js';

const isLive = !!process.env.TAURI_TEST_TOKEN;

interface Job {
  id: string;
  jobType: string;
  status: string;
  attempts: number;
  maxAttempts: number;
  lastError: string | null;
}

interface QueueSummary {
  queue: string;
  pending: number;
  processing: number;
  succeeded: number;
  dead: number;
}

describe.skipIf(!isLive)('Queues REST API', () => {
  it('GET /api/queues → per-queue depth by status', async () => {
    const res = await apiClient.get<QueueSummary[]>('/api/queues');
    expect(res.status).toBe(200);
    expect(Array.isArray(res.body)).toBe(true);
  });

  it('creating an event enqueues a youtube.upsert job that survives failure', async () => {
    const dateTime = new Date(Date.now() + 86_400_000).toISOString();
    const created = await apiClient.post<{ id: string }>('/api/events', {
      title: 'Queue test event',
      date_time: dateTime,
    });
    expect(created.status).toBe(201);
    const eventId = created.body.id;

    // YouTube is not connected in the test server, so the job must fail and be
    // rescheduled rather than vanish — that is the durability the queue buys.
    const job = await findJob(eventId);
    expect(job).toBeDefined();
    expect(job?.attempts).toBeGreaterThanOrEqual(1);
    expect(job?.lastError).toBeTruthy();

    const retry = await apiClient.post(`/api/jobs/${job?.id}/retry`);
    expect(retry.status).toBe(204);

    const purge = await apiClient.delete(`/api/jobs/${job?.id}`);
    expect(purge.status).toBe(204);

    await apiClient.delete(`/api/events/${eventId}`);
  });

  it('POST /api/jobs/{id}/retry on an unknown id → 404', async () => {
    const res = await apiClient.post('/api/jobs/00000000-0000-0000-0000-000000000000/retry');
    expect(res.status).toBe(404);
  });
});

/// The upsert job for one event, once the worker has tried it at least once.
async function findJob(eventId: string): Promise<Job | undefined> {
  for (let i = 0; i < 30; i++) {
    const res = await apiClient.get<Job[]>('/api/queues/platform_sync/jobs');
    const job = res.body.find(
      (j) => j.jobType === 'youtube.upsert' && JSON.stringify(j).includes(eventId),
    );
    if (job && job.attempts > 0 && job.status !== 'processing') return job;
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  return undefined;
}
