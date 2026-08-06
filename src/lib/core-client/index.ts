/**
 * Core client SDK — the only boundary a rendering UI uses to reach the core.
 *
 * A UI imports from `$lib/core-client` and nothing else transport-shaped: no
 * `@tauri-apps/*`, no raw `fetch`, no `new WebSocket`. That rule is enforced by
 * ESLint (see `eslint.config.js`), which allows those only inside this SDK.
 *
 * Three layers live behind it:
 *
 * - **HTTP** (`$lib/api`) — request/response core operations. Every response is
 *   validated with Zod, and the exported types are `z.infer`red from those
 *   schemas, so what the compiler believes matches what the server sent.
 * - **WebSocket** (`$lib/ws`) — realtime events and commands, same schemas.
 * - **Host** (`$lib/host`) — the optional desktop shell: which core this window
 *   talks to, log files, updates, native dialogs. Every capability is
 *   feature-detected via `hostCapabilities`; a UI outside the desktop app must
 *   degrade rather than assume.
 *
 * Transport selection is not a decision the UI makes. HTTP and WS always address
 * the configured core — the local one in server mode, a remote one in client
 * mode — and the host layer is a local fast-path for things that are not core
 * operations at all. Nothing here is only reachable through IPC, which is what
 * lets a UI run against a headless core.
 */

// ── Core operations over HTTP ─────────────────────────────────────────────────
export * from '$lib/api/activities.js';
export * from '$lib/api/broadlink.js';
export * from '$lib/api/client.js';
export * from '$lib/api/connectors.js';
export * from '$lib/api/cron-jobs.js';
export * from '$lib/api/events.js';
export * from '$lib/api/presentations.js';
export * from '$lib/api/queues.js';
export * from '$lib/api/recordings.js';
export * from '$lib/api/untracked-recordings.js';
export * from '$lib/api/uploads.js';
export { bibleApi } from '$lib/utils/bible-api.js';

// ── Realtime ──────────────────────────────────────────────────────────────────
export { connectWs, disconnectWs, sendWsCommand, connectPresenterWs } from '$lib/ws/client.js';

// ── Optional desktop host ─────────────────────────────────────────────────────
// Exported both flat and as a namespace: `hostCapabilities.logs` reads well at a
// call site, and `host.openExternal(...)` reads well when a component uses several.
export * from '$lib/host/index.js';
export * as host from '$lib/host/index.js';

// ── This build's rendering UIs ────────────────────────────────────────────────
export * from './uis.js';

// ── Boundary types, inferred from the Zod schemas ─────────────────────────────
export type {
  ConnectorConfigMap,
  ConnectorName,
  ObsStreamSettings,
} from '$lib/schemas/connectors.js';
export type { BiblePassage, BibleSuggestion } from '$lib/schemas/bible.js';
export type { WsMessage } from '$lib/schemas/ws-messages.js';
