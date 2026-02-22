import { apiFetch } from './client.js';
import type { Event, EventSummary, CreateEventPayload } from '$lib/types/event.js';

export function listEvents(): Promise<EventSummary[]> {
  return apiFetch<EventSummary[]>('/api/events');
}

export function getEvent(id: string): Promise<Event> {
  return apiFetch<Event>(`/api/events/${id}`);
}

export function createEvent(payload: CreateEventPayload): Promise<Event> {
  return apiFetch<Event>('/api/events', {
    method: 'POST',
    body: JSON.stringify(payload),
  });
}
