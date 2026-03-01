import { z } from 'zod';
import { apiFetch } from './client.js';
import {
  EventSchema,
  EventSummarySchema,
  type Event,
  type EventSummary,
  type CreateEventPayload,
  type UpdateEventPayload,
} from '$lib/schemas/event.js';

export function listEvents(): Promise<EventSummary[]> {
  return apiFetch('/api/events', z.array(EventSummarySchema));
}

export function getEvent(id: string): Promise<Event> {
  return apiFetch(`/api/events/${id}`, EventSchema);
}

export function createEvent(payload: CreateEventPayload): Promise<Event> {
  return apiFetch('/api/events', EventSchema, { method: 'POST', body: payload });
}

export function updateEvent(id: string, payload: UpdateEventPayload): Promise<Event> {
  return apiFetch(`/api/events/${id}`, EventSchema, { method: 'PUT', body: payload });
}
