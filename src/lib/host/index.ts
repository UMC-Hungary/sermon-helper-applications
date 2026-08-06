/**
 * Host capabilities — the single place a rendering UI touches Tauri.
 *
 * Core operations belong on the HTTP/WS API and work against any core, local or
 * remote. What is left here is host-side: which core this window talks to, and
 * OS integrations (log files, updates, native dialogs) a browser cannot provide.
 *
 * Every capability is feature-detected. A UI running outside the desktop shell —
 * a browser tab, a second rendering UI, a remote client — sees `available: false`
 * and must hide or degrade the control rather than assume Tauri is there.
 */

import { ApplicationLogPathSchema, ApplicationLogTextSchema } from '$lib/schemas/logs.js';

/** Thrown when a UI calls a host capability that this environment does not have. */
export class HostUnavailableError extends Error {
  constructor(capability: string) {
    super(`Host capability "${capability}" is not available outside the desktop app`);
    this.name = 'HostUnavailableError';
  }
}

/** True when running inside the Tauri desktop shell. */
export function isHost(): boolean {
  return (
    typeof window !== 'undefined' &&
    typeof (window as Window & { __TAURI_INTERNALS__?: object }).__TAURI_INTERNALS__ !== 'undefined'
  );
}

/**
 * What this environment can do. UIs should branch on these rather than probing
 * for Tauri themselves.
 */
export const hostCapabilities = {
  /** Reading and changing which core this window talks to. */
  get mode(): boolean {
    return isHost();
  },
  /** Application log file access. */
  get logs(): boolean {
    return isHost();
  },
  /** In-app update checks and installation. */
  get updater(): boolean {
    return isHost();
  },
  /** Native file/folder dialogs. */
  get dialogs(): boolean {
    return isHost();
  },
};

async function invokeHost<T>(capability: string, command: string, args?: Record<string, unknown>) {
  if (!isHost()) throw new HostUnavailableError(capability);
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<T>(command, args);
}

// ── Which core this window talks to ──────────────────────────────────────────

export type HostMode = 'server' | 'client';

/**
 * The configured mode, or `null` when setup has not run. Outside the desktop app
 * there is no local core to choose, so the window is always talking to the core
 * that served it: `'server'`.
 */
export async function getAppMode(): Promise<HostMode | null> {
  if (!isHost()) return 'server';
  const mode = await invokeHost<string | null>('mode', 'get_app_mode');
  return (mode as HostMode | null) ?? null;
}

export function completeSetup(args: {
  mode: HostMode;
  serverUrl?: string;
  clientToken?: string;
}): Promise<void> {
  return invokeHost('mode', 'complete_setup', args);
}

export function resetSetup(): Promise<void> {
  return invokeHost('mode', 'reset_setup');
}

export function getServerPort(): Promise<number> {
  return invokeHost('mode', 'get_server_port');
}

export function getLocalHost(): Promise<string | null> {
  return invokeHost('mode', 'get_local_host');
}

export function getClientUrl(): Promise<string | null> {
  return invokeHost('mode', 'get_client_url');
}

export function getClientToken(): Promise<string> {
  return invokeHost('mode', 'get_client_token');
}

export function getToken(): Promise<string> {
  return invokeHost('mode', 'get_token');
}

export function refreshToken(): Promise<string> {
  return invokeHost('mode', 'refresh_token');
}

/** Unlocks reading stored connector secrets; server mode on the host only. */
export function getAdminToken(): Promise<string> {
  return invokeHost('mode', 'get_admin_token');
}

// ── Application log ───────────────────────────────────────────────────────────

export async function readApplicationLog(): Promise<string> {
  return ApplicationLogTextSchema.parse(await invokeHost<unknown>('logs', 'read_application_log'));
}

export async function getApplicationLogPath(): Promise<string> {
  return ApplicationLogPathSchema.parse(
    await invokeHost<unknown>('logs', 'get_application_log_path'),
  );
}

export function openApplicationLog(): Promise<void> {
  return invokeHost('logs', 'open_application_log');
}

export function downloadApplicationLog(destination: string): Promise<void> {
  return invokeHost('logs', 'download_application_log', { destination });
}

export function removeApplicationLog(): Promise<void> {
  return invokeHost('logs', 'remove_application_log');
}

// ── Updates ───────────────────────────────────────────────────────────────────

import type { UpdateInfo } from '$lib/stores/updater.js';

export function checkForUpdates(): Promise<UpdateInfo | null> {
  return invokeHost('updater', 'check_for_updates');
}

export function installUpdate(): Promise<void> {
  return invokeHost('updater', 'install_update');
}

// ── Bruno collection export ───────────────────────────────────────────────────

export function saveBrunoCollection(dir: string, files: Record<string, string>): Promise<void> {
  return invokeHost('dialogs', 'save_bruno_collection', { dir, files });
}

// ── OS integrations ───────────────────────────────────────────────────────────

/** Opens a URL in the system browser; falls back to a new tab in a browser UI. */
export async function openExternal(url: string): Promise<void> {
  if (!isHost()) {
    window.open(url, '_blank', 'noopener');
    return;
  }
  const { openUrl } = await import('@tauri-apps/plugin-opener');
  return openUrl(url);
}

export async function pickDirectory(title?: string): Promise<string | null> {
  if (!isHost()) throw new HostUnavailableError('dialogs');
  const { open } = await import('@tauri-apps/plugin-dialog');
  const selected = await open({ directory: true, multiple: false, ...(title ? { title } : {}) });
  return typeof selected === 'string' ? selected : null;
}

export async function pickSavePath(options: {
  defaultPath?: string;
  filters?: { name: string; extensions: string[] }[];
}): Promise<string | null> {
  if (!isHost()) throw new HostUnavailableError('dialogs');
  const { save } = await import('@tauri-apps/plugin-dialog');
  return save(options);
}

/** Subscribes to a desktop event. Outside the host this is a no-op unsubscribe. */
export async function listenToHost<T>(
  event: string,
  handler: (payload: T) => void,
): Promise<() => void> {
  if (!isHost()) return () => {};
  const { listen } = await import('@tauri-apps/api/event');
  return listen<T>(event, (e) => handler(e.payload));
}

// ── Host-side persisted settings ──────────────────────────────────────────────

/**
 * A key/value store owned by the host, for window-local preferences (locale,
 * caption overlay setup). Not core data — it never leaves this machine. Returns
 * `null` outside the desktop app so callers fall back to `localStorage`.
 */
export async function loadHostStore(name: string) {
  if (!isHost()) return null;
  const { load } = await import('@tauri-apps/plugin-store');
  return load(name);
}

// ── Window appearance ─────────────────────────────────────────────────────────

/** The host window's theme plus a subscription, or `null` in a browser. */
export async function watchHostTheme(
  onChange: (theme: 'light' | 'dark') => void,
): Promise<'light' | 'dark' | null> {
  if (!isHost()) return null;
  const { getCurrentWindow } = await import('@tauri-apps/api/window');
  const win = getCurrentWindow();
  const theme = await win.theme();
  await win.onThemeChanged(({ payload }) => onChange(payload === 'dark' ? 'dark' : 'light'));
  return theme === 'dark' ? 'dark' : 'light';
}

/**
 * Applies the host window's translucent material when the platform supports it.
 * Returns whether it was applied, so the UI can pick a solid background instead.
 */
export async function enableWindowGlass(): Promise<boolean> {
  if (!isHost()) return false;
  const { isGlassSupported, setLiquidGlassEffect } = await import('tauri-plugin-liquid-glass-api');
  if (!(await isGlassSupported())) return false;
  await setLiquidGlassEffect({});
  return true;
}
