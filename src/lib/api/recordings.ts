import { z } from 'zod';
import { apiFetch } from './client.js';
import {
  RecordingSchema,
  type Recording,
  type CreateRecordingPayload,
} from '$lib/schemas/recording.js';

export function listRecordings(eventId: string): Promise<Recording[]> {
  return apiFetch(`/api/events/${eventId}/recordings`, z.array(RecordingSchema));
}

export function createRecording(
  eventId: string,
  payload: CreateRecordingPayload,
): Promise<Recording> {
  return apiFetch(`/api/events/${eventId}/recordings`, RecordingSchema, {
    method: 'POST',
    body: payload,
  });
}
