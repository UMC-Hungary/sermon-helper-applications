import { z } from 'zod';
import { apiFetch } from './client.js';

export const StreamStatsSchema = z.object({
  ready: z.boolean(),
  bytesReceived: z.number(),
  bytesSent: z.number(),
  readers: z.number(),
  tracks: z.array(z.string()),
});

export type StreamStats = z.infer<typeof StreamStatsSchema>;

export function getStreamStats(): Promise<StreamStats> {
  return apiFetch('/api/stream/stats', StreamStatsSchema);
}
