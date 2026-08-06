## Context

`decouple-core-from-rendering-ui` (archived) delivered a headless core, an HTTP/WS-only UI boundary and a **UI registry** — `ui/registry.json`, `METOCAST_UI` build-time selection, multi-UI staging under `build/ui/<id>/`, a launch chooser, and `ActiveUiSettings`. All of that machinery works and is already in `scripts/build-ui.mjs`. It has simply never had a second UI to prove itself against, and the one registered UI lives at the repository root rather than in `ui/`.

`design-system-and-sanctum-migration` (archived, 88/121 tasks) then tried to deliver the new design as a *theme* of that single app: two token packs, a live switch, and a route-by-route restyle. It delivered real value — a DTCG token architecture, 39 accessible components with written specifications, a `/design` catalog, and CI checks for token drift, theme completeness, contrast and design-system coverage. But its central premise was wrong. Inspection of the result made this concrete: both themes have identical 60-token key sets, 58 values differ, and **no component branches on the theme at all**. A theme could swap `--font-display` to Cormorant Garamond, `--radius-card` to zero and `--shadow-card` to none — but it could not change that the app has four nav destinations, or which screens exist, or how a screen is laid out. Switching to Sanctum recoloured the application; it could never restructure it.

The state on disk now: `src/lib/ds/` holds 9,993 lines of design system; the classic app's shell, settings and notifications were rebuilt on it; 41 routes still carry their own ad-hoc styles. `src/lib/{api,ws,host,schemas,types,utils,core-client}` plus `src/lib/locales/` is ~4,400 lines of framework-agnostic TypeScript and JSON, already fenced off by an ESLint rule that confines `fetch`, `WebSocket` and `@tauri-apps/*` to the SDK.

Constraints: the Rust core, HTTP/WS contracts, OpenAPI, Bruno and Companion are untouched. `/presenter` and `/caption` remain projection output and stay excluded. Two locales, Hungarian frequently longer. Mobile is a real target — `src-tauri/icons/android/` ships adaptive icons and `Cargo.toml` already excludes desktop-only dependencies for `ios` and `android`.

## Goals / Non-Goals

**Goals:**
- Two rendering UIs that are genuinely independent in structure, navigation, screen inventory and styling.
- One definition of the core boundary — WebSocket client, HTTP operations, Zod schemas — shared by both, so a backend message is defined once.
- A Sanctum UI covering every screen and feature of the design reference, against live data.
- The existing application preserved and working, relocated without functional change.

**Non-Goals:**
- Building the design system. That is the `sanctum-design-system` prerequisite; this change consumes its package.
- Completing the archived change's route-by-route restyle of the existing app. It is abandoned; `ui/classic` is frozen.
- A `classic` token pack or a cross-design theme selector. The classic *application* is the classic design.
- Feature parity between the UIs. Sanctum covers the reference; it does not cover queues, logs, obs-devices, obs-caption, rf-ir, connect or setup.
- Sharing anything visual. No shared components, tokens, styling or stores.
- Changing route paths, the core, or the projection surfaces.

## Decisions

### Both UIs move under `ui/`; neither keeps the root

`ui/classic/` and `ui/sanctum/`, each a complete application with its own `package.json`, Vite and SvelteKit configuration, `tsconfig.json` and `static/`. Leaving the current app at `/src` would have been a far smaller diff, and it was tempting — but it encodes the wrong idea. A root-level UI is implicitly *the* UI and everything else is an add-on, which is exactly the asymmetry that produced the theme approach. Peers in `ui/` make the registry's model true on disk.

```
ui/
  registry.json
  classic/          package.json, svelte.config.js, src/, static/
  sanctum/          package.json, svelte.config.js, src/, static/
packages/
  core-client/      package.json, src/{api,ws,host,schemas,types,utils}, locales/
scripts/build-ui.mjs
src-tauri/
pnpm-workspace.yaml
```

The cost is mechanical and real: every path-based reference moves — `svelte.config.js`, `vite.config.ts`, `tsconfig.json`, `eslint.config.js`, `tauri.conf.json`'s `devUrl`/`beforeBuildCommand`/`frontendDist`, `vitest.config.ts` and `e2e/`. It is done once, in its own step, with the existing app's tests as the proof it was behaviour-preserving.

### The shared package is the transport boundary and nothing more

`packages/core-client/` takes `api/` (679 lines), `ws/` (337), `host/` (233), `schemas/` (709), `types/` (53), `utils/` (213), `core-client/` (107) and `locales/` (2,082 lines of JSON). All of it is plain TypeScript and JSON with no Svelte import today, so the extraction is a move plus a `package.json`, not a rewrite.

The line is drawn at *framework-agnostic*: if it can be consumed by React as easily as Svelte, it is shareable. That excludes `stores/` (Svelte stores — each UI writes its own), `ds/`, `components/` and `routes/`. This keeps the promise that a future UI could be written in another framework, and it is the reason locales are shared but state is not.

Schemas are the strongest argument for sharing. Two copies of a Zod `WsMessage` schema do not fail loudly when they drift — they fail as a runtime mismatch weeks later. One copy makes a backend change a compile error in whichever UI has not kept up.

The existing ESLint rule confining `fetch`/`WebSocket`/`@tauri-apps/*` is re-scoped from "inside `src/lib/core-client`" to "inside `packages/core-client`", so it applies to both UIs automatically and to any UI added later.

### Sanctum is Svelte 5, and consumes the design-system package

Svelte 5 rather than React. The original reason was reuse of the existing design system; that system is now being discarded and rebuilt by the `sanctum-design-system` prerequisite, so the rationale is narrower: it is the framework the repository already uses, its toolchain and checks are in place, and the rebuilt library is authored in it. The shared client package stays framework-agnostic, so a third UI could still be React.

The design system arrives as `packages/design-system/` — a workspace peer of `packages/core-client/`, with tokens measured from the reference and Storybook as its catalog. Sanctum depends on it; it does not contain it. That separation is what lets the library be finished and reviewed against the reference before a screen consumes it, which is precisely what did not happen the first time.

`ui/classic` does not consume it. The prerequisite change reverts classic off the discarded components onto its own styling, and classic then stops changing.

### Sanctum ships one design with two schemes

The `classic` token pack is gone. Its purpose was to let one application present two appearances, and that purpose no longer exists — the classic appearance is now an entire application. The package delivers light and dark **schemes** of the Sanctum design; this change wires the Light/Dark/Auto control and the pre-paint inline script that avoids a mis-schemed first frame, against the package's `data-theme` contract.

### Reference coverage is complete, and its data gaps are surfaced rather than faked

Sanctum implements all six reference screens and every feature within them. The reference's own audit (carried forward in the archived change's `design-coverage.md`) enumerates what that means: the dashboard's live status and stat strip; the events filters, search and detail aside; the event editor's six numbered sections including debounced scripture lookup and the live auto-title preview; the presentations transport dock, slide queue and presenter client list; the settings overview cells, cron editor and account rows; the connectors category groups with expand-in-place detail, OBS destination chooser and Broadlink discovery. Excluded as scaffolding: the phone frame, simulated status bar and the prototype's tweaks panel.

The reference runs entirely on fixtures, so every screen needs rewiring to live data — and some of what it displays may have no backend equivalent. Its dashboard shows viewer count, bitrate and dropped-frame percentage; whether the core supplies those is unverified. The rule is that such a gap is resolved explicitly — implement it, omit it, or show it as unavailable — and never rendered as a plausible-looking invented value. An audit pass against the core's actual capabilities precedes the screen work, so these decisions are made once and up front rather than improvised per screen.

### The registry needs a second entry, not new machinery

`scripts/build-ui.mjs` already handles single-UI staging (copy to `build/`), multi-UI staging (`build/ui/<id>/` plus a chooser that reads `metocast.activeUi` from `localStorage`), and writes `bundled-uis.json`. `ActiveUiSettings` already reads it. The work is: add the `sanctum` entry, point both entries at per-UI build commands, and re-point `tauri.conf.json`. One caveat the spec pins down — both UIs must read and write the *same* persisted selection key, or a user who switches to Sanctum cannot switch back from within it.

## Risks / Trade-offs

- **The relocation is a large mechanical diff that touches everything and delivers no user-visible value.** → Do it as its own step, before any Sanctum work, with the existing e2e suite as the pass/fail gate. Nothing else proceeds until the relocated app builds and its tests pass.
- **Two Svelte apps in one workspace can drift in dependency versions,** producing bugs that reproduce in one UI only. → Workspace-level dependency management, and CI builds both UIs on every change.
- **The shared package can quietly grow into a dumping ground** — a helper here, a store there — until it is a third UI's worth of coupling. → The framework-agnostic test is the fence: anything importing a framework does not belong. It is checkable, not a matter of taste.
- **Sanctum will reveal missing backend capabilities** (viewer counts, bitrate, dropped frames are the likely ones), and each is a decision that can stall a screen. → Audit the reference's data needs against the core's actual surface *before* screen work, so the decisions are made up front.
- **`ui/classic` freezes in a half-migrated state** — four-destination shell, unified notifications and design-system settings, but 41 routes still on ad-hoc styles. It is internally inconsistent and will stay that way. → Accepted deliberately: it is the fallback UI, not the destination. The alternative, reverting three commits, spends real effort to make a frozen artefact tidier.
- **Feature asymmetry may surprise users.** A user who switches to Sanctum loses queues, logs, obs-devices, obs-caption, rf-ir, connect and setup. → The UI selector must describe each UI's coverage honestly rather than presenting them as equivalent choices.
- **This change is blocked on the design system.** Groups 1–4 (workspace, shared package, relocation, registry) are independent of it, but no screen can be built until `sanctum-design-system` delivers its package. → Sequence the independent groups first so the split is proven while the library is still being built.
- **Screen work will still find gaps in the library**, however carefully it was built — the reference never had to render a queue table or an error state. → The prerequisite's rule holds: a missing control is added to the package with its specification and stories, never implemented locally in a screen.

## Migration Plan

1. **Workspace skeleton** — `pnpm-workspace.yaml`, empty `packages/core-client/`, `ui/` directories. No moves yet.
2. **Extract the shared package** — move `api`, `ws`, `host`, `schemas`, `types`, `utils`, `core-client`, `locales`; add its `package.json`; re-scope the ESLint transport rule. The app still at `/src` now imports from the package. Tests pass here, before anything relocates.
3. **Relocate the existing app** to `ui/classic/` with its own configuration; update `tauri.conf.json`, `vitest.config.ts`, `e2e/` and CI. **Gate: the app builds, runs and its e2e suite passes.**
4. **Register both UIs** — `sanctum` entry pointing at a minimal placeholder app; verify `METOCAST_UI=classic,sanctum` builds, the chooser routes correctly, the settings selector switches, and the persisted key is shared. **Gate: switching works in both directions before any screen is built.**
5. **Consume the design-system package** — add `packages/design-system` as a Sanctum dependency, import its tokens and fonts, wire the Light/Dark/Auto control. Requires `sanctum-design-system` to be complete.
6. **Audit reference data needs against the core** — enumerate every value each screen displays, map it to a core operation or event, and record a decision for each gap.
7. **Build the screens** — settings and connectors first (densest use of the row/list idiom, and where the reference's patterns are most reusable), then dashboard, events, event editor, presentations. Each screen verified for keyboard operability, AA contrast in both schemes, mobile at 360px, and both locales.
8. **Honest UI descriptions** in the registry and selector, stating what each UI covers.

**Rollback:** `METOCAST_UI` defaults to `classic`, so every step before 8 leaves the shipped product exactly as it is today. Sanctum only reaches users when it is deliberately bundled.

## Open Questions

- Does the core supply the dashboard's viewer count, bitrate and dropped-frame percentage, or does the reference invent them? Determines whether step 6 produces backend work or omissions.
- Should `ui/classic` keep offering the UI selector once Sanctum's coverage is partial, or should the selector warn about what a switch loses?
