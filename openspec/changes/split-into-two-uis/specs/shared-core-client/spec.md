## ADDED Requirements

### Requirement: A single shared package defines the core boundary
The repository SHALL provide one shared package that is the only way a rendering UI reaches the core. It SHALL expose the core's HTTP operations, the WebSocket client, the message and response schemas, and the optional desktop host adapter, behind a single public entry point.

#### Scenario: UI reaches the core only through the package
- **WHEN** a rendering UI's source is inspected
- **THEN** every call to the core is made through the shared package's public entry point
- **AND** no UI file constructs an HTTP request, a WebSocket connection or a desktop host call directly

#### Scenario: Direct transport use is rejected
- **WHEN** a file outside the shared package uses `fetch`, `WebSocket` or a desktop host API directly
- **THEN** the lint rule fails and identifies the file

#### Scenario: Rule applies to every UI
- **WHEN** a new rendering UI is added to the workspace
- **THEN** the same restriction applies to it without per-UI configuration

### Requirement: The package is framework-agnostic
The shared package SHALL NOT depend on any user-interface framework, and SHALL NOT contain components, styling or framework-specific state primitives. It SHALL be consumable by a UI written in any framework.

#### Scenario: No framework dependency
- **WHEN** the shared package's dependencies and source are inspected
- **THEN** no user-interface framework is present as a dependency or an import

#### Scenario: Consumable from a different framework
- **WHEN** a UI written in a framework other than that of an existing UI consumes the package
- **THEN** it can perform every core operation, subscribe to every core event and read every schema type without adaptation

### Requirement: Message and response schemas are defined once
Every HTTP response and every WebSocket message SHALL be defined by exactly one schema, held in the shared package, and the types the UIs use SHALL be derived from those schemas rather than declared separately.

#### Scenario: One definition per message
- **WHEN** a WebSocket message or HTTP response shape is needed by more than one UI
- **THEN** it is defined once in the shared package
- **AND** no UI declares its own copy of that shape

#### Scenario: Adding a message updates every UI at once
- **WHEN** a new core message is added to the shared package
- **THEN** it becomes available to every UI without per-UI schema changes

#### Scenario: Types follow the schemas
- **WHEN** a schema changes
- **THEN** the exported types change with it, and a UI relying on the previous shape fails type-checking rather than failing at runtime

### Requirement: Responses are validated at the boundary
The shared package SHALL validate core responses and incoming messages against their schemas before returning them, so a UI never receives data that does not match its declared type.

#### Scenario: Invalid payload is rejected at the boundary
- **WHEN** the core returns a payload that does not satisfy its schema
- **THEN** the shared package reports a validation failure rather than passing the payload to the UI

### Requirement: Optional host capabilities are feature-detected
Desktop-only capabilities SHALL be exposed through the shared package as optional and feature-detectable. A UI running outside the desktop shell SHALL be able to determine that a capability is unavailable and degrade, rather than fail.

#### Scenario: Capability absent outside the desktop shell
- **WHEN** a UI queries a desktop-only capability while running in a browser
- **THEN** the package reports it as unavailable
- **AND** the UI continues to function without it

#### Scenario: No capability is reachable only through the desktop shell
- **WHEN** a core operation is invoked from a UI running against a headless core
- **THEN** it succeeds over HTTP or WebSocket without requiring the desktop shell

### Requirement: Translation catalogues are shared
The translation catalogues SHALL live in the shared package so that every UI translates the same keys from the same source, while remaining free to use its own translation runtime and to add UI-specific keys.

#### Scenario: Shared catalogue consumed by each UI
- **WHEN** a UI renders translated content
- **THEN** it resolves shared keys from the shared catalogues
- **AND** a correction to a shared translation reaches every UI

#### Scenario: UI-specific keys remain local
- **WHEN** a UI needs a string no other UI uses
- **THEN** it may define that key locally without adding it to the shared catalogues
