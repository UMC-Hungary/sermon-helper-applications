import { goto } from '$app/navigation';
import { sendWsCommand, connectObs, youtubeAuthUrl, openExternal } from '@metocast/core-client';
import type { WsMessage } from '@metocast/core-client';
import type { PresenterState, KeynoteStatus, WsClientInfo, PptFile } from '@metocast/core-client/schemas/ws-messages';
import { siObsstudio, siYoutube, siFacebook, siBlackmagicdesign, siDiscord } from 'simple-icons';
import { notify, resolveByKey } from './notifications.svelte';

/** Display name, kind label and brand mark per connector — matches the reference spec. */
const CONNECTOR_META: Record<string, { name: string; kind: string; brand?: string }> = {
  obs: { name: 'OBS Studio', kind: 'Encoder', brand: siObsstudio.path },
  youtube: { name: 'YouTube', kind: 'Streaming', brand: siYoutube.path },
  facebook: { name: 'Facebook Live', kind: 'Streaming', brand: siFacebook.path },
  atem: { name: 'Blackmagic ATEM', kind: 'Switcher', brand: siBlackmagicdesign.path },
  discord: { name: 'Discord', kind: 'Webhooks', brand: siDiscord.path },
};

// Sanctum's realtime state, in runes. The transport (core-client) validates every
// message; this module is the only place that decides which piece of UI state each
// one updates — the same role classic's ws-bindings plays for its stores.

type ConnState = 'disconnected' | 'connecting' | 'connected' | 'error';

let obsStreaming = $state(false);
let obsRecording = $state(false);
let connectorStatus = $state<Record<string, ConnState>>({});
let presenter = $state<PresenterState | null>(null);
let keynote = $state<KeynoteStatus | null>(null);
let clients = $state<WsClientInfo[]>([]);
let pptResults = $state<PptFile[]>([]);
let useWebPresenter = $state(true);

export const live = {
  get streaming() {
    return obsStreaming;
  },
  get recording() {
    return obsRecording;
  },
  get connectorStatus() {
    return connectorStatus;
  },
  get presenter() {
    return presenter;
  },
  get keynote() {
    return keynote;
  },
  get clients() {
    return clients;
  },
  get pptResults() {
    return pptResults;
  },
  get useWebPresenter() {
    return useWebPresenter;
  },
};

/** Names a connector for a notification, without inventing prose the core didn't send. */
function sourceName(id: string): string {
  return id.charAt(0).toUpperCase() + id.slice(1);
}

export function handleWs(msg: WsMessage): void {
  switch (msg.type) {
    case 'obs.state':
      obsStreaming = msg.isStreaming;
      obsRecording = msg.isRecording;
      break;
    case 'connector.state':
      if (msg.connector === 'obs') {
        if (msg.isStreaming !== undefined) obsStreaming = msg.isStreaming;
        if (msg.isRecording !== undefined) obsRecording = msg.isRecording;
      }
      break;
    case 'connector.status': {
      const prev = connectorStatus[msg.connector];
      const next = msg.status.type;
      connectorStatus = { ...connectorStatus, [msg.connector]: next };
      const key = `connector:${msg.connector}`;
      const meta = CONNECTOR_META[msg.connector] ?? { name: sourceName(msg.connector), kind: 'Connector' };
      const actions = msg.connector === 'obs'
        ? [
            { label: 'Reconnect', primary: true, run: () => void connectObs() },
            { label: 'Edit', run: () => void goto('/settings/connectors') },
          ]
        : msg.connector === 'youtube'
          ? [
              { label: 'Re-login', primary: true, run: () => void youtubeAuthUrl().then(openExternal) },
              { label: 'Edit', run: () => void goto('/settings/connectors?open=youtube') },
            ]
          : undefined;
      if (next === 'error') {
        notify({
          tier: 'error',
          kind: meta.kind,
          source: meta.name,
          title: `${meta.name} disconnected`,
          body: msg.status.message,
          state: 'error',
          brand: meta.brand,
          actions,
          remediation: ['Open Settings → Connectors', `Check the ${meta.name} configuration`, 'Re-enable the connector'],
          key,
        });
      } else if (next === 'connecting' && prev === 'error') {
        notify({
          tier: 'warn',
          kind: meta.kind,
          source: meta.name,
          title: `${meta.name} disconnected`,
          body: msg.status.message,
          state: 'reconnecting',
          brand: meta.brand,
          actions,
          remediation: ['Open Settings → Connectors', `Check the ${meta.name} configuration`, 'Re-enable the connector'],
          key,
        });
      } else if (next === 'connected' && (prev === 'error' || prev === 'connecting')) {
        resolveByKey(key, {
          kind: meta.kind,
          source: meta.name,
          title: `${meta.name} reconnected`,
        });
      }
      break;
    }
    case 'notification':
      notify({
        tier: msg.level === 'error' ? 'error' : msg.level === 'warn' ? 'warn' : 'ok',
        kind: 'System',
        source: 'Core',
        title: msg.message,
      });
      break;
    case 'error':
      notify({ tier: 'error', kind: 'System', source: 'Core', title: msg.message });
      break;
    case 'presentation.settings':
      useWebPresenter = msg.useWebPresenter;
      break;
    case 'presenter.state':
      presenter = msg.state;
      break;
    case 'presenter.slide_changed':
      if (presenter) presenter = { ...presenter, currentSlide: msg.currentSlide, totalSlides: msg.totalSlides };
      break;
    case 'keynote.status':
      keynote = msg.status;
      break;
    case 'clients.updated':
    case 'clients.list':
      clients = msg.clients;
      break;
    case 'ppt.search_results':
      pptResults = msg.files;
      break;
    case 'ping':
      sendWsCommand('pong', { ping_id: msg.pingId });
      break;
  }
}

/** Called once the socket opens: identify this window and pull current realtime state. */
export function registerLive(): void {
  sendWsCommand('presenter.register', { label: 'Sanctum' });
  sendWsCommand('clients.list');
}
