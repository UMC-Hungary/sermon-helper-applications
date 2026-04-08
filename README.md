# Sermon Helper Applications

Church livestream control desktop application built with Tauri 2 + SvelteKit 5 + TypeScript.

## Presenter Receiver

A lightweight terminal binary that connects to the Sermon Helper WebSocket server and displays slides directly — no browser required. Designed for Raspberry Pi / Linux framebuffer setups or macOS secondary screens.

### Install

```bash
curl -fsSL https://raw.githubusercontent.com/UMC-Hungary/sermon-helper-applications/main/presenter-receiver/install.sh | bash
```

### Install and run immediately

```bash
curl -fsSL https://raw.githubusercontent.com/UMC-Hungary/sermon-helper-applications/main/presenter-receiver/install.sh | bash -s -- ws://YOUR_SERVER_IP:3737/ws
```

### Usage

```bash
presenter-receiver ws://192.168.1.10:3737/ws
presenter-receiver ws://192.168.1.10:3737/ws --token <your-token>
```

**Supported platforms:** macOS arm64/x86_64, Linux arm64/x86_64 (including Raspberry Pi)

## Development

```bash
pnpm dev              # Vite dev server only (port 1420)
pnpm tauri dev        # Full Tauri desktop app in dev mode
pnpm tauri build      # Production build
pnpm check            # TypeScript + Svelte type checking
```
