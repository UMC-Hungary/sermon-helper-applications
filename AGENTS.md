# AGENTS.md

This file provides guidance to Codex (Codex.ai/code) when working with code in this repository.

## Project Overview

Church livestream control desktop application built with **Tauri 2 + SvelteKit 5 + TypeScript**.

## Commands

```bash
pnpm dev              # Vite dev server only (port 1420)
pnpm tauri dev        # Full Tauri desktop app in dev mode
pnpm tauri build      # Production build
pnpm check            # TypeScript + Svelte type checking
pnpm check:watch      # Watch mode for type checking
```

## Architecture

**Tech Stack:** SvelteKit 2.9 + Svelte 5 + Tauri 2 + TypeScript + Zod + Rust/Axum + PostgreSQL

**Frontend (`/src`):**

- `routes/` - SvelteKit file-based routing (SSG via adapter-static)
- `lib/components/ui/` - Reusable UI primitives (button, card, alert, dialog, etc.)
- `lib/components/sidebar.svelte` - Main navigation + system status display
- `lib/stores/` - Svelte stores for app state (`systemStore`, `obsStatus`)
- `lib/api/`, `lib/ws/`, `lib/schemas/` - Typed API/WebSocket clients with Zod validation
- `lib/utils/` - Browser/Tauri utilities and toast helpers

**Backend (`/src-tauri`):**

- Rust is the core application backend. It owns the Axum HTTP/WebSocket server, embedded PostgreSQL lifecycle, migrations, models, schedulers, upload workers, and connector orchestration.
- `server/` - Axum routes, auth, OpenAPI, WebSocket protocol, presenter/PPT/caption endpoints
- `database/` - embedded PostgreSQL startup plus SQLx migrations
- `connectors/` - OBS, YouTube, Facebook, Broadlink, Keynote, ATEM/vMix/Discord integrations
- `uploader/` and `scheduler/` - resumable upload orchestration and cron-triggered automation
- Uses Tauri plugins: store, dialog, opener, liquid-glass

**Key Integrations:**

- `obws` - Rust OBS WebSocket client for streaming/recording state and device scanning
- Axum WebSocket `/ws` - primary realtime control/event channel for the UI, Companion module, and presenter receiver
- Embedded PostgreSQL via `postgresql_embedded` + `sqlx`
- `svelte-sonner` - Toast notifications

**PostgreSQL Features:** The app uses PostgreSQL UUID primary keys generated with `pgcrypto`, `TIMESTAMPTZ`, foreign keys with cascading deletes, composite primary keys, partial indexes for pending upload work, JSON/JSONB-style payload storage through SQLx JSON support, PL/pgSQL trigger functions, and `pg_notify`/LISTEN notifications to broadcast database changes to WebSocket clients.

**WebSocket Protocol:** `src-tauri/src/server/websocket.rs` defines the command protocol for events, recordings, activities, cron jobs, uploads, connectors, Broadlink, OBS device listeners, presenter control, client registry, ping/pong latency, notifications, and presentation state broadcasts. Frontend message schemas live in `src/lib/schemas/ws-messages.ts`.

**Server/Client Mode Architecture:** The app can run in `server` or `client` mode. Mode is persisted through Tauri commands in `src-tauri/src/commands/server.rs` and mirrored in `src/lib/stores/mode.ts`. In `server` mode, the desktop app starts the embedded PostgreSQL database and Axum server on the configured port, owns connector workers, exposes localhost and LAN URLs, serves HTTP APIs, and broadcasts realtime state over `/ws`. In `client` mode, the app does not start the backend stack; it stores a remote server URL plus auth token and the Svelte frontend talks to that server via typed HTTP API calls and the same `/ws` WebSocket protocol. `src/lib/components/layout/ConnectorInit.svelte` resolves the mode at startup, sets `serverUrl`, `authToken`, `localNetworkUrl`, and connector status sources, then opens the WebSocket. The Connect page only exposes token/network install details in server mode.

## Key Patterns

**State Management:** Svelte stores (writable/derived) in `/lib/stores/`, props for component communication

**Styling:** Plain CSS with app-level CSS variables in `src/app.css`; Tailwind is not part of the active dependency stack.

**Runtime Validation:** Zod schemas define API, WebSocket, and persisted data boundaries. Infer TypeScript types from Zod schemas with `z.infer` whenever a value crosses a runtime boundary.

**Type Safety:** TypeScript runs in strict mode with `noUncheckedIndexedAccess`, `noImplicitOverride`, and `exactOptionalPropertyTypes`. Keep types explicit at module boundaries and avoid weakening strictness.

**SystemStatus Type** (`src/lib/stores/types.ts`):

```typescript
type SystemStatus = {
  obs: boolean;
  rodeInterface: boolean;
  mainDisplay: boolean;
  secondaryDisplay: boolean;
  youtubeLoggedIn: boolean;
};
```

## Implementation Status

**Complete:** Sidebar, UI components, OBS WebSocket integration, toast system, error messages display, Service Events

**Stubs:** Bible editor

## Type Definitions

Two SystemStatus definitions exist:

- `src/lib/stores/types.ts` - Flat structure (correct)
- `src/lib/types.ts` - Nested structure (outdated, to be consolidated)

## Rules

- Always fix all `pnpm check` errors before finishing, even if they are unrelated to your changes.
- Always name plan files with the format: PLAN-{feature-name}.md under the `plans` folder.
- Always use Zod for runtime validation of external, persisted, IPC, API, and WebSocket data.
- Always keep strict TypeScript types. Do not use `any`, broad casts, or relaxed compiler settings to bypass type errors.
- Respect server/client mode boundaries: server mode owns Postgres, Axum, connectors, schedulers, and uploads; client mode must call the configured server through typed HTTP/WebSocket clients rather than starting local backend services.
- Companion communication must go through the app WebSocket protocol. Do not add REST-only, filesystem, or ad hoc side channels for Companion control or status.
- Any REST or WebSocket contract change must update all matching contract surfaces in the same change: OpenAPI/Swagger generation in `src-tauri/src/server/openapi.rs`, Bruno requests under `bruno/`, project README/docs, frontend Zod schemas and API/WS clients, the presenter web view, the `presenter-receiver` terminal view, and the Companion plugin. Keep these surfaces in sync; do not leave protocol drift for a later task.
- When fixing release workflow problems, do not mint a new version tag for each CI attempt. Finish the workflow fix first, then republish the latest synchronized release tag set so the app, companion, and presenter receiver tags stay on the same version.
