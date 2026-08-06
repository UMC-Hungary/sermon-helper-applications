# Metocast

Church livestream control desktop application built with Tauri 2 + SvelteKit 5 + TypeScript.

## Rendering UIs

The front-end is swappable. A rendering UI is a static bundle that talks to the core through
one SDK (`$lib/core-client`) and never touches Tauri, `fetch` or `WebSocket` directly, so the
same UI runs in the desktop app, a browser, or a remote client-mode window.

```bash
pnpm build                            # the registry's default UI
METOCAST_UI=my-ui pnpm build          # a different registered UI
METOCAST_UI=default,my-ui pnpm build  # both, with a chooser in Settings
```

Registered UIs live in [`ui/registry.json`](ui/registry.json); [`ui/README.md`](ui/README.md) is
the contract for writing one.

## API access and secrets

The core exposes one HTTP/WebSocket API, used by the desktop app, remote client-mode UIs,
Companion and the presenter receiver alike. Access has three tiers:

| Tier | Who | What they get |
|---|---|---|
| **No credentials** | Anyone who can reach the port | `GET /health`, `/openapi.json`, `/docs`, the OBS caption overlay (`/caption`), the OAuth callback, and the UI bundle. A WebSocket may connect without a token but is limited to the read-only presenter commands. |
| **Auth token** | The desktop app, client-mode UIs, Companion | Everything under `/api/*` and full WebSocket access, via `Authorization: Bearer <token>` (or `?token=` for `/ws`). The token is shown on the Connect page. |
| **Auth token + admin token, on loopback** | The desktop app hosting the server, in server mode | Reading stored upstream credentials back. |
| **Nobody** | — | Everything else about credentials. See below. |

**Upstream credentials never leave the server.** The szentiras.eu API key, YouTube client
secret, Facebook app secret, OBS password and Discord webhook URL are stored server-side and
used only for the core's own outbound requests. `GET /api/connectors/{name}/config` always
returns them blank, with a `<field>Set` boolean saying whether one is stored — holding the auth
token is not enough to read them. When saving, send a new value to replace a secret, leave it
blank to keep the stored one, or send `"<field>Set": false` to clear it.

The one exception is the machine actually running the server: its own desktop app can re-read a
credential it stored, via `GET /api/connectors/{name}/config/secrets`. That needs a second
admin token — regenerated every run, kept in memory, handed to the host window over Tauri IPC and
never over the network — *and* the request must arrive on loopback. A client-mode window is
talking to someone else's core, so it never gets one, and the **Show** action stays hidden.

Full contract, including what is deliberately public and what is still open (per-device keys,
encryption at rest, TLS): [plans/PLAN-api-access-contracts.md](plans/PLAN-api-access-contracts.md).

## Presenter Receiver

A standalone binary that connects to the Metocast server over WebSocket and renders slides directly on a display — no browser required. Designed for Raspberry Pi / Linux framebuffer setups or macOS secondary screens.

See [presenter-receiver/PRESENTER_RECEIVER.md](presenter-receiver/PRESENTER_RECEIVER.md) for full documentation: installation, auto-start on boot, supported platforms, update instructions, and WebSocket protocol reference.

### Quick start

```bash
# Install
curl -fsSL https://raw.githubusercontent.com/UMC-Hungary/sermon-helper-applications/main/presenter-receiver/install.sh | bash -s -- ws://YOUR_SERVER_IP:3737/ws

# Or with authentication token
presenter-receiver ws://192.168.1.10:3737/ws --token YOUR_TOKEN

# Local development against the Tauri backend
cargo run --manifest-path presenter-receiver/Cargo.toml --bin presenter-receiver -- \
  ws://127.0.0.1:3737/ws --token YOUR_TOKEN
```

## Companion Module

The Companion module communicates with Metocast only through the app WebSocket. Its Textus and Lekcio presets call `presenter.load_bible_reference` without an `event_id`, so the backend-selected current/next event is used.

## Development

```bash
pnpm dev              # Vite dev server only (port 1420)
pnpm tauri dev        # Full Tauri desktop app in dev mode
pnpm tauri build      # Production build
pnpm check            # TypeScript + Svelte type checking
```
