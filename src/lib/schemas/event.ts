import { z } from 'zod';

export const EventConnectionSchema = z.object({
  platform: z.string(),
  externalId: z.string().nullable().default(null),
  streamUrl: z.string().nullable().default(null),
  eventUrl: z.string().nullable().default(null),
  scheduleStatus: z.string().default('not_scheduled'),
  privacyStatus: z.string().nullable().default(null),
  extra: z.record(z.string(), z.unknown()).nullable().default(null),
});

export type EventConnection = z.infer<typeof EventConnectionSchema>;

export const EventSchema = z.object({
  id: z.string().uuid(),
  title: z.string(),
  dateTime: z.string(),
  speaker: z.string(),
  description: z.string(),
  textus: z.string(),
  leckio: z.string(),
  textusTranslation: z.string(),
  leckioTranslation: z.string(),
  autoUploadEnabled: z.boolean(),
  connections: z.array(EventConnectionSchema).default([]),
  createdAt: z.string(),
  updatedAt: z.string(),
});

export type Event = z.infer<typeof EventSchema>;

export const EventSummarySchema = z.object({
  id: z.string().uuid(),
  title: z.string(),
  dateTime: z.string(),
  speaker: z.string(),
  recordingCount: z.number().int().nonnegative(),
  createdAt: z.string(),
  updatedAt: z.string(),
});

export type EventSummary = z.infer<typeof EventSummarySchema>;

export const CreateEventPayloadSchema = z.object({
  title: z.string().min(1),
  date_time: z.string(),
  speaker: z.string().optional(),
  description: z.string().optional(),
  textus: z.string().optional(),
  leckio: z.string().optional(),
  textus_translation: z.string().optional(),
  leckio_translation: z.string().optional(),
  auto_upload_enabled: z.boolean().optional(),
  connections: z
    .array(z.object({ platform: z.string(), privacy_status: z.string().optional() }))
    .optional(),
});

export type CreateEventPayload = z.infer<typeof CreateEventPayloadSchema>;

export const UpdateEventPayloadSchema = CreateEventPayloadSchema;
export type UpdateEventPayload = CreateEventPayload;
