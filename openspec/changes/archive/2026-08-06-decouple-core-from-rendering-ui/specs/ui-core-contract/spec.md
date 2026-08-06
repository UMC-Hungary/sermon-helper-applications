## ADDED Requirements

### Requirement: Single core client SDK
A rendering UI SHALL access the core exclusively through one typed core client SDK that unifies HTTP and WebSocket access. UIs MUST NOT call Tauri `invoke`, raw `fetch`, or construct WebSockets directly.

#### Scenario: All access routed through the SDK
- **WHEN** a rendering UI performs any core operation or subscribes to any realtime event
- **THEN** it does so through a method of the core client SDK
- **AND** no rendering-UI code imports `@tauri-apps/api` IPC, raw `fetch`, or the `WebSocket` constructor directly

#### Scenario: Runtime validation at the boundary
- **WHEN** data crosses the SDK boundary (HTTP response, WS message, or IPC result)
- **THEN** the SDK validates it with Zod schemas and returns inferred TypeScript types

### Requirement: Transport-agnostic operation with optional IPC fast-path
The core client SDK SHALL select its transport based on runtime mode: HTTP/WS against a remote core in client mode, and HTTP/WS against the local core in server mode, using Tauri IPC only as an optional fast-path when running inside the desktop app.

#### Scenario: Client mode uses network transport
- **WHEN** the app runs in client mode
- **THEN** the SDK routes all calls over HTTP/WS to the configured server URL with the stored auth token

#### Scenario: Desktop server mode may use IPC fast-path
- **WHEN** the app runs inside the Tauri desktop app in server mode
- **THEN** the SDK MAY use Tauri IPC for local operations while remaining functionally equivalent to the HTTP/WS path
- **AND** disabling the IPC fast-path leaves all operations working over HTTP/WS

### Requirement: Documented UI authoring convention
The project SHALL define and document a convention for authoring a rendering UI ("how to write a UI"), specifying the required entrypoint, the core client SDK dependency, the produced static build output, and the metadata a UI must expose to the registry.

#### Scenario: New UI follows the convention
- **WHEN** a developer creates a new rendering UI following the documented convention
- **THEN** the UI depends only on the core client SDK, exposes the required registry metadata, and produces a static build consumable as `frontendDist`
- **AND** it requires no changes to the Rust core to function
