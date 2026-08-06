## Context

The core stack (embedded PostgreSQL, Axum HTTP/WS server, connectors, scheduler, uploads) is bootstrapped inside `src-tauri/src/lib.rs::run` → `start_server` (`lib.rs:392`), which depends on Tauri `AppHandle` and `tauri-plugin-store` for config/token/mode. A second, hand-maintained bootstrap already exists in `src-tauri/src/bin/test_server.rs` — it boots the same stack with **no Tauri or display** by calling `server::build_and_serve(...)` directly, proving headless is feasible but duplicating startup and drifting from the real path (no static serving, no AppHandle-based OAuth, no scheduler wiring parity).

The rendering UI is the single SvelteKit app in `/src`, built via `adapter-static` to `/build` and loaded by Tauri through `frontendDist` in `tauri.conf.json`. It talks to the core through **two** channels with no unified boundary: Tauri IPC (`invoke`, plugins store/dialog/opener/liquid-glass; commands in `src-tauri/src/commands/`) and HTTP + `/ws` WebSocket (`lib/api/`, `lib/ws/`, schemas in `lib/schemas/`). Server/client mode already exists (`lib/stores/mode.ts`, `commands/server.rs`, `ConnectorInit.svelte`).

Confirmed intent: **full separate frontends**, **build-time selection** of which UI ships, and priority order **headless core first → standardized UI authoring + registry**.

## Goals / Non-Goals

**Goals:**
- A single shared, display-free core runtime used by both the Tauri app and a first-class headless server binary; delete the duplicated `test_server.rs` bootstrap.
- Replace the core's direct `tauri-plugin-store` dependency with a `ConfigProvider`/`SecretProvider` abstraction (Tauri-backed on desktop, env/file-backed headless).
- Every UI-needed operation/event reachable over HTTP/WS; Tauri IPC becomes an optional desktop fast-path adapter over the same core operations.
- One typed core client SDK on the frontend that all UIs use; no UI touches IPC/`fetch`/`WebSocket` directly.
- A UI registry + build-time selection wiring `frontendDist`; current app registered as UI #1. Settings selector when >1 UI is bundled.

**Non-Goals:**
- Runtime hot-swapping of UIs without reload (explicitly build-time selection).
- Shipping a second real design in this change (the example/second UI is future work; this change delivers the boundary + registry mechanics and keeps the existing UI working).
- Changing connector behavior, DB schema, or the WS protocol semantics beyond surfacing previously-IPC-only operations.
- Multi-tenant/auth redesign.

## Decisions

### 1. Extract a `core::runtime` bootstrap shared by desktop + headless
Introduce a `runtime` module in `metocast_lib` that owns the ordered bootstrap (Postgres → migrations → pool → connectors → scheduler → `build_and_serve`) and returns a handle. `lib.rs::start_server` and the new headless binary both call it. `AppRuntime` fields that are Tauri-specific are supplied by the caller, not assumed.
- **Why over keeping two paths:** eliminates the `test_server.rs` drift and makes headless a first-class, tested path instead of a test-only shim.
- **Alternative considered:** keep `test_server.rs` and sync manually — rejected; already drifting.

### 2. `ConfigProvider` + `SecretProvider` traits abstract persistence
Define traits for reading/writing mode, server port, auth token, client URL, and connector configs. Desktop impl wraps `tauri-plugin-store` (current behavior preserved); headless impl reads env vars + a config file (extends today's `TAURI_AUTH_TOKEN`/`TEST_SERVER_PORT` env approach). The core depends only on the traits.
- **Why:** the core's only remaining Tauri coupling is config access; abstracting it is what unlocks headless.
- **Alternative:** pass a plain config struct — rejected because some values are mutated at runtime (token/mode) and must persist back through the provider.

### 3. HTTP/WS is the source of truth; IPC is an optional adapter
Audit every `invoke()` call site in `/src` and every command in `src-tauri/src/commands/`. For any operation a UI needs that is IPC-only today, add/confirm an HTTP or WS equivalent and update all contract surfaces (OpenAPI `server/openapi.rs`, Bruno `bruno/`, Zod schemas, presenter web view, `presenter-receiver`, Companion) per repo rules. IPC commands remain as thin wrappers delegating to the same core functions for the desktop fast-path.
- **Why:** headless + separate frontends both require that no capability is trapped behind IPC.
- **Trade-off:** some desktop-only OS integrations (dialogs, opener, liquid-glass, badge) are inherently IPC/Tauri; these are treated as **host capabilities** exposed through the SDK behind capability detection, degrading gracefully in client/headless UIs.

### 4. Core client SDK on the frontend (`src/lib/core-client/`)
A single typed module composes today's `lib/api/` + `lib/ws/` and an optional IPC transport. It picks transport from `appMode` + Tauri detection, validates all boundary data with Zod (`z.infer` types), and exposes host-capability methods with feature detection. UI code imports only this SDK. Lint rule/convention forbids direct `@tauri-apps/api` IPC, raw `fetch`, and `new WebSocket` outside the SDK.
- **Why:** one boundary is the precondition for "standardize how to write a UI."
- **Alternative:** publish the SDK as a separate npm package now — deferred; keep in-repo until a second UI actually needs it.

### 5. UI registry + build-time selection
Add a registry manifest (e.g. `ui/registry.json` or a typed module) listing `{ id, displayName, buildDir, entry }`; register the existing app as `default`. A build step selects the active UI(s) via an env/config flag (default = existing app), builds them, and stages output into `frontendDist` (adjust `beforeBuildCommand`/`frontendDist` in `tauri.conf.json` and `svelte.config.js` output as needed). When >1 UI is bundled, a new `settings` control (`src/routes/settings`, `src/lib/components/settings`) plus a persisted setting selects the active UI, loaded on next start.
- **Why over runtime loader:** user chose build-time selection; avoids shipping/serving multiple bundles and a dynamic loader.
- **Alternative:** Tauri serves a chosen bundle at runtime from disk — out of scope (runtime switching non-goal).

## Risks / Trade-offs

- **Hidden IPC-only capabilities surface late** → Do the `invoke()` call-site audit up front (Task phase 2) and gate the change on parity before touching build/registry work.
- **Contract-surface drift** (OpenAPI/Bruno/Zod/presenter/receiver/Companion) → Repo rule already mandates same-change sync; enumerate touched endpoints in tasks and verify via Bruno + `pnpm check`.
- **Desktop-only OS features can't be headless** → Model them as optional host capabilities with feature detection; UIs must tolerate their absence rather than assume Tauri.
- **Refactor regressions in startup ordering** → Keep `build_and_serve` signature stable initially; move only the bootstrap orchestration; validate with existing e2e (`e2e/`) against the new headless binary.
- **Scope creep toward a second UI** → Explicit non-goal; this change proves the boundary with the existing app as the sole registered UI.

## Migration Plan

1. Land `core::runtime` + provider traits with desktop impl; keep behavior identical (no user-visible change).
2. Replace `bin/test_server.rs` with the headless binary on the shared runtime; point `e2e` at it.
3. Audit and close IPC→HTTP/WS parity gaps, syncing all contract surfaces.
4. Introduce `core-client` SDK; migrate `/src` call sites onto it; add the no-direct-IPC convention.
5. Add registry + build-time selection (default = current app) and the settings selector.

Rollback: each phase is independently revertable; phases 1–2 are internal refactors with no contract change, and the build/registry step defaults to current behavior when unconfigured.

## Open Questions

- Exact home for the headless binary/config schema (new `[[bin]]` name, env var names) — resolve during phase 2.
- Whether the core client SDK stays in-repo or becomes a workspace package once a real second UI exists (default: in-repo for now).
