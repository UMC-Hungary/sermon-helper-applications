# Writing a rendering UI

A rendering UI is a static front-end that talks to the Metocast core. The core does not know
or care which one is running: the desktop app, a browser tab, and a remote client-mode window
all speak the same HTTP/WebSocket API. This document is the contract a UI has to satisfy.

## The rules

1. **Reach the core only through the SDK.** Import from `$lib/core-client`. Never import
   `@tauri-apps/*`, never call `fetch` directly, never construct a `WebSocket`. ESLint enforces
   this for everything under `src/routes/` and `src/lib/components/`; the SDK itself is the one
   place those are allowed.
2. **Assume no desktop shell.** Anything Tauri-only is a *host capability*: which core the
   window talks to, log files, updates, native dialogs. Check `hostCapabilities` before
   offering it and hide or degrade the control when it is absent. A UI that throws outside the
   desktop app is broken, not "desktop-only".
3. **Validate at the boundary.** Every HTTP response and WebSocket message is parsed with a Zod
   schema, and the TypeScript types are `z.infer`red from those schemas. Do not hand-write an
   interface for something that crossed the wire.
4. **Build to static files.** No server-side rendering at runtime — the output is plain files
   that Tauri embeds and the core serves. The existing app uses `@sveltejs/adapter-static`.
5. **Change no Rust.** If a UI needs something the API does not expose, that is a core change
   with its own contract-surface updates (OpenAPI, Bruno, schemas), not a shortcut through IPC.

## What the SDK gives you

`$lib/core-client` re-exports three layers, so a UI has one import site:

| Layer | What is behind it |
|---|---|
| HTTP (`$lib/api`) | Events, recordings, uploads, queues, connector config/status, Bible lookups, presenter |
| WebSocket (`$lib/ws`) | `connectWs`, `sendWsCommand`, live events; `connectPresenterWs` for a standalone display |
| Host (`$lib/host`) | `hostCapabilities` plus the optional desktop-only calls, each feature-detected |

Transport is not a choice a UI makes. HTTP and WS always address the configured core — local in
server mode, remote in client mode — and the host layer covers what is not a core operation at
all.

## Registering a UI

Add an entry to [`registry.json`](registry.json):

```json
{
  "id": "my-ui",
  "displayName": "My UI",
  "description": "One line shown in the settings selector.",
  "buildCommand": "pnpm --filter my-ui build",
  "buildDir": "ui/my-ui/dist",
  "entry": "index.html"
}
```

| Field | Meaning |
|---|---|
| `id` | Stable identifier; used in `METOCAST_UI` and in the bundle path |
| `displayName` | Shown in the settings selector |
| `description` | Optional one-liner under the name |
| `buildCommand` | Run from the repo root; must produce static files |
| `buildDir` | Where that command writes them, relative to the repo root |
| `entry` | The HTML file to open, relative to `buildDir` |

## Building

`scripts/build-ui.mjs` runs as `pnpm build` (which `tauri build` invokes) and reads
`METOCAST_UI`:

```bash
pnpm build                          # the registry's default UI
METOCAST_UI=my-ui pnpm build        # one other registered UI
METOCAST_UI=default,my-ui pnpm build  # both, with a chooser in settings
```

With a single UI the output is staged at `build/` exactly as before — an ordinary build is
unchanged. With several, each lands in `build/ui/<id>/`, `build/bundled-uis.json` lists them,
and `build/index.html` becomes a small chooser page that sends the window to the UI selected in
**Settings → Rendering UI**. That selection is stored per machine and applies at the next start;
switching live is deliberately not supported.
