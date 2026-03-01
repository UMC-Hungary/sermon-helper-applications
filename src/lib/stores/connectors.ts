import { writable } from 'svelte/store';
import type { ConnectorState } from '$lib/connectors/types.js';

export type ConnectorStatus = 'disconnected' | 'connecting' | 'connected' | 'error';

export interface ObsConfig {
	enabled: boolean;
	host: string;
	port: number;
	password: string | null;
}

export interface VmixConfig {
	enabled: boolean;
	host: string;
	port: number;
}

export interface AtemConfig {
	enabled: boolean;
	host: string;
	port: number;
}

export interface YouTubeConfig {
	enabled: boolean;
	clientId: string;
	clientSecret: string;
}

export interface FacebookConfig {
	enabled: boolean;
	appId: string;
	appSecret: string;
	pageId: string;
}

export interface DiscordConfig {
	enabled: boolean;
	webhookUrl: string;
}

/** Payload shape emitted by the Rust backend. */
interface ConnectorStatusPayload {
	type: ConnectorStatus;
	message?: string;
}

/** Map the tagged-union payload from Rust to the simple status string. */
export function mapConnectorStatus(payload: ConnectorStatusPayload): ConnectorStatus {
	return payload.type;
}

// ── Status stores (source of truth for ConnectorStatusBadge) ─────────────────

export const obsStatus = writable<ConnectorStatus>('disconnected');
export const vmixStatus = writable<ConnectorStatus>('disconnected');
export const atemStatus = writable<ConnectorStatus>('disconnected');
export const youtubeStatus = writable<ConnectorStatus>('disconnected');
export const facebookStatus = writable<ConnectorStatus>('disconnected');
export const discordStatus = writable<ConnectorStatus>('disconnected');

// ── Config stores ─────────────────────────────────────────────────────────────

export const obsConfig = writable<ObsConfig>({
	enabled: false,
	host: 'localhost',
	port: 4455,
	password: null
});

export const vmixConfig = writable<VmixConfig>({
	enabled: false,
	host: 'localhost',
	port: 8088
});

export const atemConfig = writable<AtemConfig>({
	enabled: false,
	host: '',
	port: 9910
});

export const youtubeConfig = writable<YouTubeConfig>({
	enabled: false,
	clientId: '',
	clientSecret: ''
});

export const facebookConfig = writable<FacebookConfig>({
	enabled: false,
	appId: '',
	appSecret: '',
	pageId: ''
});

export const discordConfig = writable<DiscordConfig>({
	enabled: false,
	webhookUrl: ''
});

// ── Extended state stores (status + boolean capability flags) ─────────────────
// These are updated alongside the status stores. The boolean flags (isStreaming,
// isRecording, isLive) will be populated when WS messages for them arrive.

export const obsState = writable<ConnectorState>({ connection: 'disconnected' });
export const vmixState = writable<ConnectorState>({ connection: 'disconnected' });
export const atemState = writable<ConnectorState>({ connection: 'disconnected' });
export const youtubeState = writable<ConnectorState>({ connection: 'disconnected' });
export const facebookState = writable<ConnectorState>({ connection: 'disconnected' });
export const discordState = writable<ConnectorState>({ connection: 'disconnected' });

/** True when the cron.youtube_pull message reports at least one live broadcast. */
export const youtubeLiveActive = writable<boolean>(false);
