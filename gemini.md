# gemini.md

This file provides guidance to Gemini when working with code in this repository.

## Project Overview

Church livestream control desktop application built with **Tauri 2 + SvelteKit 5 + TypeScript**. The application provides a unified interface to control various aspects of a church service's live stream, including OBS, presentations, and potentially other devices.

## Commands

```bash
pnpm dev              # Vite dev server only (port 1420)
pnpm tauri dev        # Full Tauri desktop app in dev mode
pnpm tauri build      # Production build
pnpm check            # TypeScript + Svelte type checking
pnpm check:watch      # Watch mode for type checking
```

## Architecture

**Tech Stack:** SvelteKit 5 + Svelte 5 + Tauri 2 + Tailwind CSS 4 + TypeScript

**Frontend (`/src`):**
- `routes/` - SvelteKit file-based routing (SSG via adapter-static).
- `lib/components/` - Reusable UI components for the application.
- `lib/stores/` - Svelte stores for managing application state, such as `system-store` and `obs-devices-store`.
- `lib/services/` - Higher-level services that compose functionality from utils and stores.
- `lib/utils/` - Utility functions for specific tasks like OBS control (`obs-websocket.ts`), PowerPoint generation (`pptx-generator.ts`), and user notifications (`toast.ts`).

**Backend (`/src-tauri`):**
- A significant Rust backend that provides core application functionality. It's not just a thin wrapper around a webview.
- **Web Server:** An `axum` based web server is running to handle local API requests.
- **Network Discovery:** An mDNS service (`mdns-sd`) is used to discover other devices on the network.
- **Device Control:** It integrates with Broadlink (`rbroadlink`) devices for RF/IR remote control.
- **Video Upload:** Contains logic for video uploading.
- **Tauri Plugins:** It leverages several Tauri plugins for functionalities like `store` (persistent storage), `websocket`, `opener`, `dialog`, `fs`, `log`, `process`, `updater`, and `deep-link`.

**Key Integrations:**
- **OBS:** `obs-websocket-js` is used for direct control over OBS via its WebSocket interface.
- **PowerPoint:** `pptxgenjs` is used to generate `.pptx` files.
- **Internationalization:** `svelte-i18n` is used for providing multi-language support.
- **Notifications:** `svelte-sonner` is used for displaying toast notifications.
- **Broadlink:** The Rust backend integrates with Broadlink devices, allowing the application to send RF and IR commands.

## Key Patterns

**State Management:** Svelte stores (writable and derived) are the primary means of state management in the frontend.
**Styling:** Tailwind CSS 4 is used for styling, with `clsx` and `tailwind-merge` for combining CSS classes.
**Backend Communication:** The frontend communicates with the Rust backend through Tauri's command and event system.

## Implementation Status

**Complete:**
- Core UI framework and component library.
- OBS WebSocket integration for scene and source control.
- Toast notification system.
- Internationalization setup.
- Event management and scheduling.

**In Progress:**
- Bible verse integration and presentation generation.
- Broadlink device integration for RF/IR control.
- YouTube integration for stream scheduling and control.
- Full implementation of the video upload process.

## Type Definitions

Multiple `SystemStatus` definitions exist:
- `src/lib/stores/types.ts` - Used by the `system-store`.
- `src/lib/types.ts` - A more detailed structure that seems to be a target for future consolidation.

It is recommended to consolidate these types to have a single source of truth for the application's status.

## Planning Rules
- Always name plan files with the format: PLAN-{feature-name}.md under the `plans` folder.
