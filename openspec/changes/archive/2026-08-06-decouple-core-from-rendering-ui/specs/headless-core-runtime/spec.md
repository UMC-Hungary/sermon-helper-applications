## ADDED Requirements

### Requirement: Display-free core bootstrap
The core runtime SHALL boot the full application stack — embedded PostgreSQL, migrations, connection pool, connectors, scheduler, and the Axum HTTP/WS server — without requiring Tauri, a windowing system, or a display.

#### Scenario: Headless binary starts the full stack
- **WHEN** the standalone headless server binary is launched on a machine with no display
- **THEN** it starts embedded PostgreSQL, runs migrations, initializes connectors and the scheduler, and serves the Axum HTTP/WS endpoints
- **AND** it exits cleanly on SIGINT/SIGTERM, stopping the embedded database

#### Scenario: Desktop and headless share one bootstrap path
- **WHEN** the Tauri desktop app starts in server mode
- **THEN** it initializes the core through the same shared runtime module used by the headless binary
- **AND** no core bootstrap logic is duplicated between the desktop entrypoint and the headless binary

### Requirement: UI-agnostic configuration and secrets
The core runtime SHALL obtain configuration, mode, auth token, and connector secrets without depending on `tauri-plugin-store`, so a headless process can supply them from environment variables and its own database.

Configuration reaches the core as plain values through the `CoreOptions` struct its callers construct, and connector configuration is stored in the core's own `app_settings` table. Provider *traits* were considered and dropped: with no Tauri store left in the startup path there was no second implementation for them to abstract over.

#### Scenario: Headless config from environment
- **WHEN** the headless binary starts with configuration supplied via environment variables
- **THEN** the core reads mode, server port and auth token from the values passed into `CoreOptions`, and connector configuration from `app_settings`, without any Tauri store access

#### Scenario: Desktop config preserved
- **WHEN** the Tauri desktop app starts
- **THEN** it reads its host-side settings from `tauri-plugin-store` and passes them into the same `CoreOptions`, preserving existing persisted settings behavior
- **AND** connector configuration still held in the desktop store is imported into `app_settings` on first boot

### Requirement: Full core operations exposed over HTTP/WS
Every core operation and event required by a rendering UI SHALL be reachable over the HTTP/WS API so that a UI can function against a headless core with no Tauri IPC available.

#### Scenario: Client-mode UI needs no IPC
- **WHEN** a rendering UI runs in client mode against a remote headless core
- **THEN** it can perform every **core** operation and receive all realtime events through HTTP and the `/ws` WebSocket protocol without invoking any Tauri command

#### Scenario: Host capabilities are optional and feature-detected
- **WHEN** a rendering UI runs outside the desktop shell, or in a shell that lacks a capability
- **THEN** capabilities that are host-side rather than core-side — choosing which core this window talks to, application log files, in-app updates, native dialogs — report their absence through `hostCapabilities`
- **AND** the UI hides or degrades those controls instead of failing, since no core operation depends on them

#### Scenario: Contract surfaces stay in sync
- **WHEN** an operation previously available only via Tauri IPC is exposed over HTTP/WS
- **THEN** the OpenAPI generation, Bruno requests, frontend Zod schemas/clients, presenter web view, presenter-receiver, and Companion plugin are updated in the same change to reflect the new contract
