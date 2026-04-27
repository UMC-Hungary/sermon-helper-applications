# Sermon Helper Applications

Church livestream control desktop application built with Tauri 2 + SvelteKit 5 + TypeScript.

## Presenter Receiver

A standalone binary that connects to the Sermon Helper server over WebSocket and renders slides directly on a display — no browser required. Designed for Raspberry Pi / Linux framebuffer setups or macOS secondary screens.

See [presenter-receiver/PRESENTER_RECEIVER.md](presenter-receiver/PRESENTER_RECEIVER.md) for full documentation: installation, auto-start on boot, supported platforms, update instructions, and WebSocket protocol reference.

### Quick start

```bash
# Install
curl -fsSL https://raw.githubusercontent.com/UMC-Hungary/sermon-helper-applications/main/presenter-receiver/install.sh | bash -s -- ws://YOUR_SERVER_IP:3737/ws

# Or with authentication token
presenter-receiver ws://192.168.1.10:3737/ws --token YOUR_TOKEN
```

## Development

```bash
pnpm dev              # Vite dev server only (port 1420)
pnpm tauri dev        # Full Tauri desktop app in dev mode
pnpm tauri build      # Production build
pnpm check            # TypeScript + Svelte type checking
```
