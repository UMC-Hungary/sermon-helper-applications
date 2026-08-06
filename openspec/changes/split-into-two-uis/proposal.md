## Why

The previous change (`2026-08-06-design-system-and-sanctum-migration`, now archived) tried to deliver the new Sanctum design as a *theme* of the existing application — one codebase, two token packs, a settings switch. That approach is structurally wrong for what is wanted. A theme can only change colour, type, radius and elevation; it cannot change layout, navigation, information architecture or which screens exist. The result was one structure wearing two skins: switching to Sanctum recoloured the app but left its structure, and the new design's structure is precisely what makes it a different design.

The two designs should be two independent applications. The `ui-registry` capability already delivered by `decouple-core-from-rendering-ui` supports exactly this — `ui/registry.json`, build-time selection via `METOCAST_UI`, multi-UI staging under `build/ui/<id>/`, a launch chooser, and a settings selector — but only one UI was ever registered, and it lives at the repository root rather than in its own folder, so the registry has never had a second entry to prove itself against.

## What Changes

- **Restructure the repository into peer UIs.** The existing SvelteKit application moves from `/src` to `ui/classic/` as a self-contained application with its own `package.json`, build and configuration. A second application is created at `ui/sanctum/`. Neither is privileged by its location. **BREAKING** for every path-based reference: `svelte.config.js`, `vite.config.ts`, `tsconfig.json`, `eslint.config.js`, `tauri.conf.json`, the e2e suite and CI.
- **Extract a shared client package** at `packages/core-client/` from what is today `src/lib/{api,ws,host,schemas,types,utils,core-client}` plus `src/lib/locales/` — roughly 4,400 lines of framework-agnostic TypeScript and JSON. Both UIs depend on it. It is the single definition of every HTTP operation, every WebSocket message and every Zod schema; a backend message is defined once, not once per UI.
- **Build `ui/sanctum` as a new Svelte 5 application** covering **every screen and every feature** of the design reference at `~/workspace/ui/sermon-helper-svelte` — Dashboard, Events, Event editor, Presentations, Settings and Connectors — with its own structure, navigation and layout, wired to the real backend over WebSocket and HTTP through the shared package. It replaces the reference's fixture data with live data; it does not reproduce the reference's phone-frame chrome or its prototype-only tweaks panel.
- **Consume the design system from `packages/design-system/`**, delivered by the prerequisite `sanctum-design-system` change. That change discards the previous component library — whose values were fitted to an invented scale rather than measured from the reference — and rebuilds it faithfully with Storybook as its catalog. This change does not build or relocate a design system; it depends on one.
- **Reduce theming to what a theme should be.** Sanctum ships one design with **light and dark** schemes and a Light/Dark/Auto control. The `classic` token pack is dropped — the classic *application* is the classic design now, so encoding its appearance as a theme of the new one is redundant. **BREAKING** for the cross-design theme selector introduced by the archived change.
- **Register both UIs** so `METOCAST_UI=classic,sanctum` produces a bundle containing both, with the existing chooser and settings selector switching between them.
- `ui/classic` is **frozen**: it moves as-is, keeps working, and stops receiving design work. Its route-by-route migration — the bulk of the archived change and the reason it was 33 tasks short — is abandoned rather than completed.

## Capabilities

### New Capabilities
- `ui-workspace-layout`: A repository layout in which every rendering UI is a self-contained application under `ui/`, with shared code living in an explicitly shared package, and the Tauri build wired to the registry rather than to one privileged UI.
- `shared-core-client`: A framework-agnostic package that is the single source of truth for reaching the core — HTTP operations, WebSocket client, Zod message schemas, optional desktop host adapter and translation catalogues — consumable by any UI regardless of framework.
- `sanctum-ui`: A second registered rendering UI implementing the Sanctum design's own structure and covering every screen and feature of the design reference against live backend data.

### Modified Capabilities
- `ui-registry`: The registry gains a second real entry, and the requirement that the default build bundles "the existing SvelteKit app" is restated in terms of a registry id rather than a path, now that no UI lives at the repository root.

## Impact

- **Repository layout**: new `ui/classic/`, `ui/sanctum/`, `packages/core-client/`; `/src` and root-level frontend configuration removed. Package manager workspace configuration added (`pnpm-workspace.yaml`).
- **Build**: `scripts/build-ui.mjs` and `ui/registry.json` gain the second entry and per-UI build commands; `tauri.conf.json` `beforeBuildCommand`/`frontendDist`/`devUrl` re-pointed; `scripts/build-tokens.mjs`, `check-tokens.mjs`, `check-contrast.mjs` and `check-ds.mjs` move under `ui/sanctum` and stay in CI.
- **Shared package**: the ESLint rule confining `fetch`, `WebSocket` and `@tauri-apps/*` to the client SDK is re-scoped so it applies to both UIs and permits those APIs only inside `packages/core-client`.
- **Tests**: `e2e/` paths and `vitest.config.ts` updated; the archived change's classic-parity and cross-theme visual checks are dropped along with the classic token pack.
- **Not affected**: the Rust core, HTTP/WS contracts, OpenAPI, Bruno collections, Companion, `presenter-receiver`, and the `/presenter` and `/caption` projection surfaces, which remain excluded from design work.
- **Depends on**: `sanctum-design-system`, which must be complete before group 5. It supplies `packages/design-system/` and removes `src/lib/ds/`.
- **Abandoned from the archived change**: the `classic` theme pack, the cross-design appearance selector, the route-by-route restyling of the existing application's 41 remaining surfaces, and the component library itself, which the prerequisite change rebuilds.
