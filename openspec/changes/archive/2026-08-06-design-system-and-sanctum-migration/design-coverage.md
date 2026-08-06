# Design source, inventory and coverage analysis

## Source of truth

The design reference is a standalone Svelte 4 + Vite prototype at:

```
~/workspace/ui/sermon-helper-svelte/
  index.html            Google Fonts <link> — Cormorant Garamond, Inter Tight, Geist Mono
  src/styles/global.css .theme-light / .theme-dark token blocks + 4 keyframes
  src/lib/tokens.js     screen names, auth providers, connector metadata
  src/lib/stores.js     darkMode, currentScreen, streamLive, auth, notifications
  src/lib/data.js       fixture data (events, file library, connectors, slides, clients)
  src/lib/helpers.js    date/verse/visibility formatting
  src/screens/          6 screens
  src/components/       28 components
```

It is **not** in this repository and is **not** a dependency. It is a visual and interaction reference that this change reads from and reimplements. Nothing is imported from it; every component listed below is rebuilt inside `src/lib/ds/` against the accessibility and token rules in the specs.

It is a **mobile prototype**: `PhoneShell.svelte` renders a 402×874 phone frame with a notch island and home indicator, and navigation is a 4-destination bottom `TabBar`. Screens carry `@media (min-width: 760px)` and `(min-width: 1360px)` blocks that reflow to two-column desktop layouts, so the *content* layouts are desktop-ready even though the *shell* is not.

## 1. Design inventory — what the reference actually provides

### Screens (6)

| Screen | Provides |
| --- | --- |
| `Dashboard` | Serif greeting header, "now playing" broadcast card with pulsing live dot + elapsed time + 3-up stat strip, "Up next" event row, quick-action list. Two-column at ≥760px. |
| `Events` | Filter tabs (Upcoming/Live/Past/Drafts), inert search bar, date-block event rows with live dot and draft tag, sticky "Selected" detail aside. |
| `EventEditor` | Numbered `ChamberSection`s (01–06), title textarea with counter, native date/time pickers, scripture `RefField`s with debounced lookup, live auto-title preview on inverted ink, privacy `Segmented`, recording toggle + custom select, `StickyActionBar`. |
| `Presentations` | Web/Keynote mode tabs, transport dock, slide search with queue slots, presenter support panel with client list and slide preview. |
| `Settings` | 3-up overview cells, language tabs, app-mode row, sign-in account rows, connectors link, web-presenter toggle, cron job rows + draft form, appearance tabs, version row, serif footer lockup. |
| `Connectors` | 3-up overview cells, category-grouped connector rows with expand-in-place detail panels, per-connector field lists, OBS destination chooser, Broadlink discovery. |

### Components (28)

| Group | Component | Provides |
| --- | --- | --- |
| primitives | `Icon` | ~11 hand-drawn 24px stroke paths: home, calendar, slides, gear, chev, back, plus, search, first, prev/next, last |
| | `Glyph` | Brand marks: youtube, facebook, discord, obs, atem, slides, settings, twitch, vmix, broadlink |
| | `Dot` | Status dot, optional pulse animation |
| | `TextIcon` | Single-character glyph in a bordered box (`+`, `↳`, `▣`, `✓`) |
| | `DateBlock` | Stacked month/day block used as a row leading element |
| | `Row` | The core list item: leading slot, title, meta, detail, control slot, chevron; clickable and danger variants |
| | `List` | Hairline-bordered container for `Row`s |
| | `SectionLabel` | Mono uppercase 10px/2px-tracked section heading with optional right-aligned hint |
| | `PageHeader` | Eyebrow + 44px serif title, optional back button, trailing and title-trailing slots |
| | `Field` | Label + value, read-only or bound |
| | `Toggle` | Switch control |
| | `Segmented` | Multi-option selector with glyph + label + hint per option |
| | `Stat` | Label + value + optional unit |
| | `OverviewCell` | Coloured stat cell with optional divider, used in 3-up strips |
| | `HeaderIcon` | Icon button sized for the page header |
| | `Lockup` | Wordmark + flame |
| | `FlameMark` | Brand flame |
| forms | `ChamberSection` | Numbered form section with label and hint |
| | `Chamber` | Borderless input surround: mono label above, hairline below, `focus-within` treatment |
| | `ChamberNative` | Native date/time input with formatted display text |
| | `ToggleRow` | Label + sub-label + toggle |
| | `RefField` | Scripture reference input with rank badge and fetched-verse display |
| layout | `PhoneShell` | 402×874 phone frame, notch, home indicator |
| | `StatusBar` | Fake iOS status bar |
| | `TabBar` | 4-destination bottom tab bar |
| | `StickyActionBar` | Sticky bottom secondary/primary action pair |
| notifications | `ToastOverlay` / `ToastCard` | Stacked toasts, max 3, tier-styled |
| | `NotificationCenter` | Full notification list with expand, mark-read, dismiss |
| | `NotifBell` | Bell with unread count and worst-tier colouring |
| settings | `SettingsSheet` | Bottom sheet, `role="dialog"` + `aria-modal`, drag-to-close |
| | `SettingsIconButton` | Icon button with `aria-label` |
| connectors | `BroadlinkDiscovery` | Scan button, scanning state, discovered device list |
| presentations | `SlideRemoteDock` | Transport controls (first/prev/play/next/last/stop) |
| | `SlideSearch` | Filter input, results, queue action |
| | `SlideQueue` | 5 numbered slots with clear buttons |
| | `PresenterSupport` | Presenter URL + copy, connected client list with latency, slide preview |
| | `PresentationFolderSettings` | Folder path list editor |
| root | `LoginModal` | OAuth provider login/settings modal |
| | `TweaksPanel` | Prototype-only demo control panel — **not migrated** |

## 2. Coverage matrix — every app surface against the design

Verdicts: **Covered** = the design has a direct counterpart. **Partial** = the design covers some of it; the rest must be designed. **None** = no counterpart exists in the reference.

### Routes (18)

| App route | Lines | Design counterpart | Verdict |
| --- | --- | --- | --- |
| `/` dashboard | 179 | `Dashboard` screen | **Covered** |
| `/events` | — | `Events` screen | **Covered** |
| `/events/new` | — | `EventEditor` screen | **Covered** |
| `/events/[id]/edit` | — | `EventEditor` screen | **Covered** |
| `/events/[id]` detail | — | `Events` "Selected" aside only | **Partial** — the aside is a preview card, not a full detail page |
| `/presentations` | — | `Presentations` screen | **Covered** |
| `/settings` | 105 | `Settings` screen | **Covered** |
| `/connect` | 119 | — | **None** — token display, connection guide, presenter install, SSH access |
| `/live-events` | 635 | — | **None** — platform tabs, content tabs, live/upcoming/duration badges, skeleton loaders, empty state, error state, "not connected" state |
| `/queues` | 351 | — | **None** — data table, metric cards, filter row, mono/error cells |
| `/logs` | 482 | — | **None** — monospace log surface, filter row, status panels, icon buttons |
| `/errors` | 261 | `NotificationCenter` conceptually | **Partial** — the design's notification centre is a transient overlay, not a persistent error log page |
| `/obs-devices` | 565 | — | **None** — panels, device grid, listeners table, inline edit, add form, native selects, status badges |
| `/obs-caption` | 423 | — | **None** — three configuration sections with native selects |
| `/rf-ir` | 75 | `BroadlinkDiscovery` (partial) | **Partial** — discovery is covered; command list, learn/import/code-entry dialogs are not |
| `/setup` | 249 | — | **None** — first-run wizard, server/client radio group, client fields, error surface |
| `/caption` | — | — | **Out of scope** — OBS overlay, projection output, fixed presentation design |
| `/presenter` | — | — | **Out of scope** — presenter output, projection surface, fixed presentation design |

**Route coverage: of the 16 in-scope routes, 6 are fully covered (38%), 3 are partial, 7 have nothing.** `/presenter` and `/caption` are excluded entirely — they are projection output, not application chrome, and no appearance setting may affect them. Note the near-namesakes that *are* in scope: `/presentations` (operator control screen) and `/obs-caption` (caption configuration screen).

### Shared components (35)

| App component group | Design counterpart | Verdict |
| --- | --- | --- |
| `events/EventCard`, `EventList` | `Row` + `DateBlock` + `List` | **Covered** |
| `events/CreateEventForm` | `Chamber*` + `StickyActionBar` | **Covered** |
| `events/BibleSuggestions` | `RefField` | **Covered** |
| `settings/*` (5 components) | `Settings` screen rows | **Covered** |
| `connectors/ConnectorSettingsBlock` | `Connectors` expand-in-place row | **Covered** |
| `connectors/ConnectorStatusBadge` | `Dot` | **Partial** — dot only, no badge form |
| `connectors/ConnectorDashboardWidget` | Dashboard quick-actions list | **Partial** |
| `connectors/ConnectorFixModal`, `ReLoginModal` | `LoginModal`, `SettingsSheet` | **Partial** — modal shell exists; these carry remediation flows the design never shows |
| `connectors/broadlink/DiscoveryPanel` | `BroadlinkDiscovery` | **Covered** |
| `connectors/broadlink/DeviceList`, `CommandList` | `List` + `Row` | **Partial** |
| `connectors/broadlink/LearnDialog`, `CodeEntryDialog`, `ImportDialog` | — | **None** — three dialogs with live-capture state |
| `connect/ConnectionGuide`, `TokenDisplay`, `PresenterInstallCard`, `SshAccessCard` | — | **None** |
| `recordings/*` (5 components) | — | **None** — recording list, create form, assign dialog, upload modal with progress |
| `presentations/SlideEditorModal` | — | **None** — slide editing is absent from the design |
| `layout/NavConnectors`, `NavErrorBadge` | `TabBar` / `NotifBell` | **Partial** — sidebar-footer affordances the design has no place for |
| `layout/ConnectorInit`, `ReLoginHandler`, `UpdateChecker` | — | N/A — behavioural, no UI |

## 3. Component gaps — vocabulary the design has no answer for

These must be **designed**, not ported. Each becomes a design-system component authored in the Sanctum idiom (hairline, square, mono micro-labels, serif for display only) but with no reference to copy from.

| # | Missing | Needed by | Note |
| --- | --- | --- | --- |
| G1 | **Data table** | queues, obs-devices | The design's densest structure is a `Row` list. Real tables need column headers, alignment, sortability, overflow and a mobile reflow — none of it exists in the reference. |
| G2 | **Select / listbox** | obs-caption (3), obs-devices (2), event editor | The reference has one hand-rolled `.select` in `EventEditor` — a `<button>` toggling a `<div>` of buttons, with no roles, no arrow-key handling, no focus return. It cannot be used as-is. |
| G3 | **Skeleton loader** | live-events | Reference data is always present, so no loading state is ever shown. |
| G4 | **Empty state** | live-events, queues, recordings | |
| G5 | **Error state / retry surface** | live-events, logs, setup | |
| G6 | **"Not connected" state** | live-events | A distinct full-panel state with icon, title, description and a settings link. |
| G7 | **Log stream surface** | logs | Monospace scrollback, level colouring, follow-tail, copy. |
| G8 | **Progress indicator** | UploadModal, recordings | Determinate upload progress. |
| G9 | **Badge / pill** | live-events (live, upcoming, duration), obs-devices (status) | The reference uses `Dot` plus a bare mono `<em>`; there is no badge component. |
| G10 | **Real Tabs** | live-events (platform + content tabs), presentations, events filters, settings language | The reference hand-rolls tab-like UI **four separate times** in four different visual treatments — `.tabs` in Settings, `.filters` in Events, `.tabs` in Presentations, `.choice` in Connectors — none of them keyboard-accessible. One accessible Tabs component replaces all four. |
| G11 | **Radio group** | setup | |
| G12 | **Wizard / stepped flow** | setup | The `ChamberSection` numbering (01–06) suggests a visual language for it, but no stepped navigation exists. |
| G13 | **Tooltip** | throughout | No tooltip anywhere in the reference. |
| G14 | **Icon coverage** | throughout | The reference ships ~11 icons. The app uses `lucide-svelte`. Either lucide is restyled to the reference's 1.4px-stroke 24px idiom, or ~40 icons get drawn by hand. |

## 4. Breaking changes — design patterns not supported today

| # | Change | Impact | Recommendation |
| --- | --- | --- | --- |
| B1 | **Navigation model.** The reference has a 4-destination bottom `TabBar` and no sidebar. The app has a 10-item sidebar. The reference nests Connectors *under* Settings; the app has `/connect` at top level. | Six sidebar items have no destination in the reference's architecture. | **Decided: adopt the reference's architecture.** Primary navigation collapses to Dashboard, Events, Presentations, Settings. Settings becomes the hub for Connectors, Connect, OBS caption, OBS devices, RF/IR, Queues, Logs and Setup. `/live-events` folds into the Events "Live" filter the reference already defines; `/errors` folds into the notification centre (B6). **Paths do not move** — only the way there. This is also what makes mobile viable: ten items cannot become a tab bar, four can. |
| B2 | **Phone chrome.** `PhoneShell`'s 402×874 frame, notch island, home indicator and fake iOS `StatusBar` are mockup scaffolding. | None if dropped. | **Do not port the chrome** — but the `TabBar` inside it **is** adopted as the mobile navigation. The reference's base (sub-760px) styles are the mobile target and its `≥760px` / `≥1360px` reflows are the desktop target; both are migration targets. |
| B10 | **The app is not responsive; the reference is mobile-first.** The app has ~12 scattered `max-width` queries for content width and a fixed 220px sidebar. Mobile is a real target — `src-tauri/icons/android/` ships adaptive icons, `Cargo.toml` already excludes desktop-only deps for `ios`/`android`, and client mode serves the UI to any phone browser. | Every surface needs a mobile form it does not currently have. | **Adopt mobile-first.** Breakpoints become tokens; every component is responsive by construction; 44×44 touch targets; `env(safe-area-inset-*)` on bottom-anchored elements; tables reflow to stacked rows; dialogs present as bottom sheets — which `SettingsSheet` already is. |
| B3 | **No accent colour.** The reference uses `--ink` (near-black / near-white) as the primary fill and has no `--accent`. The app resolves `--accent` to macOS `AccentColor` in **79 places**. | Every primary button, active nav item and focus ring changes meaning. | **Decided.** Sanctum gets its own accent token, a deep lapis at least 60° from the ember, error and warn hues — the warm range is already spent on meaning, so a warm accent would make a focused field and a failed upload share a hue. Inverted-ink primary buttons are kept as the reference's signature; the accent governs focus rings, active nav, selection and links. |
| B4 | **Sticky action bar forms.** The reference's editor commits through a sticky bottom `Cancel` / `Primary` bar. App forms submit inline. | Changes form UX on every editing surface, not just the event editor. | Adopt for full-page editors (event new/edit, setup). Keep inline submission inside dialogs. |
| B5 | **Chamber inputs have no visible boundary.** `Chamber` renders a mono label over a borderless input with a hairline beneath and a `focus-within` treatment on the label. | A field with no visible boundary is harder to locate, and the only focus affordance is a label colour change — which fails the 3:1 focus-appearance requirement in `design-system-components`. | Keep the chamber aesthetic; add a compliant focus indicator and a visible resting boundary. This is a deliberate deviation from the reference. |
| B6 | **Notification model is richer than the app's.** The reference has tiered notifications (critical/high/medium/low), persistence, per-notification CTAs, grouped notifications, and expandable "why this happened" remediation steps. The app has `svelte-sonner` toasts (40 call sites in 8 files) **plus** a separate `/errors` page rendering the same `connectorErrors` store — two parallel notification concepts already. | Adopting it is a capability gain, and the app's `ConnectorError` store already carries the needed fields (`connectorId`, `message`, `infoMarkdown`, `timestamp`). | **Decided: adopt in full and unify.** One notification system — `svelte-sonner` removed, all 40 call sites migrated, `/errors` absorbed into the notification centre with its recheck/remediation/fix-modal behaviour intact, `NavErrorBadge` replaced by `NotifBell`. One store, several views. The system must not become the third parallel mechanism. |
| B7 | **Expand-in-place connector rows.** The reference expands connector detail inline within the list. The app uses `ConnectorSettingsBlock` cards plus modals. | Restructures the connectors surface. | Adopt — it is better than the current cards, and the app's data model already supports it. |
| B8 | **Serif display type at 44px** in page headers. | Long Hungarian titles and the 100-character event titles in the app's own fixture data will wrap to three or more lines. | Adopt with a responsive clamp and verified wrapping, not the fixed 44px. |
| B9 | **Fonts load from Google Fonts.** | Impossible offline / in embedded-asset release builds. | Self-host as woff2 subsets — already covered in tasks 3.1–3.4. |

## 5. Reference deficiencies that must be fixed, not copied

Measured across the reference's 28 components:

- **1 focus rule in the entire codebase** (`label:focus-within` in `Chamber.svelte`). Zero `:focus-visible`. A keyboard user sees no indication of focus anywhere.
- **17 total `aria-*`/`role` occurrences.** `SettingsSheet` is the only element with `role="dialog"` + `aria-modal`, and it has **no focus trap**, no focus restoration and no `Escape` handling beyond a backdrop click.
- **Every tab-like control is a plain `<button>`** with no `role="tab"`, no `aria-selected` and no arrow-key handling — in all four places it is reimplemented.
- **`Row`'s clickable variant is a `<button>` wrapping block content**, which is valid but has no `aria-current` for navigational rows.
- **Status is conveyed by `Dot` colour alone** in several places, with no text or shape alternative.
- **The ink ramp is decorative-grade.** `--ink-faint #b8b2a4` on `--card #f6f2e8` measures roughly **1.9:1** — usable for hairlines and separators, never for text. It is currently used for text (`SectionLabel` hints, `EventEditor` counters, the settings footer).

None of this is a criticism of the prototype — it is a visual reference and it does that job well. It does mean the design **cannot be ported component-for-component**. Every interactive component is rebuilt against the WAI-ARIA behaviour contracts in `specs/design-system-components/spec.md`, keeping the reference's appearance and discarding its markup.

## 6. What this means for the plan

- **Roughly a third of the in-scope app has a design to follow.** The other two thirds are composed from the design system in the Sanctum idiom. G1–G14 are the components that make that possible, and they are built *before* the surfaces that need them.
- **Projection output is excluded.** `/presenter` and `/caption` keep their fixed presentation design and are unaffected by any appearance setting.
- **Both theme packs are permanent and selectable**, shipped in one build. A theme pack is ~5 KB of custom properties against a 104 MB bundle, so keeping classic costs nothing measurable and preserves the rollback path indefinitely.
- **The reference's architecture is adopted, its phone chrome is not.** Navigation collapses to four destinations with everything else under Settings (B1); the `TabBar` becomes the mobile navigation and the sidebar the desktop one; the 402×874 frame, notch and fake status bar are dropped (B2).
- **Mobile is a first-class target** (B10), and the reference — being mobile-first with desktop reflows — supplies both layouts for the six screens it covers.
- **One notification system replaces two** (B6), and no component in the library may have a twin.
- **Four deliberate deviations** from the reference, each with a reason: an accent token (B3), compliant focus indicators (B5, §5), a darkened ink ramp for text (§5), and a responsive display type scale (B8).
- **No deferrals.** Every B-row now carries a decision; B1, B3, B6 and B10 are the ones that change the shape of the work.
