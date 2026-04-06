# PLAN: Presenter Anonymous (Read-Only) Access

## Problem

External browsers on the local network need to open `/presenter` and connect to the
WebSocket without knowing the auth token — they are **display-only** consumers.
Currently, `/ws` rejects any connection whose `?token=` does not match the server
token, making tokenless access impossible.

The requirement: **GET requests are allowed without a token, in read-only mode.**
All mutating operations still require a valid token.

---

## Current Auth Architecture

| Layer | Mechanism | Protects |
|---|---|---|
| HTTP REST | `auth_middleware` — requires `Authorization: Bearer <token>` header | All `/api/*` routes |
| WebSocket | `ws_handler` validates `?token=` query param; rejects 401 if wrong | `/ws` upgrade |
| Static files | None — `ServeDir` is public | SvelteKit pages (HTML/JS/CSS) |

The presenter SvelteKit page (`/presenter`) is already a public static file.
Only the WebSocket connection and the `POST /api/presenter/parse` REST endpoint
need changes.

---

## Design: Optional-Auth WebSocket

Make the token **optional** on the WS upgrade. The connection is tagged as either
`Authenticated` or `ReadOnly`. The same `/ws` endpoint is reused — no new endpoint
needed.

### Authentication levels

| Level | How established | Allowed WS commands |
|---|---|---|
| `Authenticated` | Valid `?token=<token>` provided | All commands |
| `ReadOnly` | No token, or token omitted | Presenter read-only set only |

### Read-only command allowlist (no token required)

- `presenter.register` — sets the client's display label (harmless)
- `presenter.status` — queries current presenter state (read)
- `pong` — responds to a server-initiated ping (harmless)

Every other command returns `{ "type": "error", "message": "unauthorized" }`.

### Broadcasts

All push messages (including `presenter.state`, `presenter.slide_changed`,
`clients.updated`, `ping`) are sent to **all** connected clients regardless of
auth level. The LAN presenter displays need to receive these in real time.

---

## Changes Required

### 1. `src-tauri/src/server/websocket.rs`

**`WsQuery` struct** — make token optional:
```rust
pub struct WsQuery {
    token: Option<String>,
}
```

**`ws_handler`** — derive `is_authenticated` from the optional token:
```rust
let current_token = state.auth_token.read().await.clone();
let is_authenticated = query.token.as_deref() == Some(current_token.as_str());
// No early 401 return — proceed to upgrade regardless
ws.on_upgrade(move |socket| handle_socket(socket, state, server_id, user_agent, is_authenticated))
```

**`handle_socket`** — accept `is_authenticated: bool`, thread it through to the
command handler:
```rust
async fn handle_socket(socket: WebSocket, state: AppState, server_id: String,
    user_agent: Option<String>, is_authenticated: bool)
```

**`handle_ws_command`** — add `is_authenticated: bool` parameter. At the top of the
function, before dispatching, check:
```rust
const READONLY_COMMANDS: &[&str] = &["presenter.register", "presenter.status", "pong"];
// The serde tag is the `type` field — check it before deserialising by
// peeking at the raw JSON, or enforce by matching the enum variants.
```

Because the enum is `#[serde(tag = "type")]`, the simplest approach is to add a guard
**before** full deserialisation — deserialise only the `type` field first, then
check `is_authenticated` or membership in the allowlist:

```rust
// In the message-receive loop, before calling handle_ws_command:
if !is_authenticated {
    // Peek at the type field
    let type_only: Option<serde_json::Value> = serde_json::from_str(&text).ok();
    let cmd_type = type_only
        .as_ref()
        .and_then(|v| v.get("type"))
        .and_then(|t| t.as_str())
        .unwrap_or("");
    const ALLOWED: &[&str] = &["presenter.register", "presenter.status", "pong"];
    if !ALLOWED.contains(&cmd_type) {
        let _ = client_tx.send(Message::Text(
            r#"{"type":"error","message":"unauthorized"}"#.into()
        ));
        continue; // skip to next message
    }
}
```

This keeps the check at the receive loop, before any actual dispatch, without
touching every individual command handler.

### 2. `src-tauri/src/server/auth.rs` — no change

`auth_middleware` already protects all `/api/*` routes including
`POST /api/presenter/parse`. That endpoint stays token-protected because it is a
write/compute operation.

### 3. `src/routes/presenter/+page.svelte` — frontend standalone WS

Currently, the standalone connection reads the token from `?token=` URL params and
connects with `ws://${host}/ws?token=${token}`.

**Change:** if no `?token=` param is present, connect without it (tokenless = read-only).
The presenter display doesn't need to send any authenticated commands.

```typescript
function connectStandalone(token: string | null) {
    const wsUrl = token
        ? `ws://${window.location.host}/ws?token=${encodeURIComponent(token)}`
        : `ws://${window.location.host}/ws`;
    // …rest unchanged
}

onMount(() => {
    const tokenParam = $page.url.searchParams.get('token');
    isStandalone = true;
    connectStandalone(tokenParam);  // null → no token → read-only
    // ...
});
```

This means the page always uses a standalone connection (instead of checking whether
a token is present to decide between standalone vs in-app mode). The in-app iframe
benefits from this too — it passes a token so it stays authenticated.

> **Note on in-app mode:** The main Tauri app WS (in `ws/client.ts`) is already
> authenticated via its own connection. The `/presenter` page only needs the
> standalone path because it runs in an iframe or external browser.

### 4. Presenter page — always standalone

Since `/presenter` is always loaded either:
- In an external browser (no Tauri app context), or
- In an iframe inside the Tauri app

…it never has access to the main app's Svelte stores. The current "in-app vs standalone"
detection (checking for `?token=`) can be simplified: **always use a standalone
connection on the presenter page**. This removes the branching on `isStandalone`.

---

## What stays the same

- All `/api/*` routes remain token-protected (no change to `auth_middleware`)
- The main Tauri WS client (`ws/client.ts`) still sends `?token=` and is fully authenticated
- Navigation commands (`presenter.next`, `presenter.prev`, etc.) from external
  browsers are rejected with `error: unauthorized` — they can only watch
- The `presentations/+page.svelte` management UI (Tauri app) retains full control

---

## Security considerations

- The server binds to `0.0.0.0` so any device on the LAN can reach `/ws`
- Read-only access exposes: current presenter slide state, WS client list (via
  `clients.updated`), and all other broadcast push messages (connector statuses,
  upload progress, etc.)
- This is acceptable for a church LAN — no personal data in broadcasts
- If tighter isolation is needed in the future, a separate `/ws/presenter` endpoint
  could be added that only forwards presenter-related messages

---

## Implementation order

1. `websocket.rs` — optional token + `is_authenticated` flag + command guard
2. `presenter/+page.svelte` — always-standalone WS, token optional
3. `pnpm check` + `cargo build` — 0 errors, 0 warnings
