import { z } from 'zod';

export const RecordingSchema = z.object({
  id: z.string().uuid(),
  eventId: z.string().uuid(),
  filePath: z.string(),
  fileName: z.string(),
  fileSize: z.number().int().nonnegative(),
  durationSeconds: z.number().nonnegative(),
  detectedAt: z.string(),
  whitelisted: z.boolean(),
  uploaded: z.boolean(),
  uploadedAt: z.string().optional(),
  videoId: z.string().optional(),
  videoUrl: z.string().optional(),
  customTitle: z.string().optional(),
  createdAt: z.string(),
  updatedAt: z.string(),
});

export type Recording = z.infer<typeof RecordingSchema>;

export const CreateRecordingPayloadSchema = z.object({
  file_path: z.string().min(1),
  file_name: z.string().min(1),
  file_size: z.number().int().nonnegative().optional(),
  duration_seconds: z.number().nonnegative().optional(),
  custom_title: z.string().optional(),
});

export type CreateRecordingPayload = z.infer<typeof CreateRecordingPayloadSchema>;
