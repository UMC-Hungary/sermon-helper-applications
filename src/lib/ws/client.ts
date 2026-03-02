import { get } from 'svelte/store';
import { serverPort, authToken } from '$lib/stores/server-url.js';
import { appMode } from '$lib/stores/mode.js';
import { serverUrl } from '$lib/stores/server-url.js';
import { wsStatus, lastWsMessage } from '$lib/stores/ws.js';
import { events } from '$lib/stores/events.js';
import {
	obsStatus,
	obsState,
	vmixStatus,
	vmixState,
	atemStatus,
	atemState,
	broadlinkStatus,
	broadlinkState,
	youtubeStatus,
	youtubeState,
	facebookStatus,
	facebookState,
	discordStatus,
	discordState,
	youtubeLiveActive
} from '$lib/stores/connectors.js';
import { WsMessageSchema } from '$lib/schemas/ws-messages.js';
import type { EventSummary } from '$lib/schemas/event.js';
import { toast } from 'svelte-sonner';

let socket: WebSocket | null = null;
let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
let reconnectDelay = 3000;
const RECONNECT_DELAY_MIN = 3000;
const RECONNECT_DELAY_MAX = 30000;

export function connectWs(): void {
	if (socket && (socket.readyState === WebSocket.OPEN || socket.readyState === WebSocket.CONNECTING))
		return;

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
		reconnectDelay = RECONNECT_DELAY_MIN;
		if (reconnectTimer !== null) {
			clearTimeout(reconnectTimer);
			reconnectTimer = null;
		}
	});

	socket.addEventListener('message', (ev) => {
		const result = WsMessageSchema.safeParse(JSON.parse(ev.data as string));
		if (!result.success) return;
		const msg = result.data;
		lastWsMessage.set(msg);
		handleMessage(msg);
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
	reconnectDelay = RECONNECT_DELAY_MIN;
	wsStatus.set('disconnected');
}

function scheduleReconnect(): void {
	reconnectTimer = setTimeout(() => {
		connectWs();
	}, reconnectDelay);
	reconnectDelay = Math.min(reconnectDelay * 2, RECONNECT_DELAY_MAX);
}

function handleMessage(msg: ReturnType<typeof WsMessageSchema.parse>): void {
	if (msg.type === 'event.changed') {
		const { operation, record: rec } = msg.data;
		const summary: EventSummary = {
			id: rec.id,
			title: rec.title,
			dateTime: rec.dateTime,
			speaker: rec.speaker,
			recordingCount: 0,
			createdAt: rec.createdAt,
			updatedAt: rec.updatedAt
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
		if (operation === 'INSERT') {
			toast.success(`New event created: ${rec.title}`);
		} else if (operation === 'UPDATE') {
			toast.info(`Event updated: ${rec.title}`);
		}
	} else if (msg.type === 'connector.status') {
		const status = msg.status.type;
		if (msg.connector === 'obs') {
			obsStatus.set(status);
			obsState.update((s) => ({ ...s, connection: status }));
		} else if (msg.connector === 'vmix') {
			vmixStatus.set(status);
			vmixState.update((s) => ({ ...s, connection: status }));
		} else if (msg.connector === 'atem') {
			atemStatus.set(status);
			atemState.update((s) => ({ ...s, connection: status }));
		} else if (msg.connector === 'broadlink') {
			broadlinkStatus.set(status);
			broadlinkState.update((s) => ({ ...s, connection: status }));
		} else if (msg.connector === 'youtube') {
			youtubeStatus.set(status);
			youtubeState.update((s) => ({ ...s, connection: status }));
		} else if (msg.connector === 'facebook') {
			facebookStatus.set(status);
			facebookState.update((s) => ({ ...s, connection: status }));
		} else if (msg.connector === 'discord') {
			discordStatus.set(status);
			discordState.update((s) => ({ ...s, connection: status }));
		}
	} else if (msg.type === 'connector.state') {
		const patch: { isStreaming?: boolean; isRecording?: boolean } = {};
		if (msg.isStreaming !== undefined) patch.isStreaming = msg.isStreaming;
		if (msg.isRecording !== undefined) patch.isRecording = msg.isRecording;
		if (msg.connector === 'obs') {
			obsState.update((s) => ({ ...s, ...patch }));
		} else if (msg.connector === 'vmix') {
			vmixState.update((s) => ({ ...s, ...patch }));
		} else if (msg.connector === 'broadlink') {
			broadlinkState.update((s) => ({ ...s, ...patch }));
		}
	} else if (msg.type === 'cron.youtube_pull') {
		youtubeLiveActive.set(msg.hasLive);
		youtubeState.update((s) => ({ ...s, isLive: msg.hasLive }));
	}
}
