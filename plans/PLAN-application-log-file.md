# PLAN-application-log-file

## Goal

Give operators a simple way to inspect the real application log from inside Metocast.

The app should write durable logs to a known file, expose a button such as **Open Application Log**, and provide a lightweight in-app log view with only level filtering: all, info, warning, and error.

## Problem

On a packaged macOS install, server-mode startup can fail before Axum, PostgreSQL, or the app WebSocket becomes usable. Rust already emits useful `tracing` messages, including:

- embedded PostgreSQL startup
- pool connection
- migrations
- Axum startup
- `Backend startup failed: {e}`

But those messages are not easy for a non-developer to find. The immediate need is not a full developer console. The need is: "show me the log file, and let me quickly separate info/warnings/errors."

## MVP

1. Write Rust `tracing` output to a persistent log file.
2. Add a Tauri command that returns the log file path.
3. Add a Tauri command that reads the current log file contents.
4. Add a Tauri command or frontend action that opens the file with `tauri-plugin-opener`.
5. Add a small in-app log screen or settings panel with level filters only.
6. Add a button labeled **Open Application Log** to open the raw file externally.
7. If the log file does not exist yet, create it with a short header before opening or reading it.
8. Remove or redact obviously sensitive values from startup logs.

## Log File Location

Use the Tauri app log/data directory, for example:

```text
<app_data_dir>/logs/metocast.log
```

Keep the first version intentionally simple:

- one active log file
- append on app startup
- lightweight in-app viewer backed by the same file
- no database storage
- no REST or WebSocket API
- no live streaming required

Optional small rotation can be added if easy:

- rotate at 5 MB
- keep `metocast.1.log`

## Backend Plan

### 1. Add A Logging Helper

Add `src-tauri/src/logging.rs`.

Responsibilities:

- resolve `<app_data_dir>/logs/metocast.log`
- create the parent directory
- initialize `tracing_subscriber` with a file writer
- expose helper functions for log path/open behavior

### 2. Initialize File Logging Early

In `src-tauri/src/lib.rs`, replace the current `tracing_subscriber::fmt().with_env_filter(...).init()` setup with a setup that writes to the log file.

Keep `RUST_LOG` support:

```text
RUST_LOG=debug
```

should still increase verbosity during debugging.

### 3. Keep Startup Logs Useful

Before and after the main startup steps, keep or add logs:

- app setup started
- mode loaded
- server mode detected
- starting embedded PostgreSQL
- connecting database pool
- running migrations
- starting Axum
- backend startup failed
- backend started successfully

Important: do not rely on the server being alive to read these logs.

### 4. Redact Sensitive Startup Values

At minimum, stop logging the full PostgreSQL connection URL in `start_server()`.

Change:

```rust
tracing::info!("Connecting pool to {connection_url}");
```

to something like:

```rust
tracing::info!("Connecting pool to embedded PostgreSQL");
```

Also avoid logging auth tokens, OAuth secrets, OBS passwords, or bearer headers.

### 5. Add Tauri Commands

Add `src-tauri/src/commands/logs.rs`.

Commands:

```rust
get_application_log_path() -> Result<String, String>
read_application_log() -> Result<String, String>
open_application_log() -> Result<(), String>
```

`open_application_log()` can use the existing `tauri-plugin-opener` capability from Rust, or the frontend can call `get_application_log_path()` and open it with the existing opener plugin. Prefer the Rust command if it creates the file before opening.

`read_application_log()` should read the same file shown by **Open Application Log**. If the file grows large, it can return only the last 1-2 MB in the MVP.

Register the commands for desktop in `src-tauri/src/lib.rs`.

## Frontend Plan

### Log File UI

Add either:

- `src/routes/logs/+page.svelte`, linked from the sidebar near Settings, or
- a compact diagnostics panel inside Settings.

Prefer `/logs` if the UI needs enough room to be readable.

Controls:

- segmented filter: **All**, **Info**, **Warnings**, **Errors**
- **Refresh**
- **Open Application Log**
- optional **Copy Log Path**

Behavior:

- load raw text from `read_application_log`
- split into lines
- classify each line by level using the stable log format
- filter locally in Svelte
- click **Open Application Log** to open the raw file externally
- show a toast on open/copy failure

Display:

- monospace log lines
- timestamp/level/target/message can remain as plain text
- warning and error lines can have subtle color accents
- empty states for "no logs yet" and "no matching lines"
- no structured detail drawer, search syntax, export bundle, live tailing, or source filters in the MVP

### Log Format For Filtering

Use a stable human-readable line format that the UI can classify without a structured logging database:

```text
2026-05-30T12:34:56.789Z INFO metocast_lib: Starting embedded PostgreSQL
2026-05-30T12:34:57.102Z WARN metocast_lib::database::embedded: PG start failed ...
2026-05-30T12:34:57.450Z ERROR metocast_lib: Backend startup failed: ...
```

The frontend can classify lines containing the level token after the timestamp:

- `INFO`
- `WARN`
- `ERROR`

If JSONL is easier for Rust, keep the external file readable enough for a human. Do not make the operator open a file full of opaque objects unless that is clearly better after implementation.

### Dashboard Shortcut

Optional but useful:

If `$appMode === 'server'` and WebSocket status is `error` or remains disconnected after startup, show a compact troubleshooting action:

```text
Server is not reachable. View Logs
```

This should not require a full backend status system in the first version.

### Internationalization

Update:

- `src/lib/locales/en.json`
- `src/lib/locales/hu.json`

Add strings for:

- diagnostics section title
- logs nav label
- level filters
- refresh button
- open log button
- copy path button
- success/error toast

## Testing Plan

1. Run `pnpm check`.
2. Run `pnpm tauri dev`.
3. Open the Logs UI.
4. Confirm the raw log lines appear.
5. Confirm All, Info, Warnings, and Errors filters work.
6. Click **Open Application Log**.
7. Confirm the same file opens externally and contains startup logs.
8. Configure server mode and restart.
9. Force a startup failure, for example by occupying the configured port.
10. Confirm the final log line includes `Backend startup failed`.
11. Confirm the log does not include auth tokens, OAuth secrets, OBS passwords, or the raw PostgreSQL URL.

## Future Enhancements

- Add **Reveal in Finder**.
- Add **Copy Log Path** if the opener fails.
- Add small log rotation.
- Add search or live tailing later if level filtering is not enough.
- Add a one-click support bundle with app version, OS version, mode, and connector statuses.
