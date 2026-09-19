# PLAN: Reusable Rust backend libraries for native Apple applications

Status: implemented
Scope: Rust workspace, Tauri host boundary, headless server, native iOS/macOS reuse, and Rust toolchain
Planning assumption: iOS is a remote client; macOS may be either a remote client or a local server host.

Implementation note: Apple device/simulator compilation and Swift package import verification run in
`.github/workflows/rust-libraries.yml`; local execution still requires a complete Xcode installation.

## Goal

Turn the current `src-tauri` Rust package into a small set of libraries with one-way dependencies so
that:

- a native iOS/macOS application can reuse typed protocol models, validation, business rules, and a
  remote client through Swift bindings;
- a native macOS host can optionally embed the same server runtime used by Tauri and
  `metocast-server`;
- Tauri becomes an adapter at the edge instead of a type used by the backend;
- the existing HTTP/WebSocket wire contract and server/client-mode behavior do not change during
  the extraction;
- all crates compile with strict Rust settings and a pinned, current stable toolchain.

Success is not “the code is spread across many crates.” Success is that an Apple target can link
only the parts it needs and no reusable crate imports Tauri, Axum, SQLx, or device-only libraries.

## Current state and constraints

The repository currently has one `src-tauri` package containing the Tauri shell, domain/wire types,
PostgreSQL access, Axum routes, WebSocket dispatch, connectors, scheduler, uploads, and both desktop
and headless bootstraps. The intended display-free boundary in `src-tauri/src/runtime.rs` is a good
starting point, but it still accepts `tauri::AppHandle`; the same type also leaks into server state,
asset serving, caption settings, log routes, and connector status emission.

Other facts that shape the split:

- `src-tauri/src/server/websocket.rs` and `routes.rs` combine wire DTOs, parsing, dispatch, and
  persistence. They are approximately 3,800 and 3,200 lines respectively.
- Public API models currently carry `sqlx::FromRow` in several places, coupling wire/domain types to
  PostgreSQL.
- Connector configuration and status types live beside concrete connector implementations.
- `blackmagic-camera` and `rodecaster` are already separate device libraries. Do not wrap or split
  them again without another real consumer.
- There is no root Cargo workspace, no `rust-toolchain.toml`, and no `rust-version` declaration.
  Local Rust is 1.93.1 and CI uses an unpinned `stable` channel.
- As of 2026-09-17, the current stable release is Rust 1.98.1. It fixes a compiler miscompilation in
  1.98.0, so the plan targets 1.98.1 rather than 1.98.0. See the
  [official Rust 1.98.1 announcement](https://blog.rust-lang.org/releases/latest/).
- The project rule requiring matching updates to OpenAPI, Bruno, docs, Zod schemas, presenter,
  presenter receiver, and Companion remains in force. This refactor should preserve the wire
  contract; any intentional contract change must update every surface in the same phase.

## Non-goals

- Do not run PostgreSQL, Axum, uploads, schedulers, or hardware connectors on iOS.
- Do not create one crate per connector, model, endpoint, or repository.
- Do not add a generic repository framework, dependency-injection container, internal service bus,
  or traits with only one implementation.
- Do not rewrite the HTTP/WebSocket protocol while moving it.
- Do not replace PostgreSQL or redesign authentication.
- Do not expose upstream credentials to the Apple bridge. The existing auth/admin/loopback rules
  remain server-only.
- Do not make the current Tauri-generated Apple project the distribution mechanism for the native
  library. Produce a normal XCFramework/Swift Package artifact that a separate Swift application can
  consume.

## Target architecture

```text
                                  +-------------------------+
                                  | native iOS/macOS Swift |
                                  +------------+------------+
                                               |
                                      generated Swift API
                                               |
                                  +------------v------------+
                                  | metocast-apple          |
                                  | UniFFI adapter only     |
                                  +------------+------------+
                                               |
                                  +------------v------------+
                                  | metocast-client         |
                                  | typed HTTP + WebSocket  |
                                  +------------+------------+
                                               |
                         +---------------------v---------------------+
                         | metocast-core                              |
                         | DTOs, commands/events, validation, rules  |
                         +---------------------^---------------------+
                                               |
                                  +------------+------------+
                                  | metocast-server         |
                                  | Axum/SQLx/runtime/devices|
                                  +------+-------------+----+
                                         |             |
                                +--------v---+   +-----v-------------+
                                | Tauri host |   | headless binary   |
                                +------------+   +-------------------+
```

Dependencies point downward only. `metocast-core` depends on no project crate. The client and server
may both depend on core, but never on one another. The Apple bridge depends on the client and core;
the reusable crates never depend on the bridge.

### 1. `crates/metocast-core`

A platform-neutral `rlib` containing only shared meaning:

- API request/response records and WebSocket command/event enums;
- event, recording, activity, connector, presenter, Bible, queue, and scheduler DTOs that are part of
  the public contract;
- connector configuration and status types plus their validation;
- pure rules such as current-event selection and published-title fallback;
- typed identifiers and closed enums where they prevent invalid states;
- typed, serializable public errors where the error is part of the contract.

Allowed dependencies should stay small: `serde`, `serde_json` only where the wire contract is
actually dynamic, `uuid`, `chrono`, and `thiserror`. It must not depend on Tauri, Tokio, reqwest,
Axum, SQLx, PostgreSQL, filesystem APIs, or concrete connectors.

Do not make SQL rows domain types. Keep private `*Row` records and SQL queries in the server crate,
then map them into core DTOs. Keep `serde_json::Value` only for deliberately open payloads such as a
third-party `extra` field; replace it with named records where the schema is owned by Metocast.

### 2. `crates/metocast-client`

A Rust remote-client library that compiles for iOS and macOS:

- `ClientConfig` validates the base URL and holds a redacted auth-token newtype;
- `MetocastClient` owns the HTTP client and typed endpoint methods;
- one WebSocket connection accepts core command types and emits core event types;
- reconnect, ping/pong, cancellation, and authentication failures have explicit states;
- public methods return named `thiserror` error enums, not `anyhow::Error` or stringly errors;
- callbacks/streams never expose Axum WebSocket frames or raw `serde_json::Value` to Swift.

Reuse the already-present `reqwest`, Tokio, Serde, and tracing stack. Configure TLS so Apple targets
do not require OpenSSL. Add only the smallest WebSocket client dependency needed after an Apple
cross-compile spike; do not introduce a second HTTP stack.

The first usable vertical slice is: health check, authentication failure, events list/get, connector
status snapshot, presenter status, and one presenter control command. Expand by native-app use case,
not by generating wrappers for every route before any app consumes them.

### 3. `crates/metocast-server`

A Tauri-free library plus the existing `metocast-server` binary target:

- Axum router and auth middleware;
- PostgreSQL pool, migrations, SQL repositories, and embedded PostgreSQL lifecycle;
- schedulers, queues, upload workers, Bible upstream access, concrete connectors, and presentation
  processing;
- a typed `ServerOptions` and lifecycle handle with graceful shutdown;
- the current headless binary as a thin configuration/logging adapter.

Use direct repository modules backed by `PgPool`; there is only one database implementation, so a
generic repository trait would add ceremony without reuse. Split the large route and WebSocket files
by feature only while code is being moved: events, recordings, connectors, presenter, and jobs. Keep
router composition and shared state in one obvious place.

The library must not import Tauri. Replace the current `Option<tauri::AppHandle>` with one narrow host
boundary backed by real implementations:

- a Tauri host adapter for status emission, embedded assets, host settings, and application logs;
- a headless adapter for filesystem assets/logs and no UI events;
- optionally, a native macOS host adapter later.

Prefer existing Tokio broadcast channels for connector status inside the server. The host adapter
should subscribe at the edge; connector implementations should not know that Tauri exists. Define
only capabilities currently used. Missing capabilities return a typed “unavailable” result rather
than being inferred from a null handle.

Expose lifecycle explicitly: `start` returns a handle containing the bound address and a graceful
shutdown operation; the headless binary awaits it, while Tauri/native macOS stores it. Use Tokio
channels already in the dependency graph rather than adding a lifecycle framework.

### 4. `crates/metocast-apple`

The only FFI crate, built as `staticlib` for an XCFramework:

- use UniFFI-generated Swift bindings instead of handwritten C ABI functions;
- expose task-oriented client objects, core records/enums, callbacks, and typed error codes;
- own one async runtime per client/library instance, never one runtime per method call;
- make cancellation and callback threading explicit and compatible with Swift concurrency;
- redact tokens and secrets from `Debug`, display strings, errors, and logs;
- contain no business rules that belong in core or client.

All handwritten reusable Rust remains `#![forbid(unsafe_code)]`. Any unsafe code required by generated
FFI scaffolding is confined to this crate/build output and is not copied into application code.

Before committing to the complete bridge, build a small spike for device iOS, Apple Silicon simulator,
and macOS. Verify it against the supported Xcode/Swift versions and deployment targets. UniFFI's
official guide documents static-library/Xcode integration, but its generated module maps have had
recent Apple-toolchain compatibility reports; pin the tested UniFFI version and keep a Swift import
smoke test in CI. References:
[UniFFI Xcode integration](https://mozilla.github.io/uniffi-rs/latest/swift/xcode.html) and
[UniFFI Swift module guidance](https://github.com/mozilla/uniffi-rs/blob/main/docs/manual/src/swift/module.md).

## Workspace and toolchain policy

Create a root `Cargo.toml` workspace with resolver 3 and these members:

- `src-tauri` (the thin Tauri host, retaining its current package identity initially);
- `crates/metocast-core`;
- `crates/metocast-client`;
- `crates/metocast-server`;
- `crates/metocast-apple`;
- existing first-party Rust crates such as `blackmagic-camera`, `rodecaster`, and
  `presenter-receiver` where workspace membership does not break their release process.

Exclude vendored third-party code from workspace lint inheritance. Consolidate first-party dependency
versions under `[workspace.dependencies]`, package metadata under `[workspace.package]`, and lints
under `[workspace.lints]`. A single root `Cargo.lock` becomes authoritative for applications and
binaries; remove subordinate first-party lockfiles only after the workspace builds and release jobs
have been updated.

Add `rust-toolchain.toml`:

```toml
[toolchain]
channel = "1.98.1"
profile = "minimal"
components = ["clippy", "rustfmt"]
```

Set `rust-version = "1.98"` and `edition = "2024"` through workspace package metadata. Upgrade in
two reviewable steps: first compile the current edition on Rust 1.98.1, then run the edition migration
and review it before changing manifests to 2024. Do not mix dependency upgrades into either step
unless Rust 1.98.1 requires one.

Pin CI to the same toolchain instead of `stable`, update Rust cache paths from `./src-tauri -> target`
to the root workspace target, and let platform jobs add only their required targets. This makes local,
CI, release, and XCFramework builds reproducible.

Use workspace lints rather than per-crate drift:

- `unsafe_code = "forbid"` for all non-FFI first-party crates;
- `unused_must_use = "deny"`;
- Clippy `all = "warn"`, enforced as errors in CI with `-D warnings`;
- no blanket `allow` attributes to silence migration warnings.

Do not enable the entire Clippy `pedantic` group globally: it creates churn without improving the
important boundaries. Enable individual additional lints only when they prevent a demonstrated class
of bug.

## Strict type and API rules

1. Every external boundary is decoded once into a named type: HTTP, WebSocket, database row, stored
   setting, environment variable, and FFI input.
2. Reuse `Uuid`, `DateTime<Utc>`, enums, and bounded integer types internally. Parse strings only at
   the boundary. Add newtypes when values are easy to confuse or security-sensitive, not for every
   field.
3. Core request and command enums use exhaustive matching. Unknown inbound WebSocket messages produce
   a typed protocol error; forward-compatible payload fields remain explicitly documented.
   Wire enums are platform-invariant: for example, an iOS client must be able to represent a Keynote
   command sent to a macOS server. Runtime capabilities, not `cfg`-removed protocol variants, decide
   whether an operation is supported.
4. Public library APIs use domain-specific errors. `anyhow` is allowed only in binaries and top-level
   orchestration where errors are logged and the process exits.
5. Secrets use redacted wrappers and never implement plaintext `Debug` or `Display`.
6. FFI methods expose typed records/enums and error codes. Do not use “JSON string in, JSON string out”
   as the public Swift API.
7. SQLx derives and database column names stay in server-private row structs. Mapping to core DTOs is
   explicit and tested.
8. Preserve the existing camelCase JSON contract. Moving a type between crates must not rename fields,
   tags, variants, or nullability.

## Migration plan

Each phase must leave the desktop app and headless server buildable. Keep moves separate from behavior
changes so review can distinguish relocation from redesign.

### Phase 0 - Freeze behavior with characterization tests

1. Record baseline commands and versions in the plan implementation PR.
2. Add small Serde compatibility tests for representative REST bodies, WebSocket commands, server
   events, connector statuses, and error envelopes. Use ordinary JSON fixtures/assertions; do not add
   a snapshot-test framework.
3. Save the generated OpenAPI document and compare it semantically before and after extraction.
4. Confirm the existing auth E2E suite covers public/authenticated route placement, secret redaction,
   loopback admin access, and unauthenticated WebSocket restrictions.
5. Record the supported iOS, macOS, Xcode, and Swift minimums before producing an XCFramework; do not
   silently inherit the build machine's newest deployment target.

Exit: there is a cheap test that fails if the refactor changes the public wire format or access rules.

### Phase 1 - Pin Rust and create the workspace

1. Add `rust-toolchain.toml` for Rust 1.98.1 and compile the unchanged code on it.
2. Add the root workspace manifest, shared dependency versions, package metadata, and lints.
3. Move to one root target directory and root lockfile; update pnpm scripts, CI caches, build commands,
   and release/version scripts.
4. Migrate first-party crates to edition 2024 one at a time with `cargo fix --edition`, review the
   diff, then format and lint.
5. Keep vendored `tauri-plugin-liquid-glass` independent.

Exit: current Tauri and headless binaries behave unchanged on Rust 1.98.1; all first-party crates use
the root workspace and CI no longer floats on `stable`.

### Phase 2 - Extract `metocast-core`

1. Move connector config/status types and their pure validation from `connectors/mod.rs`.
2. Split public DTOs from SQL row structs in `models/*`; move only the public DTOs and pure rules.
3. Move public Bible and presenter content types, leaving upstream HTTP/filesystem parsing in server
   code unless it is demonstrably pure and needed by Apple.
4. Move WebSocket command/event envelopes out of the Axum handler. Replace ad hoc outgoing `json!`
   payloads with the shared event enum feature by feature while preserving serialized JSON.
5. Move REST request/response records used by both server and client.
6. Update the server to consume core types and run the compatibility fixtures after each feature move.

Exit: `cargo tree -p metocast-core` contains no runtime, database, web framework, Tauri, or hardware
dependency, and it checks for both iOS and macOS targets.

### Phase 3 - Extract the Tauri-free `metocast-server`

1. Move database, models/repositories, Axum server, scheduler, queue, uploader, Bible upstream client,
   presentation processing, and concrete connectors into the server crate.
2. Replace `tauri::async_runtime::spawn` with Tokio spawning inside server code.
3. Route connector status through existing Tokio channels; implement Tauri event emission as a
   subscriber in the host crate.
4. Move legacy Tauri-store import, app-data-directory lookup, plugin setup, native dialogs, updater,
   and Tauri commands back to the Tauri edge.
5. Put caption logo settings and application logging behind narrow host capabilities or server-owned
   persisted settings. Prefer server-owned settings where remote/headless behavior should be identical.
6. Separate router construction from socket binding and add graceful lifecycle control.
7. Move the headless binary into the server package and update `pnpm build:server`, CI, and docs.
8. Split oversized handlers by feature during the move, without introducing generic layers.

Exit: `rg 'tauri(::|_)' crates/metocast-server` returns no application dependency; Tauri and headless
start the same server library, and all current E2E tests pass.

### Phase 4 - Build the typed Apple-capable client

1. Implement validated client configuration and redacted credentials.
2. Add the first HTTP vertical slice using core request/response types.
3. Add one WebSocket transport with typed commands/events, reconnect state, cancellation, and bounded
   buffering/backpressure behavior.
4. Add contract tests against the real `metocast-server`, including invalid JSON, auth rejection,
   disconnect/reconnect, ping/pong, and unknown messages.
5. Check `metocast-client` for device iOS, Apple Silicon simulator, and both macOS architectures.
6. Expand endpoint coverage only as native application features require it.

Exit: a Rust integration test can start the real server, connect with `MetocastClient`, list events,
observe connector/presenter state, send a presenter command, and shut down cleanly.

### Phase 5 - Add the Apple bridge and package

1. Prove the UniFFI toolchain with a tiny `health()`/version API on all required Apple targets.
2. Expose the phase-4 client slice as typed Swift async methods, event callbacks/streams, and typed
   errors. Keep transport internals out of the Swift API.
3. Build device-iOS, Apple-Silicon-simulator, and macOS static libraries with consistent deployment
   targets; create one XCFramework and a Swift Package wrapper.
4. Add a minimal Swift test target that imports the package, constructs a client, handles a typed
   auth error, and decodes at least one event.
5. Verify Swift 6 concurrency warnings are zero and callbacks do not retain the client after shutdown.
6. Document artifact versioning: the Swift package version follows the compatible Metocast protocol
   version, while the server continues using the synchronized application release version.

Exit: a clean native iOS/macOS sample target imports the binary package without local header edits or
unchecked casts and completes the first vertical slice against a running server.

### Phase 6 - Remove compatibility scaffolding and document ownership

1. Remove temporary re-exports and old modules only after all consumers import the new crates.
2. Update the architecture section in `README.md` and `ui/README.md`, plus headless/native build docs.
3. Document which crate owns every public contract and how to add an endpoint/event without drift.
4. Add a CI path filter/matrix that runs core/client checks for Apple-related changes and full server
   E2E for server/protocol changes.
5. Confirm no secret field or admin capability entered core/client/Apple response types.

Exit: the old backend modules no longer exist under `src-tauri`; ownership and dependency direction
are obvious from manifests and enforced by compilation.

## Verification commands

The implementation may adjust exact package names, but the completed workspace must provide
equivalent checks with zero warnings and zero errors:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo check -p metocast-core --target aarch64-apple-ios
cargo check -p metocast-core --target aarch64-apple-ios-sim
cargo check -p metocast-client --target aarch64-apple-ios
cargo check -p metocast-client --target aarch64-apple-ios-sim
cargo check -p metocast-client --target aarch64-apple-darwin
cargo check -p metocast-client --target x86_64-apple-darwin
pnpm check
pnpm lint
pnpm test:e2e
```

The XCFramework job additionally builds and tests the Swift Package with the repository's supported
Xcode version. Hardware-specific connector tests may remain target-gated, but the server crate must
still compile on every desktop CI platform currently shipped.

## Acceptance criteria

- Rust 1.98.1 is pinned locally and in CI; all first-party manifests declare Rust 1.98 and edition
  2024 after the staged migration.
- There is one first-party workspace lockfile and one shared target directory.
- `metocast-core` compiles for iOS/macOS and has no Tauri, Tokio, HTTP, SQL, or hardware dependencies.
- `metocast-client` compiles for iOS/macOS and exposes typed requests, events, and errors.
- `metocast-server` contains no Tauri import and is started by both the Tauri host and headless binary.
- Tauri-specific code is limited to plugin/command setup and concrete host capabilities.
- A Swift test imports the generated package and exercises the initial client vertical slice.
- Existing HTTP and WebSocket serialization, auth tiers, OpenAPI, Bruno examples, TypeScript Zod
  schemas, presenter receiver, Companion behavior, and server/client mode remain synchronized.
- No upstream credential becomes readable through core/client/FFI APIs or logs.
- All verification commands finish with zero warnings and zero errors.

## Risks and controls

| Risk | Control |
|---|---|
| A large move hides behavior changes | Move one feature at a time; keep relocation and redesign in separate commits; compare JSON/OpenAPI fixtures. |
| Workspace conversion breaks release scripts/caches | Convert scripts and CI in the same phase; build every current release target before removing old lockfiles. |
| Tauri remains coupled through a renamed wrapper | Enforce a source scan and dependency-tree check; host adapter uses project-owned types only. |
| Apple TLS/WebSocket dependency does not cross-compile | Prove the client dependency set on device and simulator before implementing full endpoint coverage. |
| UniFFI/Xcode module-map incompatibility | Pin the tested versions and run a real Swift import test; stop after the spike if the current toolchain is not clean. |
| FFI expands into a second business layer | Bridge task-oriented core/client APIs only; no decisions or persistence in `metocast-apple`. |
| Swift callbacks leak or violate concurrency rules | Explicit shutdown/cancellation, weak ownership in the sample, and Swift 6 warning-free tests. |
| Strict linting creates a cleanup mega-commit | Apply workspace lints crate by crate and keep `-D warnings` in CI; do not enable blanket pedantic linting. |

## Deliberate simplifications

- Four new crates are enough. Do not split connectors or repositories until a second independent
  consumer needs them.
- Keep one server state and direct SQLx repositories. A port-and-adapter framework is not required to
  remove Tauri.
- Ship one native-client vertical slice before pursuing full endpoint parity.
- Use the existing JSON protocol; typed shared enums and compatibility tests solve today’s drift
  without inventing a new binary protocol.
- Keep iOS client-only. Revisit an embedded iOS server only if a concrete offline product requirement
  justifies replacing PostgreSQL, hardware integrations, and long-running background services.
