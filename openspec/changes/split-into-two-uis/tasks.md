## 1. Workspace skeleton

- [ ] 1.1 Add `pnpm-workspace.yaml` covering `ui/*` and `packages/*`
- [ ] 1.2 Create `packages/core-client/` and `ui/` directory structure with no moves yet
- [ ] 1.3 Confirm the existing app still builds and its checks pass before anything moves

## 2. Extract the shared client package

- [ ] 2.1 Move `src/lib/{api,ws,host,schemas,types,utils,core-client}` into `packages/core-client/src/`
- [ ] 2.2 Move `src/lib/locales/*.json` into `packages/core-client/locales/`
- [ ] 2.3 Add `packages/core-client/package.json` with a single public entry point; confirm no user-interface framework appears in its dependencies or imports
- [ ] 2.4 Replace `$lib/...` imports across the app with the package name; remove the old paths
- [ ] 2.5 Re-scope the ESLint rule confining `fetch`, `WebSocket` and `@tauri-apps/*` so it permits them only inside `packages/core-client` and applies to every UI automatically
- [ ] 2.6 Add the check that fails when one UI imports from another UI's directory
- [ ] 2.7 Verify: app builds, `pnpm check` and `pnpm lint` clean, e2e passes — all before relocating anything

## 3. Relocate the existing app to `ui/classic`

- [ ] 3.1 Move `/src`, `static/` and the frontend configuration into `ui/classic/` with its own `package.json`, `svelte.config.js`, `vite.config.ts`, `tsconfig.json`
- [ ] 3.2 Remove root-level frontend configuration so no UI remains at the repository root
- [ ] 3.3 Update `vitest.config.ts` and `e2e/` paths
- [ ] 3.4 Update CI to build and check every workspace member
- [ ] 3.5 **Gate:** the relocated app builds, runs and passes its e2e suite with no assertion weakened or deleted; behaviour, routes and appearance unchanged

## 4. Register both UIs and prove switching

- [ ] 4.1 Update `ui/registry.json` — `classic` and `sanctum` entries, each with id, display name, description, per-UI build command, build dir and entry
- [ ] 4.2 Re-point `tauri.conf.json` `devUrl`, `beforeBuildCommand` and `frontendDist` through the registry; confirm no desktop-shell config names a UI directory directly
- [ ] 4.3 Scaffold `ui/sanctum/` as a minimal Svelte 5 app that boots and reaches the core through the shared package
- [ ] 4.4 Verify `METOCAST_UI=classic` and `METOCAST_UI=sanctum` each produce a working single-UI bundle
- [ ] 4.5 Verify `METOCAST_UI=classic,sanctum` stages both under `build/ui/<id>/`, writes `bundled-uis.json`, and the chooser routes to the stored selection
- [ ] 4.6 Confirm both UIs read and write the **same** persisted active-UI key, so a user who switches into Sanctum can switch back from within it
- [ ] 4.7 **Gate:** switching works in both directions before any screen work begins

## 5. Verify Sanctum can consume its dependencies

The Sanctum application itself — its shell and its six screens — is the `sanctum-ui` change. This group only proves the scaffold can reach what it needs.

- [ ] 5.1 Add `packages/design-system` and `packages/core-client` as dependencies of `ui/sanctum`
- [ ] 5.2 Confirm the scaffold renders a design-system component through the package's public entry point, with its tokens and bundled fonts loading
- [ ] 5.3 Confirm the scaffold performs one core operation and receives one core event through `packages/core-client`
- [ ] 5.4 Confirm Storybook and its dependencies are absent from Sanctum's production bundle
- [ ] 5.5 Confirm the transport lint rule rejects a direct `fetch` or `WebSocket` written inside `ui/sanctum`

## 6. Completion

- [ ] 6.1 Write honest coverage descriptions for both UIs in the registry and the selector, stating what each does and does not cover
- [ ] 6.2 Decide whether the selector should warn about features lost when switching to Sanctum, and whether Sanctum hosts the selector itself so a user can switch back from within it
- [ ] 6.3 Confirm `ui/classic` behaves exactly as before the restructure — routes, features, appearance, tests
- [ ] 6.4 Confirm no regression to the core, HTTP/WS contracts, OpenAPI, Bruno or Companion
- [ ] 6.5 Update `AGENTS.md` and `README.md` — the workspace layout, how to add a UI, what belongs in the shared package and what does not, and how to build one or both UIs
