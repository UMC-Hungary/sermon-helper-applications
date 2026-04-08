import { z } from 'zod';
import { apiFetch } from './client.js';

const ChannelVideoItemSchema = z.object({
  id: z.string(),
  title: z.string(),
  thumbnailUrl: z.string(),
  publishedAt: z.string().nullable().optional(),
  viewCount: z.number().nullable().optional(),
  likeCount: z.number().nullable().optional(),
  duration: z.string().nullable().optional(),
  liveStatus: z.string(),
  scheduledStartTime: z.string().nullable().optional(),
  watchUrl: z.string(),
  privacyStatus: z.string().default('public'),
});

export const ChannelContentSchema = z.object({
  liveBroadcasts: z.array(ChannelVideoItemSchema),
  videos: z.array(ChannelVideoItemSchema),
});

export type ChannelVideoItem = z.infer<typeof ChannelVideoItemSchema>;
export type ChannelContent = z.infer<typeof ChannelContentSchema>;

export function youtubeLogout(): Promise<void> {
  return apiFetch('/api/auth/youtube/logout', z.void(), { method: 'POST' });
}

export function facebookLogout(): Promise<void> {
  return apiFetch('/api/auth/facebook/logout', z.void(), { method: 'POST' });
}

export function triggerYouTubeSchedule(eventId: string): Promise<void> {
  return apiFetch(`/api/connectors/youtube/schedule/${eventId}`, z.void(), { method: 'POST' });
}

export function triggerFacebookSchedule(eventId: string): Promise<void> {
  return apiFetch(`/api/connectors/facebook/schedule/${eventId}`, z.void(), { method: 'POST' });
}

export function fetchYouTubeContent(): Promise<ChannelContent> {
  return apiFetch('/api/connectors/youtube/content', ChannelContentSchema);
}

const StreamKeySchema = z.object({ rtmpUrl: z.string() });

export function fetchYouTubeStreamKey(): Promise<{ rtmpUrl: string }> {
  return apiFetch('/api/connectors/youtube/stream-key', StreamKeySchema);
}

export function fetchFacebookStreamKey(): Promise<{ rtmpUrl: string }> {
  return apiFetch('/api/connectors/facebook/stream-key', StreamKeySchema);
}
