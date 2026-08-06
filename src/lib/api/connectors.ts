import { z } from 'zod';
import { apiFetch } from './client.js';
import { getAdminToken } from '$lib/host/index.js';
import {
  ConnectorConfigSchemas,
  ConnectorStatusesSchema,
  ObsStreamSettingsSchema,
  type ConnectorConfigMap,
  type ConnectorName,
  type ObsStreamSettings,
} from '$lib/schemas/connectors.js';

export function fetchConnectorConfig<K extends ConnectorName>(
  name: K,
): Promise<ConnectorConfigMap[K]> {
  return apiFetch(`/api/connectors/${name}/config`, ConnectorConfigSchemas[name]) as Promise<
    ConnectorConfigMap[K]
  >;
}

export function saveConnectorConfig<K extends ConnectorName>(
  name: K,
  config: ConnectorConfigMap[K],
): Promise<void> {
  return apiFetch(`/api/connectors/${name}/config`, z.void(), { method: 'PUT', body: config });
}

/**
 * Reads a connector's stored secrets. Only works on the machine hosting the
 * server: the admin token comes from Tauri IPC and the core additionally
 * requires the request to arrive on loopback.
 */
export async function revealConnectorSecrets<K extends ConnectorName>(
  name: K,
): Promise<ConnectorConfigMap[K]> {
  const adminToken = await getAdminToken();
  return apiFetch(`/api/connectors/${name}/config/secrets`, ConnectorConfigSchemas[name], {
    headers: { 'X-Admin-Token': adminToken },
  }) as Promise<ConnectorConfigMap[K]>;
}

export function fetchConnectorStatuses(): Promise<z.infer<typeof ConnectorStatusesSchema>> {
  return apiFetch('/api/connectors/status', ConnectorStatusesSchema);
}

export function connectObs(): Promise<void> {
  return apiFetch('/api/connectors/obs/connect', z.void(), { method: 'POST' });
}

export function disconnectObs(): Promise<void> {
  return apiFetch('/api/connectors/obs/disconnect', z.void(), { method: 'POST' });
}

export function fetchObsStreamSettings(): Promise<ObsStreamSettings> {
  return apiFetch('/api/connectors/obs/stream-settings', ObsStreamSettingsSchema);
}

export function applyObsStreamSettings(server: string, key: string): Promise<void> {
  return apiFetch('/api/connectors/obs/stream-settings', z.void(), {
    method: 'PUT',
    body: { server, key },
  });
}

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

const AuthUrlSchema = z.object({ url: z.string() });

export async function youtubeAuthUrl(): Promise<string> {
  return (await apiFetch('/api/auth/youtube/url', AuthUrlSchema)).url;
}

export async function facebookAuthUrl(): Promise<string> {
  return (await apiFetch('/api/auth/facebook/url', AuthUrlSchema)).url;
}

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
