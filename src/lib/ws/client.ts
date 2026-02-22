import { get } from 'svelte/store';
import { serverPort, authToken } from '$lib/stores/server-url.js';
import { appMode } from '$lib/stores/mode.js';
import { serverUrl } from '$lib/stores/server-url.js';
import { wsStatus, lastWsMessage } from '$lib/stores/ws.js';
import { events } from '$lib/stores/events.js';
import type { WsMessage } from '$lib/types/ws-messages.js';
import type { EventSummary } from '$lib/types/event.js';

let socket: WebSocket | null = null;
let reconnectTimer: ReturnType<typeof setTimeout> | null = null;

export function connectWs(): void {
  if (socket && socket.readyState === WebSocket.OPEN) return;

  const mode = get(appMode);
  const token = get(authToken);

  let wsUrl: string;
  if (mode === 'server') {
    const port = get(serverPort);
    wsUrl = `ws://localhost:${port}/ws?token=${encodeURIComponent(token)}`;
  } else {
    const base = get(serverUrl).replace(/^http/, 'ws');
    wsUrl = `${base}/ws?token=${encodeURIComponent(token)}`;
  }

  wsStatus.set('connecting');
  socket = new WebSocket(wsUrl);

  socket.addEventListener('open', () => {
    wsStatus.set('connected');
    if (reconnectTimer !== null) {
      clearTimeout(reconnectTimer);
      reconnectTimer = null;
    }
  });

  socket.addEventListener('message', (ev) => {
    try {
      const msg = JSON.parse(ev.data as string) as WsMessage;
      lastWsMessage.set(msg);
      handleMessage(msg);
    } catch {
      // ignore unparseable messages
    }
  });

  socket.addEventListener('close', () => {
    wsStatus.set('disconnected');
    socket = null;
    scheduleReconnect();
  });

  socket.addEventListener('error', () => {
    wsStatus.set('error');
    socket?.close();
  });
}

export function disconnectWs(): void {
  if (reconnectTimer !== null) {
    clearTimeout(reconnectTimer);
    reconnectTimer = null;
  }
  socket?.close();
  socket = null;
  wsStatus.set('disconnected');
}

function scheduleReconnect(): void {
  reconnectTimer = setTimeout(() => {
    connectWs();
  }, 3000);
}

function handleMessage(msg: WsMessage): void {
  if (msg.type === 'event.changed') {
    const rec = msg.data.record;
    const summary: EventSummary = {
      id: rec.id,
      title: rec.title,
      dateTime: rec.dateTime,
      speaker: rec.speaker,
      recordingCount: 0,
      createdAt: rec.createdAt,
      updatedAt: rec.updatedAt,
    };
    events.update((list) => {
      const idx = list.findIndex((e) => e.id === summary.id);
      if (idx >= 0) {
        const updated = [...list];
        updated[idx] = { ...updated[idx], ...summary };
        return updated;
      }
      return [summary, ...list];
    });
  }
}
