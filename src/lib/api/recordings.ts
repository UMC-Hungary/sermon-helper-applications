import { apiFetch } from './client.js';
import type { Recording, CreateRecordingPayload } from '$lib/types/recording.js';

export function listRecordings(eventId: string): Promise<Recording[]> {
  return apiFetch<Recording[]>(`/api/events/${eventId}/recordings`);
}

export function createRecording(eventId: string, payload: CreateRecordingPayload): Promise<Recording> {
  return apiFetch<Recording>(`/api/events/${eventId}/recordings`, {
    method: 'POST',
    body: JSON.stringify(payload),
  });
}
