import { z } from 'zod';
import { EventSchema, EventSummarySchema, EventActivitySchema } from './event.js';
import { RecordingSchema, RecordingWithEventSchema } from './recording.js';
import { UntrackedRecordingSchema } from './untracked-recording.js';

const ConnectorStatusPayloadSchema = z.object({
  type: z.enum(['disconnected', 'connecting', 'connected', 'error']),
  message: z.string().optional(),
});

export const KeynoteStatusSchema = z.object({
  appRunning: z.boolean(),
  slideshowActive: z.boolean(),
  currentSlide: z.number().nullable(),
  totalSlides: z.number().nullable(),
  documentName: z.string().nullable(),
});

export const PptFileSchema = z.object({
  id: z.string(),
  name: z.string(),
  path: z.string(),
  folderId: z.string(),
});

export const PptFolderSchema = z.object({
  id: z.string().uuid(),
  path: z.string(),
  name: z.string(),
  sortOrder: z.number().int(),
});

export const CronJobSchema = z.object({
  id: z.string().uuid(),
  name: z.string(),
  cronExpression: z.string(),
  enabled: z.boolean(),
  pullYoutube: z.boolean(),
  autoUpload: z.boolean(),
  createdAt: z.string(),
  updatedAt: z.string(),
});

export const StreamStatsSchema = z.object({
  ready: z.boolean(),
  bytesReceived: z.number().int().nonnegative(),
  bytesSent: z.number().int().nonnegative(),
  readers: z.number().int().nonnegative(),
  tracks: z.array(z.string()),
});

export const BroadlinkDeviceSchema = z.object({
  id: z.string().uuid(),
  name: z.string(),
  deviceType: z.string(),
  model: z.string().nullable(),
  host: z.string(),
  mac: z.string(),
  isDefault: z.boolean(),
});

export const BroadlinkCommandSchema = z.object({
  id: z.string().uuid(),
  deviceId: z.string().uuid().nullable(),
  name: z.string(),
  slug: z.string(),
  code: z.string(),
  codeType: z.string(),
  category: z.string(),
});

export type KeynoteStatus = z.infer<typeof KeynoteStatusSchema>;
export type PptFile = z.infer<typeof PptFileSchema>;
export type PptFolder = z.infer<typeof PptFolderSchema>;
export type CronJob = z.infer<typeof CronJobSchema>;
export type StreamStats = z.infer<typeof StreamStatsSchema>;
export type BroadlinkDevice = z.infer<typeof BroadlinkDeviceSchema>;
export type BroadlinkCommand = z.infer<typeof BroadlinkCommandSchema>;

export const WsMessageSchema = z.discriminatedUnion('type', [
  // ── Core ───────────────────────────────────────────────────────────────────
  z.object({ type: z.literal('connected'), serverId: z.string() }),
  z.object({ type: z.literal('ok') }),
  z.object({ type: z.literal('error'), message: z.string() }),
  // ── Connector push (server → client) ───────────────────────────────────────
  z.object({
    type: z.literal('event.changed'),
    data: z.object({ operation: z.string(), record: EventSchema }),
  }),
  z.object({
    type: z.literal('recording.changed'),
    data: z.object({ operation: z.string(), record: RecordingSchema }),
  }),
  z.object({
    type: z.literal('connector.status'),
    connector: z.enum(['obs', 'vmix', 'atem', 'youtube', 'facebook', 'discord', 'broadlink']),
    status: ConnectorStatusPayloadSchema,
  }),
  z.object({
    type: z.literal('connector.state'),
    connector: z.enum(['obs', 'vmix', 'atem', 'broadlink', 'youtube', 'facebook', 'discord']),
    isStreaming: z.boolean().optional(),
    isRecording: z.boolean().optional(),
  }),
  z.object({
    type: z.literal('cron.youtube_pull'),
    hasLive: z.boolean(),
  }),
  z.object({
    type: z.literal('recording.detected'),
    fileName: z.string(),
    eventTitle: z.string().nullable(),
  }),
  z.object({
    type: z.literal('recording.untracked.removed'),
    id: z.string().uuid(),
  }),
  z.object({
    type: z.literal('obs.state'),
    isStreaming: z.boolean(),
    isRecording: z.boolean(),
  }),
  z.object({
    type: z.literal('broadlink.device.discovered'),
    device: z.object({
      name: z.string(),
      host: z.string(),
      mac: z.string(),
      deviceType: z.string(),
      model: z.string().nullable(),
    }),
  }),
  z.object({
    type: z.literal('broadlink.learn.result'),
    code: z.string().nullable(),
    error: z.string().nullable(),
  }),
  z.object({
    type: z.literal('keynote.status'),
    status: KeynoteStatusSchema,
  }),
  z.object({
    type: z.literal('ppt.search_results'),
    files: z.array(PptFileSchema),
  }),
  z.object({
    type: z.literal('ppt.folders_changed'),
  }),
  z.object({
    type: z.literal('upload.progress'),
    recordingId: z.string().uuid(),
    platform: z.string(),
    progressBytes: z.number().int(),
    totalBytes: z.number().int(),
  }),
  z.object({
    type: z.literal('upload.completed'),
    recordingId: z.string().uuid(),
    platform: z.string(),
    videoId: z.string(),
    videoUrl: z.string(),
  }),
  z.object({
    type: z.literal('upload.failed'),
    recordingId: z.string().uuid(),
    platform: z.string(),
    error: z.string(),
  }),
  z.object({
    type: z.literal('upload.paused'),
    recordingId: z.string().uuid(),
    reason: z.string(),
  }),
  // ── Events (WS command responses) ──────────────────────────────────────────
  z.object({ type: z.literal('events.list'), events: z.array(EventSummarySchema) }),
  z.object({ type: z.literal('events.get'), event: EventSchema }),
  z.object({ type: z.literal('events.create'), event: EventSchema }),
  z.object({ type: z.literal('events.update'), event: EventSchema }),
  // ── Recordings (WS command responses) ──────────────────────────────────────
  z.object({ type: z.literal('recordings.list'), recordings: z.array(RecordingSchema) }),
  z.object({ type: z.literal('recordings.list_all'), recordings: z.array(RecordingWithEventSchema) }),
  z.object({ type: z.literal('recordings.create'), recording: RecordingSchema }),
  z.object({ type: z.literal('recordings.untracked.list'), recordings: z.array(UntrackedRecordingSchema) }),
  z.object({ type: z.literal('recordings.untracked.assign'), recording: RecordingSchema }),
  // ── Activities (WS command responses) ──────────────────────────────────────
  z.object({ type: z.literal('activities.list'), activities: z.array(EventActivitySchema) }),
  z.object({ type: z.literal('activities.create'), activity: EventActivitySchema }),
  // ── Cron jobs (WS command responses) ───────────────────────────────────────
  z.object({ type: z.literal('cron_jobs.list'), jobs: z.array(CronJobSchema) }),
  z.object({ type: z.literal('cron_jobs.create'), job: CronJobSchema }),
  z.object({ type: z.literal('cron_jobs.update'), job: CronJobSchema }),
  // ── PPT (WS command responses) ─────────────────────────────────────────────
  z.object({ type: z.literal('ppt.folders.list'), folders: z.array(PptFolderSchema) }),
  z.object({ type: z.literal('ppt.folders.add'), folder: PptFolderSchema.nullable() }),
  // ── Connectors (WS command responses) ──────────────────────────────────────
  z.object({
    type: z.literal('connectors.status'),
    obs: ConnectorStatusPayloadSchema,
    vmix: ConnectorStatusPayloadSchema,
    youtube: ConnectorStatusPayloadSchema,
    facebook: ConnectorStatusPayloadSchema,
  }),
  z.object({
    type: z.literal('connectors.state'),
    obs: z.object({ isStreaming: z.boolean(), isRecording: z.boolean() }).nullable(),
  }),
  z.object({ type: z.literal('connectors.youtube.stream_key'), rtmpUrl: z.string() }),
  z.object({ type: z.literal('connectors.facebook.stream_key'), rtmpUrl: z.string() }),
  z.object({
    type: z.literal('connectors.youtube.content'),
    content: z.object({
      liveBroadcasts: z.array(z.object({
        id: z.string(),
        title: z.string(),
        thumbnailUrl: z.string(),
        publishedAt: z.string().nullable(),
        viewCount: z.number().nonnegative().nullable(),
        likeCount: z.number().nonnegative().nullable(),
        duration: z.string().nullable(),
        liveStatus: z.string(),
        scheduledStartTime: z.string().nullable(),
        watchUrl: z.string(),
        privacyStatus: z.string(),
      })),
      videos: z.array(z.object({
        id: z.string(),
        title: z.string(),
        thumbnailUrl: z.string(),
        publishedAt: z.string().nullable(),
        viewCount: z.number().nonnegative().nullable(),
        likeCount: z.number().nonnegative().nullable(),
        duration: z.string().nullable(),
        liveStatus: z.string(),
        scheduledStartTime: z.string().nullable(),
        watchUrl: z.string(),
        privacyStatus: z.string(),
      })),
    }),
  }),
  // ── Auth (WS command responses) ────────────────────────────────────────────
  z.object({ type: z.literal('auth.youtube.url'), url: z.string() }),
  z.object({ type: z.literal('auth.facebook.url'), url: z.string() }),
  // ── Stream (WS command responses) ──────────────────────────────────────────
  z.object({ type: z.literal('stream.stats'), stats: StreamStatsSchema }),
  // ── Broadlink (WS command responses) ───────────────────────────────────────
  z.object({ type: z.literal('broadlink.status'), status: ConnectorStatusPayloadSchema }),
  z.object({ type: z.literal('broadlink.devices.list'), devices: z.array(BroadlinkDeviceSchema) }),
  z.object({ type: z.literal('broadlink.devices.add'), device: BroadlinkDeviceSchema }),
  z.object({ type: z.literal('broadlink.commands.list'), commands: z.array(BroadlinkCommandSchema) }),
  z.object({ type: z.literal('broadlink.commands.add'), command: BroadlinkCommandSchema }),
  z.object({ type: z.literal('broadlink.commands.update'), command: BroadlinkCommandSchema }),
]);

export type WsMessage = z.infer<typeof WsMessageSchema>;
