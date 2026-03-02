import { z } from 'zod';
import { EventSchema } from './event.js';
import { RecordingSchema } from './recording.js';

const ConnectorStatusPayloadSchema = z.object({
  type: z.enum(['disconnected', 'connecting', 'connected', 'error']),
  message: z.string().optional(),
});

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
    connector: z.enum(['obs', 'vmix', 'atem', 'broadlink', 'youtube', 'facebook', 'discord']),
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
]);

export type WsMessage = z.infer<typeof WsMessageSchema>;
