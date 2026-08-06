## Why

The application core (Axum HTTP/WS server, embedded PostgreSQL, connectors, schedulers, uploads) is currently bootstrapped inside the Tauri layer (`src-tauri/src/lib.rs::start_server`), entangled with `AppHandle` and `tauri-plugin-store`. This means the core cannot run without a desktop window/display, and the rendering UI (the single SvelteKit app in `/src`) talks to the core through a mix of Tauri IPC *and* HTTP/WS with no stable, documented boundary. To support multiple independent frontend designs and true server/client deployments, the core must first be able to run **headless**, and the UI-to-core boundary must be **standardized** so any UI can be authored against it.

## What Changes

- Extract a **UI-agnostic core runtime** from `lib.rs` so the same bootstrap path (Postgres → migrations → pool → connectors → scheduler → Axum) is shared by the Tauri desktop app and a standalone headless binary. The existing `src-tauri/src/bin/test_server.rs` (which already boots the core with no Tauri/display) is replaced by a first-class headless server binary using this shared runtime, eliminating the current duplication/drift.
- Make **all core functionality reachable over HTTP/WS**. Tauri IPC commands (`src-tauri/src/commands/`) become a thin *optional* desktop adapter over the same core operations, not a required path. **BREAKING** for any UI code that assumes `invoke()` is always available.
- Replace the core's direct dependency on `tauri-plugin-store` for config/token/mode persistence with a **UI-agnostic config/secret provider** the headless binary can also satisfy (env/file-backed).
- Define a **standardized UI-core client contract**: a single typed SDK (wrapping HTTP + WS, with an optional IPC fast-path) that every rendering UI depends on instead of touching Tauri or raw fetch/WebSocket. Establish conventions ("how to write a UI") documenting required entrypoints, the core client dependency, and build output.
- Introduce a **UI registry**: a manifest describing available rendering UIs, **build-time selection** of which UI(s) are bundled into `frontendDist`, and a **settings selector** to choose the active UI among those bundled.

## Capabilities

### New Capabilities
- `headless-core-runtime`: A shared, display-free core bootstrap (Postgres, Axum, connectors, scheduler, uploads) usable by both the Tauri app and a standalone headless server binary, driven by a UI-agnostic config/secret provider.
- `ui-core-contract`: A standardized, UI-agnostic client SDK and authoring convention that exposes all core operations/events over HTTP+WS (IPC as optional desktop adapter), so any frontend design can be built against a stable boundary.
- `ui-registry`: A registry/manifest of selectable rendering UIs, with build-time bundling selection and a runtime settings selector for the active UI.

### Modified Capabilities
<!-- None: this is the first OpenSpec change; no existing specs in openspec/specs/. -->

## Impact

- **Rust core (`src-tauri/`)**: refactor `lib.rs::start_server` into a shared runtime module; new headless binary replacing `bin/test_server.rs`; config/secret provider abstraction replacing direct `tauri-plugin-store` reads in the core; `commands/*` reframed as optional IPC adapter. Must keep `pnpm check` and Rust build warning-free (no `#[allow(dead_code)]`).
- **Frontend (`/src`)**: consolidate all IPC/HTTP/WS access behind the core client SDK (`lib/api/`, `lib/ws/`, IPC call sites); the current app becomes the first UI in the registry.
- **Build/config**: `tauri.conf.json` `frontendDist`/`beforeBuildCommand`, `svelte.config.js`, and a UI-selection build step; new `settings` UI entry for active-UI selection (`src/routes/settings`, `src/lib/components/settings`).
- **Contract surfaces** (per repo rules): keep OpenAPI (`server/openapi.rs`), Bruno (`bruno/`), Zod schemas, presenter web view, `presenter-receiver`, and Companion in sync as HTTP/WS surfaces expand to cover previously IPC-only operations.
- **Modes**: reinforces existing server/client mode; server mode == headless-capable core, client mode == UI-only against a remote core.
