## Status

Foundations, the design system, the notification unification, the shell and the catalog are
complete and enforced in CI. The route-by-route surface migration (group 9) has started with
the settings shell and is the bulk of the work still outstanding; `pnpm ds:check` prints the
41 surfaces that remain. Items left unticked below are either those surfaces or checks that
need a human at a real macOS window — reference screenshots, pixel parity against the
pre-change appearance, offline font rendering, and the four-combination visual passes.

Legend: `[x]` done · `[~]` partly done · `[ ]` outstanding.

## 0. Reference audit

The component-by-component audit of `~/workspace/ui/sermon-helper-svelte` is recorded in `design-coverage.md`. These tasks keep it current and act on it.

- [x] 0.1 Re-verify the coverage matrix against the reference before starting, and against the app's routes if any have been added since
- [x] 0.2 B1–B9 are settled: B1 navigation collapses to 4 destinations with the rest under Settings, responsive tab bar (mobile) / sidebar (desktop); B3 Sanctum gets its own distinct accent hue; B6 the reference's notification system is adopted in full and replaces every existing mechanism. Re-confirm only if the reference changes
- [x] 0.5 Map every one of the 10 current sidebar items to its new home and confirm the Settings hub grouping (Connections / Devices & output / Diagnostics / App)
- [x] 0.6 Inventory the reference's mobile-first layouts (its base styles below `760px`) alongside its `≥760px` and `≥1360px` reflows — both are the migration target, not just the desktop one
- [x] 0.3 Copy the reference's `global.css` theme blocks and per-component styles into the token authoring notes so nothing is lost when the prototype folder is not to hand
- [ ] 0.4 Capture reference screenshots of all 6 screens in light and dark at desktop widths (≥760px and ≥1360px) as the visual target for migration review

## 1. Token foundations

- [x] 1.1 Create `src/lib/ds/` skeleton (`tokens/`, `primitives/`, `patterns/`, `a11y/`, `docs/`, `index.ts`) and a `ds/README.md` stating the layering rule: components consume semantic/component tokens only
- [x] 1.2 Author `tokens/tokens.json` in DTCG format — primitive layer (colour ramps, `space.*`, `radius.*`, `border.*`, `font.*`, `duration.*`, `ease.*`, `z.*`, `breakpoint.*`, `target.min`) and the semantic role names every theme must define
- [x] 1.2b Define breakpoint tokens from the reference's own thresholds (mobile below `760px`, desktop from `760px`, wide from `1360px`) and a 44px minimum touch target token
- [x] 1.3 Write `scripts/build-tokens.mjs` (no dependencies) emitting `tokens/generated/tokens.css` as `:root` + `[data-design]` + `[data-design][data-theme='dark']` blocks
- [x] 1.4 Add `pnpm tokens:build` and `pnpm tokens:check` (regenerate to temp dir, diff against committed CSS, fail on drift); wire `tokens:check` into CI
- [x] 1.5 Write the theme-completeness validator: fail naming any semantic token missing from any theme in either colour scheme
- [x] 1.6 Write `scripts/check-contrast.mjs` — resolve every documented token pairing per theme × scheme, compute WCAG 2.2 ratios, fail below threshold (4.5:1 text, 3:1 large text / UI / focus ring), emit a JSON report for the catalog

## 2. Theme packs

- [x] 2.1 Author `themes/classic.json` — a faithful token encoding of today's `src/app.css`, light and dark, including glass tints, blur values, `data-glass='false'` values and reduced-transparency values
- [ ] 2.2 Verify classic parity: `app.css` reduced to reset + token import, app renders pixel-identical to pre-change in light and dark, on glass and non-glass
- [x] 2.3 Run `check-contrast` against the raw Sanctum prototype palette and record which pairings fail AA before authoring the pack
- [x] 2.4 Author `themes/sanctum.json` light scheme — surfaces, ink ramp (darkened where 2.3 failed), hairlines, status colours, ember accent, typography roles, square radii, no elevation
- [x] 2.5 Author `themes/sanctum.json` dark scheme, complete and independently contrast-verified
- [x] 2.6 Derive Sanctum glass tints from its surface colours at the existing alphas; move blur/saturate, `data-glass='false'` and reduced-transparency values into per-theme token blocks
- [x] 2.7 Define Sanctum's interactive accent as a distinct hue — start from a deep lapis (light) / lightened lapis (dark), at least 60° from `--live`, error and warn, distinguishable from `--ok`, and verified distinguishable under common colour-vision deficiencies. It governs focus rings, active nav, selection and links; the inverted-ink primary button is unchanged
- [x] 2.8 Add hue-separation validation so a theme whose accent sits within 60° of its error or live colour fails with the measured separation reported
- [x] 2.9 Both completeness, hue-separation and contrast checks pass for both themes in both schemes

## 3. Typography assets

- [x] 3.1 Add latin + latin-ext woff2 subsets for Cormorant Garamond (400/500/600 + 400 italic), Inter Tight (400/500/600/700), Geist Mono (400/500) to `static/fonts/`
- [x] 3.2 Declare `@font-face` rules with `font-display: swap` and per-role platform fallback stacks in the token CSS
- [x] 3.3 Verify Hungarian glyphs (`ő`, `ű`, `Ő`, `Ű`) render in all three families
- [ ] 3.4 Verify zero external font requests at startup and correct rendering with networking disabled

## 4. Appearance state and settings

- [x] 4.1 Add `src/lib/stores/appearance.ts` — `designTheme`, `colorScheme` (`light`/`dark`/`auto`), `resolvedScheme` derived from `colorScheme` + `systemTheme`
- [x] 4.2 Rework `src/lib/stores/system-appearance.ts` so `systemTheme` feeds `auto` instead of being written to the DOM; keep `glassSupported` and `reduceTransparency` behaviour intact
- [x] 4.3 Non-Tauri fallback: `auto` follows `matchMedia('(prefers-color-scheme: dark)')` and updates live (covers browser client mode, `/caption`, `/presenter`)
- [x] 4.4 Persist both choices via the `src/lib/i18n.ts` pattern — `localStorage` plus Tauri `settings.json` through `loadHostStore`, failures warned not surfaced
- [x] 4.5 Add the blocking inline script to `src/app.html` before `%sveltekit.head%` that applies `data-design` and `data-theme` from `localStorage` pre-paint
- [x] 4.6 Update `+layout.svelte` `$effect`s to write `data-design` from `designTheme` and `data-theme` from `resolvedScheme`
- [x] 4.7 Build `src/lib/components/settings/AppearanceSettings.svelte` — theme selector (self-hiding at one registered theme) and Light/Dark/Auto control, both labelled and keyboard operable
- [x] 4.8 Mount it in `src/routes/settings/+page.svelte`; add en + hu strings to `src/lib/locales/`
- [x] 4.9 Handle a persisted theme id that is no longer registered by falling back to the default without error
- [ ] 4.10 Verify: live switch without reload, no wrong-scheme first frame, choices survive restart, explicit Light/Dark overrides the OS, Auto tracks it live

## 5. Accessibility primitives

- [x] 5.1 `a11y/focus-trap` — trap within container, restore focus to the opener on release
- [x] 5.2 `a11y/roving-tabindex` — arrow/Home/End movement, orientation-aware, optional wrapping
- [x] 5.3 `a11y/dismissable` — Escape, outside-click, layered so only the topmost overlay dismisses
- [x] 5.4 `a11y/live-region` — polite and assertive announcement channels
- [x] 5.5 Document each action's contract in `ds/docs/`

## 6. Component library

- [x] 6.1 Agree the per-component doc template: anatomy, props API, variants, states, tokens consumed, keyboard map, ARIA roles/attributes, a11y acceptance criteria, do/don't
- [x] 6.2 Form controls: Button, IconButton, Toggle, Checkbox, RadioGroup, Segmented — full state set incl. focus-visible, disabled, loading
- [x] 6.3 Text inputs: TextField, TextArea, Select, FormField wrapper — label association, error text wired via `aria-describedby`, `aria-invalid`
- [x] 6.4 Overlays: Dialog, Sheet, Tooltip, Toast — composed from the a11y actions, meeting the APG scenarios in the spec
- [x] 6.5 Navigation: Tabs (roving tabindex, correct roles and `aria-selected`)
- [x] 6.6 Display: Badge, StatusDot (never colour-only), Spinner, ProgressBar, EmptyState, Stat, StatGrid
- [x] 6.7 Structure: List, Row (leading/title/meta/detail/control/chevron, clickable + danger variants), SectionLabel, PageHeader (responsive serif clamp per B8), DateBlock, Stat, OverviewCell, SettingsRow — the Sanctum hairline/flat treatment, driven by tokens
- [x] 6.8 Chamber form family: ChamberSection, Chamber, ChamberNative, ToggleRow, RefField — with the visible resting boundary and compliant focus indicator required by B5
- [x] 6.9 StickyActionBar for full-page editors (B4)
- [x] 6.10 Write the doc for every component; add the check that fails on any export lacking a doc
- [x] 6.11 Export everything through `ds/index.ts`; add the check that fails on any app import of a `ds` internal path

## 6b. Gap components (G1–G14) — no reference exists, design them in the Sanctum idiom

- [x] 6b.1 G1 Table — column headers, alignment, sort affordance, overflow, narrow-width reflow (queues, obs-devices)
- [x] 6b.2 G2 Select/listbox — replaces the reference's roleless `.select` button; full APG listbox behaviour (obs-caption, obs-devices, event editor)
- [x] 6b.3 G10 Tabs — one accessible component replacing the reference's four hand-rolled treatments (`.tabs` ×2, `.filters`, `.choice`)
- [x] 6b.4 G3 Skeleton loader, G4 EmptyState, G5 ErrorState with retry, G6 NotConnectedState with settings link
- [x] 6b.5 G7 LogStream — monospace scrollback, level colouring, follow-tail, copy
- [x] 6b.6 G8 ProgressBar (determinate, for uploads) and G9 Badge/Pill (live, upcoming, duration, device status)
- [x] 6b.7 G11 RadioGroup and G12 stepped wizard shell for `/setup`
- [x] 6b.8 G13 Tooltip
- [x] 6b.9 G14 Icon strategy — decide between restyling `lucide-svelte` to the reference's 24px/1.4px-stroke idiom or hand-drawing the ~40 icons the app needs; the reference ships only ~11
- [x] 6b.10 Brand marks: Glyph (youtube, facebook, discord, obs, atem, vmix, broadlink, twitch), Lockup, FlameMark, TextIcon
- [x] 6b.11 Every gap component gets a doc and a catalog entry on the same terms as 6.10
- [x] 6b.12 Audit the finished inventory for duplicated concepts — badge/pill/status-chip, dot/indicator, sheet/dialog, list-row/settings-row — and collapse each to one component with variants

## 6c. Notification system (replaces every existing mechanism)

- [x] 6c.1 Extend the notification model on `src/lib/stores/errors.ts` — tier (critical/high/medium/low), persistence, per-notification actions, grouping, read/dismissed state, expandable remediation — reusing the existing `infoMarkdown` and `connectorId` fields rather than introducing a parallel store
- [x] 6c.2 Build Toast + ToastOverlay (stacked, capped, tier-styled, `live-region` announced) and NotificationCenter + NotifBell (unread count, worst-tier colouring) as design-system components
- [x] 6c.3 Migrate all 40 `toast.*` call sites across the 8 files that use them to the unified API
- [x] 6c.4 Remove the `svelte-sonner` dependency and its `Toaster` from `+layout.svelte`; confirm no import remains
- [x] 6c.5 Absorb `/errors` into the notification centre — carry over recheck, markdown remediation rendering and `ConnectorFixModal`; retire the route as a destination with no loss of capability
- [x] 6c.6 Replace `NavErrorBadge` with `NotifBell` in the shell; confirm no second unread/error indicator remains anywhere

## 7. Catalog

- [x] 7.1 Create `src/routes/design/` with foundations pages (colour, type, space, radius, border, elevation, motion, z-index) showing token names and resolved values
- [x] 7.2 Component inventory rendering every export across variants and states, each labelled
- [x] 7.3 Theme/scheme preview scoped to a container (never mutating the user's saved settings), plus a light/dark side-by-side view
- [x] 7.4 Forced `data-glass='false'` and reduced-transparency preview modes
- [x] 7.5 Render each component's keyboard map and ARIA semantics from its doc
- [x] 7.6 Render the contrast report from `check-contrast`, flagging failures with ratio, threshold, theme and scheme
- [x] 7.7 Pseudo-locale toggle with ~2× expanded text for i18n layout checking
- [x] 7.8 Completeness check: every `ds/index.ts` export appears in the catalog
- [x] 7.9 Exclude the route from production builds and verify its modules are absent from the output bundle

## 8. Shell migration

- [x] 8.1 Define the four destinations once, and build one navigation component with two presentations: bottom tab bar below the mobile breakpoint, sidebar at or above it
- [x] 8.2 Rebuild `src/routes/+layout.svelte` around it — nav, footer affordances, content pane — adopting the reference's mono-uppercase, hairline, ink-fill-active treatment. Do not port the phone frame, notch island, home indicator or fake status bar (B2)
- [x] 8.3 Build the Settings hub: grouped entries for Connectors, Connect, OBS caption, OBS devices, RF/IR, Queues, Logs and Setup, linking to their existing unchanged paths
- [x] 8.4 Fold `/live-events` into the Events "Live" filter; retire it as a destination while keeping its path working
- [x] 8.5 Preserve macOS traffic-light spacing, `data-tauri-drag-region` strips and window corner radius on desktop; suppress that chrome below the mobile breakpoint
- [x] 8.6 Honour `env(safe-area-inset-*)` on the tab bar and any bottom-anchored element
- [x] 8.7 Mark the active nav destination with `aria-current` and distinguish it by more than colour, in both presentations
- [x] 8.8 Migrate `NavConnectors.svelte`; replace `NavErrorBadge` per 6c.6
- [ ] 8.9 Verify the shell in both themes × both schemes × mobile and desktop widths, on glass and non-glass

## 9. Surface migration

Verdicts from `design-coverage.md`: **[C]** the reference has a counterpart to follow · **[P]** partial, the rest must be designed · **[N]** no counterpart, compose from the system.

- [ ] 9.1 **[C]** Settings — `settings/+page.svelte` and all `components/settings/*` (Language, AppMode, ActiveUi, CronJobs, AppVersion); follows the reference `Settings` screen
- [ ] 9.2 **[C]** Dashboard — `routes/+page.svelte`, `ConnectorDashboardWidget`; follows the reference `Dashboard` incl. broadcast card, stat strip, quick actions
- [ ] 9.3 **[C]** Events list, `new`, `[id]/edit` — plus `EventCard`, `EventList`, `CreateEventForm`, `BibleSuggestions`; follows `Events` + `EventEditor` incl. chamber sections and auto-title preview
- [ ] 9.4 **[P]** Event detail `[id]` — the reference only has a preview aside; design the full detail page from the system
- [ ] 9.5 **[C]** Presentations route; **[N]** `SlideEditorModal` has no reference
- [ ] 9.6 **[C]** Connectors — `ConnectorSettingsBlock` adopts the reference's expand-in-place rows (B7); **[P]** `ConnectorStatusBadge` needs G9, `ConnectorFixModal` and `ReLoginModal` carry remediation flows the reference never shows
- [ ] 9.7 **[P]** Broadlink — `DiscoveryPanel` follows `BroadlinkDiscovery`; `DeviceList`/`CommandList` from List+Row; **[N]** `LearnDialog`, `CodeEntryDialog`, `ImportDialog` need designing
- [ ] 9.8 **[N]** Recordings — `RecordingsBlock`, `RecordingList`, `CreateRecordingForm`, `AssignRecordingDialog`, `UploadModal` (needs G8 progress)
- [ ] 9.9 **[N]** Connect route — `ConnectionGuide`, `TokenDisplay`, `PresenterInstallCard`, `SshAccessCard`
- [ ] 9.10 **[N]** Live events (635 lines) — now the Events "Live" filter rather than a destination; needs G10 tabs, G9 badges, G3 skeletons, G4/G5/G6 states
- [ ] 9.11 **[N]** Queues (351 lines) — needs G1 table, metric cards, filter row
- [ ] 9.12 **[N]** Logs (482 lines) — needs G7 log stream, status panels
- [ ] 9.13 **Absorbed** Errors (261 lines) — folded into the notification centre per 6c.5, not migrated as a standalone page
- [ ] 9.14 **[N]** OBS devices (565 lines) — needs G1 table, G2 selects, G9 status badges, inline edit, add form
- [ ] 9.15 **[N]** OBS caption (423 lines) — needs G2 selects
- [ ] 9.16 **[P]** RF/IR — discovery covered; command list, learn/import/code-entry dialogs are not
- [ ] 9.17 **[N]** Setup (249 lines) — needs G11 radio group, G12 wizard shell, G5 error surface
- [ ] 9.18 **Out of scope — do not touch** `/presenter` and `/caption`. They are projection output with a fixed presentation design. Verify they receive no theme or colour-scheme attribute, consume no design-system token, and are visually unchanged by any appearance setting
- [ ] 9.19 Do not migrate `TweaksPanel` — prototype-only demo control panel with no app equivalent
- [ ] 9.20 For each surface above: keyboard traversal, focus order, visible focus, accessible names, AA contrast in both schemes, **usable at a 360px viewport with no horizontal page scroll and 44px touch targets**, `pnpm check` and `pnpm lint` clean, e2e passing
- [ ] 9.21 Update the e2e suite for the navigation restructure — paths are unchanged, but nav-driven journeys now route through the Settings hub and the Events "Live" filter

## 10. Enforcement and completion

- [x] 10.1 Add the styling check — no literal colour, font-size, font-family, border-radius, box-shadow or border-width in files recorded as migrated; report remaining unmigrated surfaces
- [x] 10.2 Maintain the migrated-surface register so the check's scope grows with the migration
- [x] 10.3 Make Sanctum the default theme for new installs once the high-traffic surfaces are done
- [ ] 10.4 Full-app verification pass: every route in Sanctum light and Sanctum dark, on glass, non-glass and reduced transparency
- [ ] 10.5 Same pass in classic light and classic dark — it stays a supported option, so every migrated surface must render correctly in it too
- [ ] 10.5b Full-app mobile pass at 360px and at a tablet width: navigation, safe-area insets, touch targets, table reflows, dialogs-as-sheets
- [ ] 10.5c Confirm exactly one notification path remains — no `svelte-sonner` import, no standalone errors destination, no second unread indicator
- [ ] 10.6 Verify in both locales that no layout breaks and text remains readable at 200% zoom
- [ ] 10.7 Confirm no regression to core behaviour, HTTP/WS contracts, OpenAPI, Bruno, Zod schemas or Companion
- [x] 10.8 Keep both theme packs in a single build — no per-design application builds; confirm a persisted unregistered theme id falls back cleanly
- [ ] 10.9 Confirm the appearance settings have no observable effect on `/presenter` or `/caption`
- [x] 10.10 Update `AGENTS.md` / `README.md` with the design-system rules: where tokens live, how to add a component, the doc + catalog + contrast requirements, and that both themes must be kept working
