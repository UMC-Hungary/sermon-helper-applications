## Context

The frontend is a SvelteKit 2 / Svelte 5 app (`/src`) built with `adapter-static` and loaded by Tauri as `frontendDist`, plus served over HTTP by the embedded Axum server in client mode. Its entire visual layer today is:

- `src/app.css` — ~160 lines: a reset, one flat block of `:root` custom properties, a `[data-theme='dark']` override block, a `[data-glass='false']` block for Windows/Linux, a `prefers-reduced-transparency` block, and a single `.glass-card` utility.
- `src/routes/+layout.svelte` — the app shell (sidebar, nav, drag strip, content pane) with ~100 lines of local styles.
- 18 routes and 35 components, each carrying its own `<style>` block with literal `border-radius: 8px`, `font-size: 0.875rem`, `box-shadow: 0 1px 3px …` and so on.

Colour is already partly centralised (`--text-primary`, `--border`, `--accent`, status triples), which is why a naive reskin looks deceptively cheap. But typography, spacing, radius, elevation and motion are not centralised at all, and there is no component vocabulary: the same "labelled row with a toggle" is rewritten in `ConnectorSettingsBlock.svelte`, `CronJobsSettings.svelte`, the settings page and several dialogs.

Light/dark is not a user setting. `src/lib/stores/system-appearance.ts` reads the host appearance through `watchHostTheme` and `+layout.svelte` writes it straight to `document.documentElement`'s `data-theme`. There is no override.

The target design ("Sanctum") is prototyped at `~/workspace/ui/sermon-helper-svelte` — a Svelte 4 Vite app, 28 components across 6 screens, with a warm parchment palette (`--bg #efe9dd` light / `#14120f` dark), Cormorant Garamond serif headlines, Inter Tight body, Geist Mono uppercase micro-labels at 10px/2px tracking, hairline dividers, square corners and no shadows. It is a *mobile phone-shell* prototype loading fonts from Google Fonts. It is a design reference, not a codebase to import.

**`design-coverage.md` in this change directory is the component-by-component audit of that reference against this application** and is a required input to the plan below. Its findings in short:

- **6 of the app's 18 routes have a design counterpart** (33%); 4 are partially covered; 8 have nothing — `connect`, `live-events`, `queues`, `logs`, `obs-devices`, `obs-caption`, `setup`, `caption`.
- **14 component gaps (G1–G14)** — vocabulary the reference has no answer for: data table, accessible select, skeleton, empty state, error state, "not connected" state, log stream, progress, badge, real tabs, radio group, wizard, tooltip, and an icon set beyond the reference's ~11 glyphs.
- **9 breaking changes (B1–B9)** — reference patterns not supported today, the largest being its 4-destination bottom tab bar against the app's 10-item sidebar, and its complete absence of an accent colour against the app's 79 uses of macOS `AccentColor`.
- **The reference is not accessible.** One focus rule in 28 components (`label:focus-within`), zero `:focus-visible`, 17 total `aria`/`role` occurrences, its only dialog has no focus trap, and tab-like controls are reimplemented four times as plain buttons. It cannot be ported component-for-component.

Constraints: no display-time network access (offline desktop app, embedded assets in release builds); macOS liquid-glass window transparency must survive; `pnpm check` and `pnpm lint` must stay clean; existing e2e tests must keep passing; two locales (en, hu) with the Hungarian strings frequently longer.

## Goals / Non-Goals

**Goals:**
- A token architecture where any component's appearance is fully determined by tokens, so a theme is data rather than code.
- Complete light **and** dark schemes for every theme, with a user-facing Light/Dark/Auto control — a capability the app does not have today.
- An accessible component library with a written contract per component (anatomy, props, states, tokens, keyboard map, ARIA, acceptance criteria), modelled on the WAI-ARIA Authoring Practices for behaviour and DTCG for tokens.
- Preserved macOS glass transparency, retinted to the active theme.
- Migration of every in-scope route and component, incrementally, with the app usable at every step.
- Mobile as a first-class target — responsive navigation and surfaces that work from 360px upward, which the app does not support today.
- One notification system and one component per concept, replacing the two parallel notification mechanisms that exist now.
- A living catalog that makes drift and contrast failures visible.

**Non-Goals:**
- Porting the prototype's *phone chrome* — the 402×874 frame, notch island, home indicator and fake iOS status bar (coverage B2). That is mockup scaffolding around the design, not part of it. Its bottom tab bar **is** adopted, as the mobile navigation.
- Moving route URLs. The navigation restructure (B1) changes what the nav offers and where things are reached from, not the paths themselves.
- Any change to the Rust core, HTTP/WS contracts, OpenAPI, Bruno collections, Zod schemas or Companion.
- A new runtime UI dependency. No Radix port, no component library, no Storybook, no Tailwind. The Radix/Base-UI reference is a model for *behavioural contracts*, not a dependency.
- Restyling `presenter-receiver` (native Rust renderer) or the outlined-SVG presenter output — not DOM surfaces.
- Interacting with the `ui/registry.json` whole-UI mechanism from `decouple-core-from-rendering-ui`. Different axis, deliberately untouched.

## Decisions

### Design system lives in-app at `src/lib/ds/`, not as a package

The app is the only DOM consumer that matters right now. A workspace package would add pnpm workspace wiring, a build step and a versioning story to serve exactly one consumer. In-app keeps imports as `$lib/ds`, keeps `svelte-check` covering everything in one pass, and can be lifted into `packages/` later if a second registered UI ever needs it — the public entry point requirement exists precisely to keep that extraction cheap.

```
src/lib/ds/
  tokens/
    tokens.json         DTCG source of truth (primitives + semantic aliases)
    themes/{sanctum,classic}.json
    generated/tokens.css     committed, generated
  primitives/           Button, IconButton, Toggle, Checkbox, RadioGroup,
                        Segmented, TextField, TextArea, Select, Dialog, Sheet,
                        Tabs, Tooltip, Toast, Badge, StatusDot, Spinner,
                        ProgressBar, List, Row
  patterns/             PageHeader, SectionLabel, SettingsRow, Stat, StatGrid,
                        EmptyState, Table, FormField, DialogForm
  a11y/                 focus-trap, roving-tabindex, dismissable, live-region
  docs/                 <Component>.md — one spec per component
  index.ts              the public entry point
```

*Alternatives considered:* a `packages/design-system` workspace package (rejected — cost without a second consumer today); a second registered UI in `ui/registry.json` (rejected — duplicates 18 routes and doubles maintenance for a restyle).

### Three token layers, DTCG source, committed generated CSS

Layer 1 primitives (`sanctum.clay.400`, `space.3`, `radius.none`) → layer 2 semantic roles (`surface.raised`, `text.primary`, `border.hairline`, `status.live.fg`) → layer 3 component tokens (`button.primary.bg`) only where a component genuinely needs its own knob. Components reference layers 2–3 exclusively.

The source is DTCG JSON because it is machine-checkable: completeness ("does every theme define every semantic token in both schemes?") and the contrast report both become scripted checks rather than review discipline. A small Node generator (`scripts/build-tokens.mjs`, no dependency) emits `generated/tokens.css`. That CSS is **committed** so an ordinary `pnpm build` needs no generator run; a `pnpm tokens:check` regenerates into a temp dir and diffs, and runs in CI.

Emission shape — theme on `data-design`, scheme on `data-theme`, so both are live-switchable attributes:

```css
:root { /* primitives + non-themed tokens: space, radius, motion, z */ }
[data-design='sanctum'] { --text-primary: #1c1a16; --surface-raised: #f6f2e8; … }
[data-design='sanctum'][data-theme='dark'] { --text-primary: #ede6d6; --surface-raised: #1e1b16; … }
[data-design='classic'] { … }
[data-design='classic'][data-theme='dark'] { … }
```

*Alternative considered:* Style Dictionary. Rejected — a dependency and a config file to emit CSS custom properties a 60-line script produces, in a repo that avoids frontend build dependencies.

### Sanctum keeps its palette; the glass keeps its physics

The prototype's opaque `--bg`/`--card` values become the theme's *opaque* surface tokens and the basis of its *translucent* ones. Glass tints are derived from the theme's surface colours at the alpha values the current app already uses, so transparency behaviour is unchanged and only the tint moves:

| token | classic light | sanctum light | sanctum dark |
| --- | --- | --- | --- |
| `--glass-sidebar-bg` | `rgba(246,246,246,.72)` | `rgba(246,242,232,.72)` | `rgba(30,27,22,.72)` |
| `--glass-card-bg` | `rgba(255,255,255,.6)` | `rgba(246,242,232,.6)` | `rgba(35,32,26,.6)` |
| `--content-bg` | `rgba(242,242,247,.85)` | `rgba(239,233,221,.85)` | `rgba(20,18,15,.85)` |

Blur/saturate values, the `tauri-plugin-liquid-glass` call in `enableWindowGlass()`, the `[data-glass='false']` near-opaque fallback and the `prefers-reduced-transparency` opaque fallback all stay — but each becomes a per-theme token block instead of a hard-coded rule in `app.css`. This is what "keep the transparent Apple background with the design's colours" means concretely.

One deliberate deviation from the prototype: it uses `--accent` nowhere, while the app leans on macOS `AccentColor` in 79 places. Sanctum defines its own accent (the prototype's `--live` ember, `#b8331c` / `#e66a4f`) rather than inheriting the system accent, because a system blue against parchment is exactly the incoherence this change exists to remove. Classic keeps `AccentColor`.

### Appearance state: one derived store, attributes applied in `+layout.svelte`

New `src/lib/stores/appearance.ts`:

```ts
export const designTheme  = writable<'sanctum' | 'classic'>('sanctum');
export const colorScheme  = writable<'light' | 'dark' | 'auto'>('auto');
export const resolvedScheme = derived([colorScheme, systemTheme], …); // auto → systemTheme
```

`system-appearance.ts` keeps `watchHostTheme`, `glassSupported` and `reduceTransparency`, but `systemTheme` stops being written straight to the DOM — it becomes the input to `auto`. `+layout.svelte`'s existing `$effect` blocks change to write `data-design` from `designTheme` and `data-theme` from `resolvedScheme`. Outside Tauri, `watchHostTheme` returns null and `auto` falls back to `matchMedia('(prefers-color-scheme: dark)')`, which also covers the browser-served client mode and the `/caption` and `/presenter` pages.

Persistence mirrors `src/lib/i18n.ts` exactly — `localStorage` first (synchronous, needed before paint), mirrored into the Tauri `settings.json` store via `loadHostStore`, failures swallowed with a `console.warn`. This is a deliberate reuse of an established pattern, not a new mechanism.

### Flash-free first paint via a blocking inline script in `app.html`

`localStorage` is only readable in the browser and stores load asynchronously, so a Svelte-side effect always paints one wrong frame first. A ~8-line inline script in `src/app.html`'s `<head>`, before `%sveltekit.head%`, reads the two keys and sets both attributes on `document.documentElement`. It runs before any stylesheet is applied, so the first frame is already correct. The store rehydrates from the same keys and stays consistent.

*Alternative considered:* a CSS-only `prefers-color-scheme` default. Rejected — it cannot express an explicit override, which is the whole point of the Light/Dark control.

### Accessibility: behaviour contracts implemented as reusable actions

Rather than a dependency, the four behaviours that are genuinely hard get one careful implementation each in `src/lib/ds/a11y/`, and every component composes them: `focus-trap` (Dialog, Sheet), `roving-tabindex` (Tabs, Segmented, RadioGroup), `dismissable` (Escape + outside-click + return-focus, for every overlay), and `live-region` (Toast, async status). Getting these four right once is most of what "accessible component library" means in practice; the rest is per-component labelling and contrast, which the specs pin down and the catalog verifies.

Contrast is verified by computation, not inspection: a script walks the documented token pairings, resolves each under every theme × scheme, computes WCAG 2.2 ratios and fails on any pairing below its threshold. This is the check most likely to catch a real regression, because Sanctum's low-contrast parchment palette is exactly the kind that drifts under AA — `--ink-faint #b8b2a4` on `--card #f6f2e8` measures roughly 1.9:1 and is usable only for decoration, never for text. Encoding that as a machine check rather than a guideline is the point.

### Fonts self-hosted as subset woff2

The prototype's Google Fonts `<link>` cannot ship. Cormorant Garamond (400/500/600 + 400 italic), Inter Tight (400/500/600/700) and Geist Mono (400/500) are added to `static/fonts/` as latin + latin-ext woff2 subsets — latin-ext is required for Hungarian (`ő`, `ű`). `@font-face` declarations with `font-display: swap` and a full platform fallback stack per role live in the token CSS. Expected cost roughly 150–250 kB, acceptable for a desktop bundle.

### Catalog at `src/routes/design/`, excluded from production

A plain SvelteKit route, so it renders inside the real Tauri window with real glass behind it — the one place the translucent surfaces can actually be judged. It sets `data-design`/`data-theme` on a *container* rather than the document root, so previewing never touches the user's saved settings, and a split view renders light and dark side by side. It hosts the generated contrast report and each component's keyboard map, read from `ds/docs/`. `svelte.config.js`'s `prerender.entries` / `kit.exclude` keeps it out of production output; a completeness check asserts every export in `ds/index.ts` appears in the catalog.

*Alternative considered:* Storybook. Rejected — a large new toolchain, and it renders in a browser iframe rather than in the glass window where these surfaces actually live.

### Navigation collapses to four destinations; everything else lives under Settings

The 10-item sidebar goes. The app adopts the reference's information architecture: **Dashboard, Events, Presentations, Settings**, with Settings acting as the hub for everything else — Connectors, Connect, OBS caption, OBS devices, RF/IR, Queues, Logs, Errors and Setup.

Two of the ten items resolve especially cleanly:

- **`/live-events` folds into Events.** The reference's Events screen already has `Upcoming / Live / Past / Drafts` filters. Live events become the "Live" filter rather than a separate destination — the same data, reached the way the design intends.
- **`/errors` folds into the notification centre** (see the next decision), so it stops being a destination at all.

The remainder become entries on the Settings hub, grouped: *Connections* (Connectors, Connect), *Devices & output* (OBS caption, OBS devices, RF/IR), *Diagnostics* (Queues, Logs), *App* (language, mode, appearance, cron, version, setup).

**URLs do not move.** `/logs` stays `/logs`; only the way there changes. Relocating routes under `/settings/*` would break the e2e suite and every deep link for no user-visible benefit, and it can be done later if ever wanted. A hub that links to top-level paths is unremarkable.

This is what makes mobile viable: ten sidebar items cannot become a bottom tab bar, four can.

### Navigation is responsive: tab bar on mobile, sidebar on desktop

Mobile is a real target — `src-tauri/icons/android/` ships adaptive icons, `Cargo.toml` already excludes desktop-only dependencies for `ios` and `android`, and client mode serves the UI over HTTP to any phone browser. The app today has essentially no responsive design: about a dozen scattered `max-width` queries for content width, and a fixed 220px sidebar.

The reference is the opposite — it is **mobile-first**. Its screens are authored for a 402px viewport and reflow to two-column desktop layouts at `≥760px` and again at `≥1360px`. So it supplies *both* layouts for the six screens it covers, and its `TabBar` is the mobile navigation the app is missing.

One nav component, two presentations driven by a breakpoint token:

| Viewport | Navigation | Chrome |
| --- | --- | --- |
| `< 760px` | Bottom tab bar, 4 destinations, safe-area inset padding | No sidebar, no traffic-light spacer |
| `≥ 760px` | Sidebar, 4 destinations, glass surface retained | macOS drag regions and traffic-light spacing |

The glass sidebar stays the signature desktop surface — it is where the transparent Apple background reads most strongly, and collapsing to four items makes it lighter, not weaker.

Mobile is not a separate build or a separate set of components. Breakpoints are tokens, every design-system component is responsive by construction, and the catalog previews each one at mobile and desktop widths. Concretely this means: minimum 44×44px touch targets, `env(safe-area-inset-*)` honoured on the tab bar and sticky action bar, no affordance that exists only on hover, tables (G1) that reflow to stacked rows on narrow viewports, and dialogs that present as bottom sheets on mobile — which is exactly what the reference's `SettingsSheet` already is.

### One notification system, and no component with a twin

The reference's notification model is adopted in full rather than deferred: tiered severity, persistence, per-notification actions, grouping, and expandable remediation steps. It is better than what the app has, and the app's own `ConnectorError` store already carries the fields it needs — `connectorId`, `message`, `infoMarkdown` for remediation, `timestamp` — so this is mostly wiring, not new machinery.

The binding constraint is **no duplication of components or purposes**. Today the app has two parallel notification concepts: `svelte-sonner` toasts (40 call sites across 8 files) and a separate `/errors` page rendering the same `connectorErrors` store with its own markdown renderer and fix-modal flow. The design system must not add a third.

So:

- `svelte-sonner` is **removed** as a dependency. The design system's Toast replaces it, and all 40 call sites migrate to one notification API.
- `/errors` is **absorbed** into the notification centre. Its recheck, expand-remediation and fix-modal behaviour move there; the route stops being a destination. `NavErrorBadge` becomes the reference's `NotifBell`.
- One store feeds everything. `connectorErrors` grows the tier/persistence/action fields and becomes the single notification source; toasts, the bell's unread count and the centre are all views over it.
- Nothing gains a second way to say the same thing.

Generalised, this is a rule the whole library is held to and the catalog enforces: if two components render the same concept, one of them is wrong. It is the reason the reference's four hand-rolled tab treatments collapse into one Tabs component, and the reason a "badge", a "pill" and a "status chip" are one component with variants rather than three.

### Uncovered surfaces are composed, and the reference's markup is never copied

Two thirds of the app has no design to follow. Those surfaces are not invented ad hoc: they are composed from the G1–G14 components, which are authored in the Sanctum idiom (hairline, square, mono micro-labels, serif reserved for display) and built *before* the surfaces that need them. That ordering is why the library work in group 6 precedes the surface work in group 9, and why the table, select, badge, tabs, empty/error/loading states are treated as first-class deliverables rather than incidental.

Where the reference does have a counterpart, its *appearance* is reproduced and its *markup is discarded*. Every interactive component is rebuilt against the WAI-ARIA contracts in the component spec — the reference's four separate hand-rolled tab treatments collapse into one accessible Tabs component, its borderless `.select` becomes a real listbox, and every control gains the focus indicator the reference does not have anywhere.

Four deviations from the reference are deliberate and specified rather than left to review-time judgement: an accent token that is neither ink nor the `--live` ember (B3); a visible resting boundary and compliant focus indicator on chamber inputs (B5); a darkened ink ramp wherever `--ink-faint` currently carries text (§5 of the coverage audit); and a responsive clamp on the 44px serif display size (B8).

### Migration order follows risk, not the sidebar

Shell first (it defines every surface behind everything else), then settings (where the new controls live, and the densest concentration of the repeated "row + control" pattern that drives the primitive API), then the high-traffic screens, then the long tail. Each surface is one commit: replace styles, verify keyboard/contrast in both schemes, run checks.

## Risks / Trade-offs

- **Sanctum's palette is low-contrast by design → several pairings will fail AA.** The parchment ink ramp (`--ink-muted`, `--ink-faint`) is decorative-grade. Mitigation: run the contrast script against the raw prototype values *before* writing the theme pack, and darken the ramp where needed. Fidelity to the prototype yields to AA; the spec makes this non-negotiable rather than a judgement call at review time.
- **53 files migrated by hand invites silent visual regressions.** Mitigation: incremental per-surface commits, the `classic` pack as an instant A/B reference, and the styling check (no literal visual values in migrated files) as a mechanical completion criterion. Accepted: no visual regression testing infrastructure is being added.
- **Two themes live at once doubles the surface where a token can be missing.** Mitigation: theme-completeness validation fails on any semantic token missing in any theme × scheme — the failure is loud and immediate rather than a blank region at runtime.
- **Serif headlines and 10px tracked mono labels degrade under long Hungarian strings.** Mitigation: the catalog renders a pseudo-locale with ~2× expanded text; the i18n-safety scenario is part of the component spec, not a post-hoc fix.
- **Glass retinting can only be truly judged on macOS.** The Windows/Linux `data-glass='false'` path and the reduced-transparency path are near-opaque and must be verified separately; a theme that looks right translucent can look muddy opaque. Mitigation: the catalog exposes forced `data-glass='false'` and reduced-transparency previews.
- **Dropping macOS `AccentColor` for Sanctum's ember is a real loss** of platform integration for users who set a custom system accent. Accepted deliberately for visual coherence; `classic` retains `AccentColor`, and reintroducing an "follow system accent" token later is additive.
- **The change is large and will run alongside `decouple-core-from-rendering-ui`,** which also edits `src/lib/core-client` and settings. Mitigation: the two touch disjoint files almost everywhere; the shared ones are `settings/+page.svelte` and `+layout.svelte`, and this change's edits there are additive.

## Migration Plan

1. **Foundations** — token source, generator, committed CSS, both theme packs, bundled fonts, completeness + contrast checks. Nothing user-visible; `app.css` reduced to reset + import, with `classic` reproducing today's appearance pixel-for-pixel as the proof the token layer is faithful.
2. **Appearance settings** — `appearance.ts`, the inline pre-paint script, `AppearanceSettings.svelte`, en/hu strings. Light/Dark/Auto becomes available; Sanctum is selectable but only the shell has adopted it.
3. **Primitives, a11y actions and catalog** — built against real needs from the settings and events screens, not speculatively.
4. **Shell** — `+layout.svelte` sidebar, nav, drag regions, content pane.
5. **Surfaces** — settings → dashboard → events (4 routes) → presentations → connect → connectors and their dialogs → live-events → queues → logs → errors → obs-caption → obs-devices → rf-ir → setup → caption/presenter. One commit each.
6. **Default flip** — Sanctum becomes the default theme for new installs once the high-traffic surfaces are migrated.
7. **Both themes maintained** — classic is not retired. Every migrated surface is verified in classic light/dark as well as Sanctum light/dark, and both packs ship in the one build with the selector permanently visible.

**Rollback:** switching the Appearance setting to `classic` restores the previous appearance completely, because `classic` is a faithful token encoding of today's `app.css`. Since classic is permanent rather than transitional, this rollback path never expires — which is the strongest argument for keeping it, well beyond its ~5 KB cost.

**Ongoing cost:** every migrated surface must be verified in four combinations (Sanctum light/dark, classic light/dark) rather than two. This is the real price of keeping classic — reviewer time, not bytes — and it is accepted deliberately.

## Resolved Questions

### Sanctum gets its own accent hue, distinct from status colours

The ember (`#b8331c`) is not reused as the interactive accent. Sanctum's warm palette already spends its warm hues on meaning — ember for `--live` and error, amber `#b27318` for warn, green `#3e7a4a` for ok — so an interactive accent drawn from that range is ambiguous by construction: a focused field and a failed upload would share a hue.

The accent token is therefore constrained rather than picked by eye:

- At least 60° of hue separation from `--live`, `--status-err` and `--warn`, and distinguishable from `--ok`.
- Legible on parchment surfaces in light and on the near-black surfaces in dark, meeting 4.5:1 as text and 3:1 as a focus ring or interface boundary in **both** schemes.
- Distinguishable under the common colour-vision deficiencies, since it must not be confusable with the error ember specifically.

A cool blue reads as interactive against warm parchment and is the furthest usable point from the warm status range, so the theme pack starts from a deep lapis in light and a lightened lapis in dark, with the final values fixed by the contrast check in task 2.4 rather than asserted here. The reference's inverted-ink primary button is unaffected — it stays as the signature it is; the accent governs focus rings, active nav state, selection and links.

### Presenter and caption output surfaces are out of scope entirely

The presenter view and the caption overlay are **projection output**, not application chrome. They render onto a sanctuary screen and into OBS, where their appearance is a fixed presentation design driven by legibility at distance and by what the receiver expects — not by the operator's chosen app theme. They are not touched by this change: no tokens, no theme attributes, no light/dark coupling. A theme switch in the app must have no observable effect on what the congregation or the stream sees.

The distinction matters because two route names look similar and only one is in scope:

| Route | Nature | Scope |
| --- | --- | --- |
| `/presentations` | Operator control screen in the app shell | **Migrated** |
| `/presenter` | Presenter output rendered to the projection surface | **Untouched** |
| `/obs-caption` | Caption *configuration* screen in the app shell | **Migrated** |
| `/caption` | Caption overlay rendered into OBS | **Untouched** |

This also removes the risk that reduced-transparency or dark-scheme tokens leak into a surface that is composited over video.

### `classic` stays as a permanent option, in one app build

Two app builds were considered to avoid carrying both designs. The measurements say the opposite:

| | |
| --- | --- |
| macOS app bundle | 104 MB |
| Whole frontend build | 944 KB (0.9% of the bundle) |
| `app.css` today | 4.3 KB |
| A second theme pack | ~5 KB |

A theme pack is CSS custom property declarations — no components, no logic, no assets. Classic uses system fonts, so it adds **no** font weight; the ~200 KB of woff2 belongs to Sanctum and ships either way. Carrying classic costs roughly **5 KB in a 104 MB bundle**.

Building two versions of the app would duplicate the 104 MB binary — embedded Postgres, the Axum server, all connectors — to avoid 5 KB of CSS. It would add roughly 104 MB of distribution, double the release, signing, notarisation and update surface, and split the test matrix, in exchange for a saving too small to measure. The premise that two designs mean duplicated size does not hold here, because the design system deliberately makes a theme *data* rather than code — that is the whole point of the token architecture.

So: one app, both theme packs, switchable at runtime. Task 10.7 changes from "remove classic" to "keep classic as a supported permanent option", which also means the rollback path in the migration plan never expires and the theme selector stays visible rather than self-hiding.

If a single-theme build is ever genuinely wanted, the cheap version is build-time *theme* selection inside the one app — a `METOCAST_THEME` variable making `scripts/build-tokens.mjs` emit only the selected pack and include fonts only when Sanctum is among them, mirroring how `METOCAST_UI` already works in `scripts/build-ui.mjs`. That gets a slimmer build with no second application. It is recorded as available, not built now, because nothing currently justifies it.

## Open Questions

- None outstanding.
