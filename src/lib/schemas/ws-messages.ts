import { z } from 'zod';
import { EventSchema } from './event.js';
import { RecordingSchema } from './recording.js';

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

export type KeynoteStatus = z.infer<typeof KeynoteStatusSchema>;
export type PptFile = z.infer<typeof PptFileSchema>;

export const WsMessageSchema = z.discriminatedUnion('type', [
  z.object({ type: z.literal('connected'), serverId: z.string() }),
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
]);

export type WsMessage = z.infer<typeof WsMessageSchema>;
