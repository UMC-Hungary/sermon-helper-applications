import type { Event } from './event.js';
import type { Recording } from './recording.js';

export type WsMessage =
  | { type: 'connected'; serverId: string }
  | { type: 'event.changed'; data: { operation: string; record: Event } }
  | { type: 'recording.changed'; data: { operation: string; record: Recording } };
