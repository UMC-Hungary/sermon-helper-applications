# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Church livestream control desktop application built with **Tauri 2 + SvelteKit 5 + TypeScript**.

## Commands

```bash
pnpm dev              # Vite dev server only (port 1420)
pnpm tauri dev        # Full Tauri desktop app in dev mode
pnpm tauri build      # Production build
pnpm check            # TypeScript + Svelte type checking
pnpm check:watch      # Watch mode for type checking
```

## Architecture

**Tech Stack:** SvelteKit 2.9 + Svelte 5 + Tauri 2 + Tailwind CSS 4 + TypeScript

**Frontend (`/src`):**
- `routes/` - SvelteKit file-based routing (SSG via adapter-static)
- `lib/components/ui/` - Reusable UI primitives (button, card, alert, dialog, etc.)
- `lib/components/sidebar.svelte` - Main navigation + system status display
- `lib/stores/` - Svelte stores for app state (`systemStore`, `obsStatus`)
- `lib/utils/` - Services: `obs-websocket.ts` (OBS control), `obs-store.ts` (settings persistence), `toast.ts`

**Backend (`/src-tauri`):**
- Minimal Rust - most logic in TypeScript frontend
- Uses Tauri plugins: store (persistence), websocket, opener

**Key Integrations:**
- `obs-websocket-js` - Direct OBS WebSocket client for streaming control
- `svelte-sonner` - Toast notifications
- 3-tier storage fallback: Tauri store → localStorage → in-memory

## Key Patterns

**State Management:** Svelte stores (writable/derived) in `/lib/stores/`, props for component communication

**Styling:** Tailwind CSS 4 with OKLch CSS variables, `cn()` utility for class merging (clsx + tailwind-merge)

**SystemStatus Type** (`src/lib/stores/types.ts`):
```typescript
type SystemStatus = {
  obs: boolean;
  rodeInterface: boolean;
  mainDisplay: boolean;
  secondaryDisplay: boolean;
  youtubeLoggedIn: boolean;
}
```

## Implementation Status

**Complete:** Sidebar, UI components, OBS WebSocket integration, toast system, error messages display, Service Events

**Stubs:** Bible editor

## Type Definitions

Two SystemStatus definitions exist:
- `src/lib/stores/types.ts` - Flat structure (correct)
- `src/lib/types.ts` - Nested structure (outdated, to be consolidated)

## Planning Rules
- Always name plan files with the format: PLAN-{feature-name}.md under the `plans` folder.