import { get } from 'svelte/store';
import { serverPort, authToken } from '$lib/stores/server-url.js';
import { appMode } from '$lib/stores/mode.js';
import { serverUrl } from '$lib/stores/server-url.js';
import { wsStatus, lastWsMessage } from '$lib/stores/ws.js';
import { events, untrackedRecordings } from '$lib/stores/events.js';
import {
	obsStatus,
	obsState,
	vmixStatus,
	vmixState,
	atemStatus,
	atemState,
	youtubeStatus,
	youtubeState,
	facebookStatus,
	facebookState,
	discordStatus,
	discordState,
	broadlinkStatus,
	broadlinkState,
	youtubeLiveActive
} from '$lib/stores/connectors.js';
import { broadlinkDiscoveredDevices, broadlinkLearnResult } from '$lib/stores/broadlink.js';
import { keynoteStatus, pptResults, pptFolders } from '$lib/stores/presentations.js';
import { listFolders } from '$lib/api/presentations.js';
import { WsMessageSchema } from '$lib/schemas/ws-messages.js';
import type { EventSummary } from '$lib/schemas/event.js';
import { toast } from 'svelte-sonner';

let socket: WebSocket | null = null;
let reconnectTimer: ReturnType<typeof setTimeout> | null = null;

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
	wsStatus.set('disconnected');
}

function scheduleReconnect(): void {
	reconnectTimer = setTimeout(() => {
		connectWs();
	}, 3000);
}

function handleMessage(msg: ReturnType<typeof WsMessageSchema.parse>): void {
	if (msg.type === 'event.changed') {
		const { operation, record: rec } = msg.data;
		if (operation === 'DELETE') {
			events.update((list) => list.filter((e) => e.id !== rec.id));
		} else {
			events.update((list) => {
				const idx = list.findIndex((e) => e.id === rec.id);
				const existing = idx >= 0 ? list[idx] : undefined;
				const summary: EventSummary = {
					id: rec.id,
					title: rec.title,
					dateTime: rec.dateTime,
					speaker: rec.speaker,
					recordingCount: existing?.recordingCount ?? 0,
					isCompleted: existing?.isCompleted ?? false,
					createdAt: rec.createdAt,
					updatedAt: rec.updatedAt,
				};
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
		}
	} else if (msg.type === 'recording.detected') {
		if (msg.eventTitle !== null) {
			toast.success(`Recording added to ${msg.eventTitle}`);
		} else {
			toast.info('New untracked recording detected');
		}
	} else if (msg.type === 'recording.untracked.removed') {
		untrackedRecordings.update((list) => list.filter((r) => r.id !== msg.id));
	} else if (msg.type === 'obs.state') {
		obsState.update((s) => ({ ...s, isStreaming: msg.isStreaming, isRecording: msg.isRecording }));
	} else if (msg.type === 'connector.status') {
		const status = msg.status.type;
		if (msg.connector === 'obs') {
			obsStatus.set(status);
			const resetState = status === 'disconnected' || status === 'error';
			obsState.update((s) => ({
				...s,
				connection: status,
				isStreaming: resetState ? false : (s.isStreaming ?? false),
				isRecording: resetState ? false : (s.isRecording ?? false),
			}));
		} else if (msg.connector === 'vmix') {
			vmixStatus.set(status);
			vmixState.update((s) => ({ ...s, connection: status }));
		} else if (msg.connector === 'atem') {
			atemStatus.set(status);
			atemState.update((s) => ({ ...s, connection: status }));
		} else if (msg.connector === 'youtube') {
			youtubeStatus.set(status);
			youtubeState.update((s) => ({ ...s, connection: status }));
		} else if (msg.connector === 'facebook') {
			facebookStatus.set(status);
			facebookState.update((s) => ({ ...s, connection: status }));
		} else if (msg.connector === 'discord') {
			discordStatus.set(status);
			discordState.update((s) => ({ ...s, connection: status }));
		} else if (msg.connector === 'broadlink') {
			broadlinkStatus.set(status);
			broadlinkState.update((s) => ({ ...s, connection: status }));
		}
	} else if (msg.type === 'broadlink.device.discovered') {
		broadlinkDiscoveredDevices.update((list) => {
			const exists = list.some((d) => d.mac === msg.device.mac);
			return exists ? list : [...list, msg.device];
		});
	} else if (msg.type === 'broadlink.learn.result') {
		broadlinkLearnResult.set({ code: msg.code, error: msg.error });
	} else if (msg.type === 'cron.youtube_pull') {
		youtubeLiveActive.set(msg.hasLive);
		youtubeState.update((s) => ({ ...s, isLive: msg.hasLive }));
	} else if (msg.type === 'keynote.status') {
		keynoteStatus.set(msg.status);
	} else if (msg.type === 'ppt.search_results') {
		pptResults.set(msg.files);
	} else if (msg.type === 'ppt.folders_changed') {
		listFolders().then((folders) => pptFolders.set(folders));
	}
}

export function sendWsCommand(type: string, data?: Record<string, string | number | boolean | null>): boolean {
	if (!socket || socket.readyState !== WebSocket.OPEN) return false;
	socket.send(JSON.stringify({ type, ...data }));
	return true;
}
