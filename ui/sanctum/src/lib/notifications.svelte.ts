import type { ToastTone } from '@metocast/design-system';

// Tier drives both the toast accent and the bell's worst-active mark. `live` is the
// calmest positive signal, `error` the loudest; the order here is the severity order.
export type Tier = ToastTone; // 'live' | 'ok' | 'warn' | 'error'
const RANK: Record<Tier, number> = { live: 0, ok: 1, warn: 2, error: 3 };

export interface NotifAction {
  label: string;
  primary?: boolean;
  run: () => void;
}

export interface Notification {
  id: number;
  tier: Tier;
  kind: string;
  source: string;
  title: string;
  body?: string;
  /** Technical line, set in the mono face. */
  detail?: string;
  /** Numbered remediation steps, disclosed on demand. */
  remediation?: string[];
  actions?: NotifAction[];
  /** A settled/in-progress chip beside the source — "reconnecting", "offline". */
  state?: string;
  /** Each affected source, when several are folded into one notification. */
  group?: { source: string; label: string }[];
  /** Persistent items never self-dismiss from the transient rail. */
  persistent: boolean;
  /** Dedupe/resolve handle — a connector id, so recovery clears its own failure. */
  key?: string;
  /** simple-icons path for the source's brand logo, masked into the Glyph tile. */
  brand?: string;
  /** Set once the source recovers; a resolved failure no longer blocks a fresh one. */
  resolved?: boolean;
  createdAt: number;
  railDismissed: boolean;
  read: boolean;
}

let items = $state<Notification[]>([]);
let seq = 0;
const RAIL_CAP = 3;

function worst(list: Notification[]): Tier {
  return list.reduce<Tier>((t, n) => (RANK[n.tier] > RANK[t] ? n.tier : t), 'live');
}

/** The transient rail: newest first, capped, still-showing items only. */
export function railItems(): Notification[] {
  return items.filter((n) => !n.railDismissed).slice(-RAIL_CAP).reverse();
}

/** The centre's full history, newest first. */
export function allNotifications(): Notification[] {
  return items.slice().reverse();
}

export function unreadCount(): number {
  return items.filter((n) => !n.read).length;
}

/** The worst tier still unread — what the bell's mark should show. `off` when clear. */
export function topTier(): Tier | 'off' {
  const unread = items.filter((n) => !n.read);
  return unread.length ? worst(unread) : 'off';
}

export function notify(n: Omit<Notification, 'id' | 'createdAt' | 'railDismissed' | 'read' | 'persistent'> & { persistent?: boolean }): number {
  if (n.key) {
    const active = items.find((i) => i.key === n.key && !i.resolved);
    if (active) {
      Object.assign(active, n);
      return active.id;
    }
  }
  const id = ++seq;
  const persistent = n.persistent ?? n.tier === 'error';
  items.push({ ...n, persistent, id, createdAt: Date.now(), railDismissed: false, read: false });
  if (!persistent) setTimeout(() => dismissRail(id), 6000);
  return id;
}

/** Clears a keyed failure when its source recovers, and announces the recovery. */
export function resolveByKey(key: string, recovery?: Pick<Notification, 'kind' | 'source' | 'title' | 'body'>): void {
  for (const n of items) if (n.key === key) { n.railDismissed = true; n.resolved = true; }
  if (recovery) notify({ ...recovery, tier: 'ok', persistent: false });
}

/** Removes a notification from the toast rail; it stays in the centre. */
export function dismissRail(id: number): void {
  const n = items.find((i) => i.id === id);
  if (n) n.railDismissed = true;
}

export function markAllRead(): void {
  for (const n of items) n.read = true;
}

export function clearAll(): void {
  items = [];
}

// ── Notification centre open state ───────────────────────────────────────────
let centreOpen = $state(false);
export function isCentreOpen(): boolean { return centreOpen; }
export function openCentre(): void { centreOpen = true; markAllRead(); }
export function closeCentre(): void { centreOpen = false; }

// ── Back-compat shims for the connectors page, which speaks the old toast API ──
export interface ToastItem {
  kind: string;
  source: string;
  title: string;
  body?: string;
  tone: ToastTone;
}
export function toasts(): Notification[] {
  return railItems();
}
export function pushToast(t: ToastItem): void {
  notify({ ...t, tier: t.tone, key: `${t.source}:${t.title}` });
}
export function dismissToast(id: number): void {
  dismissRail(id);
}
