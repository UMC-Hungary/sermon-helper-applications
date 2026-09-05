import { goto } from '$app/navigation';
import { sendWsCommand, connectObs, connectCamera, youtubeAuthUrl, openExternal } from '@metocast/core-client';
import type { WsMessage } from '@metocast/core-client';
import type { PresenterState, KeynoteStatus, WsClientInfo, PptFile } from '@metocast/core-client/schemas/ws-messages';
import { siObsstudio, siYoutube, siFacebook, siBlackmagicdesign, siDiscord } from 'simple-icons';
import { notify, resolveByKey } from './notifications.svelte';

/** Display name, kind label and brand mark per connector — matches the reference spec. */
const CONNECTOR_META: Record<string, { name: string; kind: string; brand?: string }> = {
  obs: { name: 'OBS Studio', kind: 'Encoder', brand: siObsstudio.path },
  youtube: { name: 'YouTube', kind: 'Streaming', brand: siYoutube.path },
  facebook: { name: 'Facebook Live', kind: 'Streaming', brand: siFacebook.path },
  'blackmagic-camera': { name: 'Blackmagic Camera', kind: 'Camera', brand: siBlackmagicdesign.path },
  atem: { name: 'Blackmagic ATEM', kind: 'Switcher', brand: siBlackmagicdesign.path },
  discord: { name: 'Discord', kind: 'Webhooks', brand: siDiscord.path },
};

const REMEDIATION = (name: string) => [
  'Open Settings → Connectors',
  `Check the ${name} configuration`,
  'Re-enable the connector',
];

// Sanctum's realtime state, in runes. The transport (core-client) validates every
// message; this module is the only place that decides which piece of UI state each
// one updates — the same role classic's ws-bindings plays for its stores.

type ConnState = 'disconnected' | 'connecting' | 'connected' | 'error';

let obsStreaming = $state(false);
let obsRecording = $state(false);
let connectorStatus = $state<Record<string, ConnState>>({});
let cameraStreaming = $state(false);
let cameraRecording = $state(false);
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
  get cameraStreaming() {
    return cameraStreaming;
  },
  get cameraRecording() {
    return cameraRecording;
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
      } else if (msg.connector === 'blackmagic-camera') {
        if (msg.isStreaming !== undefined) cameraStreaming = msg.isStreaming;
        if (msg.isRecording !== undefined) cameraRecording = msg.isRecording;
      }
      break;
    case 'connectors.state':
      obsStreaming = msg.obs?.isStreaming ?? false;
      obsRecording = msg.obs?.isRecording ?? false;
      cameraStreaming = msg['blackmagic-camera']?.isStreaming ?? false;
      cameraRecording = msg['blackmagic-camera']?.isRecording ?? false;
      break;
    case 'connectors.status':
      connectorStatus = {
        obs: msg.obs.type,
        vmix: msg.vmix.type,
        broadlink: msg.broadlink.type,
        youtube: msg.youtube.type,
        facebook: msg.facebook.type,
        'blackmagic-camera': msg['blackmagic-camera'].type,
      };
      break;
    case 'connector.status': {
      const prev = connectorStatus[msg.connector];
      const next = msg.status.type;
      connectorStatus = { ...connectorStatus, [msg.connector]: next };
      const key = `connector:${msg.connector}`;
      const meta = CONNECTOR_META[msg.connector] ?? { name: sourceName(msg.connector), kind: 'Connector' };
      // Navigation is instant, so it stays a plain action — only work that takes
      // time (a reconnect, a re-login) returns its promise and animates.
      const edit = (query = '') => ({
        label: 'Edit',
        run: () => void goto(`/settings/connectors${query}`),
      });
      const editFor =
        msg.connector === 'obs'
          ? edit()
          : msg.connector === 'blackmagic-camera'
            ? edit('?open=blackmagic-camera')
            : msg.connector === 'youtube'
              ? edit('?open=youtube')
              : null;
      // The backend restart only reports a status edge the card can already be sitting
      // on, so the click paints the reconnecting chip itself rather than waiting for one.
      const showReconnecting = (body?: string) =>
        notify({
          tier: 'warn',
          kind: meta.kind,
          source: meta.name,
          title: `${meta.name} disconnected`,
          body,
          state: 'reconnecting',
          brand: meta.brand,
          actions: editFor ? [editFor] : undefined,
          remediation: REMEDIATION(meta.name),
          key,
        });
      const reconnect = (start: () => Promise<unknown>) => () => {
        showReconnecting();
        return start();
      };
      const retry = msg.connector === 'obs'
        ? { label: 'Reconnect', primary: true, run: reconnect(connectObs) }
        : msg.connector === 'blackmagic-camera'
          ? { label: 'Reconnect', primary: true, run: reconnect(connectCamera) }
          : msg.connector === 'youtube'
            ? { label: 'Re-login', primary: true, run: () => youtubeAuthUrl().then(openExternal) }
            : null;
      const actions = retry && editFor ? [retry, editFor] : undefined;
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
          remediation: REMEDIATION(meta.name),
          key,
        });
      } else if (next === 'connecting' && prev === 'error') {
        // Already retrying, so a Reconnect button here would just repeat the chip.
        showReconnecting(msg.status.message);
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
      // This channel carries whatever a command failed with, camera error paths and
      // status codes alike — often a raw string like ".../livestreams/0/stop". Keep it
      // out of the serif title (a "0" there reads as "o") and mono in the body instead.
      // Keyed on the text so a repeated refusal updates its card instead of stacking.
      notify({
        tier: 'error',
        kind: 'System',
        source: 'Core',
        title: 'Command failed',
        body: msg.message,
        mono: true,
        key: `error:${msg.message}`,
      });
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
  sendWsCommand('connectors.status');
}
