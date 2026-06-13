# Plan: Changeable UI and Backend Separation

## Goal

Make the Svelte UI replaceable without rewriting the Rust backend, connector orchestration, WebSocket protocol, or persistence layer. The first version should still ship inside the current Tauri app, but the UI should behave like a client of a stable application contract rather than a privileged part of the backend.

## Current Friction

- `src/lib/api/client.ts` already gives most routes a typed HTTP seam, but `src/lib/components/layout/ConnectorInit.svelte`, `src/lib/components/connectors/ConnectorSettingsBlock.svelte`, setup, settings, updater, token, badge, and file-picker flows still call Tauri IPC directly.
- `src/lib/ws/client.ts` both owns WebSocket transport and mutates many UI stores. This makes the WebSocket protocol harder to reuse from another UI because message handling is tied to Svelte store implementation details.
- Connector configuration is split between REST endpoints, WebSocket updates, and direct Tauri commands. That gives the desktop UI knowledge that a web/mobile/alternate UI cannot share.
- Bootstrapping mode, auth, server URL, local network URL, connector statuses, Tauri event listeners, locale, and WebSocket startup all live in `ConnectorInit.svelte`. It is a shallow module with too much implementation leaking into the root layout.
- Some features are truly local shell capabilities, such as opening a file picker, saving a Bruno collection, opening a browser, applying macOS liquid glass, and checking desktop updates. These need a narrow local-platform seam rather than direct calls scattered through UI modules.

## Target Architecture

The UI should only know three client-side seams:

1. `AppBackend`
   - Typed HTTP API methods for events, recordings, activities, connectors, Broadlink, uploads, cron jobs, presentations, presenter control, token/config/bootstrap, and setup.
   - Implementations:
     - `HttpBackend` for normal app/server communication.
     - `TauriBootstrapBackend` only where the desktop shell must start or configure a local server before HTTP is available.
     - Test/mock backend for UI tests and future design work.

2. `RealtimeClient`
   - WebSocket connection lifecycle plus typed inbound/outbound messages.
   - It parses with Zod and emits domain events; it does not directly update Svelte stores.
   - A separate `realtimeProjection` module applies those events to Svelte stores.

3. `PlatformShell`
   - Local-only capabilities: open external URL, choose folder/file, save local files/collections, check updates, refresh token from local store if needed, window appearance, desktop-only integrations.
   - Implementations:
     - `TauriShell`
     - `BrowserShell` with safe fallbacks/no-ops for hosted or alternate UIs.

The Rust backend should expose everything that is not inherently local-shell behavior through HTTP or WebSocket. Tauri commands become bootstrapping and local shell adapters, not the primary product contract.

## Non-Goals

- Do not replace SvelteKit during this work.
- Do not remove Tauri packaging.
- Do not change the existing REST or WebSocket protocol without updating OpenAPI, Bruno, docs, presenter, presenter receiver, and Companion in the same change.
- Do not move embedded PostgreSQL, connector workers, scheduler, or upload ownership into the UI.

## Phase 1: Inventory and Contract Map

1. Create a contract inventory table covering:
   - HTTP clients under `src/lib/api/`
   - WebSocket commands/messages in `src/lib/ws/client.ts` and `src/lib/schemas/ws-messages.ts`
   - Direct Tauri IPC calls in `src/routes/` and `src/lib/`
   - Tauri event listeners
   - local shell features that cannot be served by the backend
2. Mark each item as one of:
   - backend contract, should be REST/WS
   - realtime contract, should be WS
   - local shell, should stay behind `PlatformShell`
   - bootstrap-only Tauri command
3. Add this map to the plan or a follow-up `docs/architecture/ui-backend-contract.md`.

Acceptance:

- Every `invoke(`, `listen(`, `openUrl(`, and dialog `open(` call has an intended future seam.
- No implementation changes yet except docs.

## Phase 2: Introduce Frontend Client Seams

1. Add `src/lib/backend/`:
   - `types.ts` for `AppBackend`, `RealtimeClient`, `PlatformShell`, bootstrap/session types.
   - `http-backend.ts` wrapping existing `src/lib/api/*` modules.
   - `tauri-shell.ts` wrapping direct Tauri plugins and commands that are local-only.
   - `browser-shell.ts` for browser-safe fallbacks.
   - `runtime.ts` to choose adapters based on environment and app mode.
2. Move token/server URL resolution into a `bootstrapAppSession()` function that returns a typed session object validated with Zod.
3. Keep existing stores, but make them consume the new seams rather than Tauri imports directly.

Acceptance:

- UI modules can import `getBackend()` / `getPlatformShell()` instead of Tauri APIs.
- Direct Tauri imports begin shrinking to adapter files.
- `pnpm check` passes.

## Phase 3: Deepen Realtime Handling

1. Split `src/lib/ws/client.ts` into:
   - `transport.ts`: connect, reconnect, send, disconnect.
   - `messages.ts`: Zod parsing and command builders.
   - `projection.ts`: applies typed messages to Svelte stores.
2. Replace stringly `sendWsCommand(type, data)` calls with typed command helpers.
3. Keep `lastWsMessage` only as debugging/state inspection, not as the primary integration point for screens.

Acceptance:

- WebSocket transport can run without importing Svelte stores.
- Store projection is testable with plain typed messages.
- Presenter, OBS devices, uploads, connectors, and event updates still behave the same.

## Phase 4: Move Product Capabilities to REST/WS

Add or complete backend endpoints for capabilities that are product behavior rather than local shell behavior:

- connector config read/write for OBS, vMix, ATEM, Broadlink, YouTube, Facebook, Discord
- connector connect/disconnect actions where remote control is valid
- OBS stream settings read/write
- OAuth auth URL generation and logout flows
- app setup/session status where needed by non-Tauri clients

Keep these synchronized with:

- `src-tauri/src/server/openapi.rs`
- Bruno requests under `bruno/`
- frontend Zod schemas and API clients
- Companion plugin when it uses the same protocol surface
- README/docs that describe external access

Acceptance:

- `ConnectorSettingsBlock.svelte` no longer imports Tauri APIs directly.
- Client mode and server mode use the same frontend module calls once bootstrapped.
- Tauri commands remain available only as desktop-local adapters or compatibility shims.

## Phase 5: Extract App Initialization From Layout

Replace `ConnectorInit.svelte` with a headless app runtime initializer:

- `src/lib/app/bootstrap.ts` resolves session and mode.
- `src/lib/app/connector-initial-state.ts` loads initial connector config/status through `AppBackend`.
- `src/lib/app/start-realtime.ts` opens WebSocket and wires projections.
- `src/lib/app/errors.ts` keeps connector error synchronization localized.
- The layout renders an `AppRuntime.svelte` wrapper that only starts/stops lifecycle work.

Acceptance:

- `+layout.svelte` has no product bootstrapping logic.
- Initialization can be run in a browser-hosted UI with `BrowserShell`.
- Setup/presenter/caption routes have explicit runtime behavior.

## Phase 6: Make the UI Changeable

1. Define a UI surface that another Svelte package, route set, or future frontend can consume:
   - exported backend/realtime/platform seams
   - exported domain schemas
   - CSS design tokens in `src/app.css`
   - route-level feature contracts
2. Introduce a theme/design-token layer:
   - semantic tokens for backgrounds, text, borders, status colors, spacing, and shell layout
   - no backend assumptions inside visual components
3. Separate screens from feature state:
   - routes compose feature modules
   - feature modules talk to `AppBackend`/`RealtimeClient`
   - presentational components receive props/callbacks where practical
4. Optional later step: move the UI into `apps/desktop-ui` or `packages/ui` once imports are clean enough to make the move boring.

Acceptance:

- A new UI can be prototyped using mock `AppBackend`, mock `RealtimeClient`, and `BrowserShell`.
- Visual redesign work can happen without touching Rust, SQLx, connectors, or scheduler code.
- Backend changes require schema/client updates, not component archaeology.

## Testing Strategy

- Unit-test `RealtimeClient` parsing and `projection.ts` store updates with Vitest.
- Unit-test `AppBackend` mock behavior for key screens.
- Add a small browser-mode smoke test that boots the UI with `BrowserShell` and mock/session config.
- Keep `pnpm check` mandatory after each phase.
- For any REST/WS contract changes, add backend/API verification and update Bruno/OpenAPI in the same branch.

## Suggested Implementation Order

1. Phase 1 documentation inventory.
2. `PlatformShell` first, because it removes direct Tauri imports with low protocol risk.
3. `AppBackend` session/bootstrap seam.
4. Realtime transport/projection split.
5. Connector settings migration to REST/WS.
6. UI theme/changeability work.

## Risks and Mitigations

- Risk: accidentally weakening server/client mode boundaries.
  - Mitigation: keep server ownership in Rust and route all client-mode behavior through HTTP/WS.
- Risk: protocol drift across UI, Companion, presenter, and Bruno.
  - Mitigation: treat OpenAPI, Bruno, Zod schemas, and external clients as one contract surface.
- Risk: introducing abstract pass-through modules.
  - Mitigation: use the deletion test. Keep only seams that have at least two real adapters or a clear test/mock adapter.
- Risk: too much refactor in one branch.
  - Mitigation: land phases independently, with `pnpm check` green at every stop.

## First Concrete Issue

Create `PlatformShell` and migrate these direct local-shell calls:

- `openUrl` usage in relogin, live events, updater, app version, and connector OAuth launch
- dialog folder selection in presentations and connection guide
- Bruno collection save behind a shell method
- system appearance behind a shell/window appearance adapter

This gives immediate leverage: screens stop importing Tauri plugins directly, while backend contracts remain untouched.
