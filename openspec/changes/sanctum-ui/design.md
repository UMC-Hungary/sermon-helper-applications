## Context

This is the third change in a chain. `sanctum-design-system` delivers `packages/design-system` — components measured from the reference, with Storybook. `split-into-two-uis` delivers the workspace — `ui/sanctum` registered beside `ui/classic`, with `packages/core-client` as the only route to the core. Neither builds a screen. This one does.

The design reference at `~/workspace/ui/sermon-helper-svelte` is six screens and 28 components running on `lib/data.js` — hard-coded events, a file library, connector settings, slides, presenter clients and folders — plus `lib/stores.js`, which fabricates notification scenarios on demand. Nothing in it talks to anything. Its interactions are real enough to demonstrate intent: a 500ms debounced scripture lookup, an auto-title recomposed from four fields with per-field indicators, a five-slot preload queue, connector rows expanding in place with per-connector field sets, a notification model with tiers, persistence, actions, grouping and ordered remediation steps.

The reference is therefore precise about *what* each screen does and silent about *where anything comes from*. Every screen needs rewiring, and some of what it displays may have no counterpart in the core at all — its dashboard reports viewer count, bitrate and dropped-frame percentage, and whether the core supplies those is unverified.

`ui/classic` already implements most of the underlying operations against the same core, so the API surface is largely known to exist. What is not known is the mapping for the parts of the reference that classic has no equivalent for.

## Goals / Non-Goals

**Goals:**
- Every screen and every feature of the reference, implemented — including the details that are easy to skip because they are small.
- Live data throughout, with the reference's fixtures deleted rather than ported.
- Backend gaps identified before screen work and resolved by decision, not by invention.
- The notification model the reference designs, driven by real core events.

**Non-Goals:**
- Building components. They come from `packages/design-system`; a screen needing something the library lacks adds it there.
- Feature parity with `ui/classic`. Queues, logs, OBS devices, OBS caption, RF/IR, connect and setup stay classic-only.
- The reference's phone frame, simulated status bar or tweaks panel.
- Changing the core's existing contracts, beyond whatever the gap audit determines is genuinely missing.
- Touching `/presenter` or `/caption`.

## Decisions

### The data audit comes before the screens

The first substantial task is not a screen. It is walking all six reference screens and listing every value displayed and every action offered, then mapping each to a core operation or event through `packages/core-client`.

This is deliberately front-loaded because the alternative is discovering a gap halfway through a screen, at which point the cheap resolution — quietly rendering something plausible — is also the most tempting one. The reference is fluent fiction: it shows `1,284` viewers, `6.2 Mb/s`, `0.00%` dropped, latency figures per presenter client, "connected 42m ago · pong 2s ago". Some of that the core certainly provides, some it may not, and the difference is invisible until asked.

Each gap gets one of three recorded decisions: implement the core capability, omit the element, or show it as unavailable. The spec forbids the fourth option. An "unavailable" state is honest and cheap; an invented number is a bug that looks like a feature and survives to production because nothing about it looks wrong.

### Screens are built in dependency order, not reference order

Settings and connectors first, then dashboard, events, event editor, presentations.

Settings and connectors are the densest use of the row-and-list idiom the whole design rests on, and they exercise the most component variety per screen — toggles, segmented controls, fields, expand-in-place detail, status indication, forms. Building them first surfaces gaps in the design-system library while there is still the most work left to absorb the correction. They are also the least dependent on core capabilities that may not exist: connector status, configuration and jobs are all things `ui/classic` already does.

The dashboard is deliberately *not* first, despite being the reference's first screen. It has the highest concentration of possibly-unavailable data, and building it early would mean either blocking on backend work or making the invention decision under pressure.

The event editor is the single most intricate screen — six sections, debounced lookup, live composition, four distinct input patterns — and benefits from the component library having settled.

### Notifications replace nothing, because there is nothing to replace

`ui/classic` uses a toast library plus a separate errors page. Sanctum is a new application and inherits neither. It implements one system: transient notifications, a centre, and an unread indicator, all views over one source fed by core events.

The reference's model is richer than a toast — tiers, persistence, actions, grouping, ordered remediation — and the core already emits what it needs. `ui/classic`'s `ConnectorError` carries `connectorId`, `message`, `infoMarkdown` for remediation steps and `timestamp`; the tier and persistence dimensions are the addition.

Two behaviours matter more than they look. A notification must not steal focus — the reference never had a keyboard user to disturb, but an operator mid-task during a live service certainly must not be yanked away. And announcements must match severity: a reconnection attempt is polite, a failed stream is assertive.

### Realtime is the default, polling the exception

The reference has no concept of data changing underneath it. In the real application most of these screens are watching something live: broadcast state, connector status, presenter clients connecting and dropping, slide position moving under someone else's control, upload progress.

So each screen subscribes to the core events relevant to it rather than fetching once on mount. The specs state this per screen as "without a manual refresh" because it is the difference between a control surface and a report. It also means every screen needs a defined behaviour for connection loss, which the shell owns centrally rather than each screen inventing.

### Fidelity is judged screen by screen against the reference

Each screen is reviewed against its reference counterpart as it is completed, not in a batch at the end. The design system's mechanical value-diffing covers components; screens are compositions, and composition drift — a section in the wrong order, a missing indicator, a summary omitted because it seemed decorative — is caught by comparison, not by tooling.

The reference's own audit, carried in the archived `design-coverage.md`, enumerates what each screen contains. That list is the checklist.

## Risks / Trade-offs

- **The reference's fluency invites invented data.** Its fixtures are plausible enough to feel like requirements, and the honest alternative — an "unavailable" state — looks unfinished by comparison. → The audit resolves each gap by explicit recorded decision before the screen is built, and the spec makes invention a defect rather than a judgement call.
- **The gap audit may produce real backend work,** which this change is not scoped to absorb. → Identify it early, in its own task group, so it can be scheduled as core work rather than discovered as a blocker mid-screen.
- **Screen work will find holes in the design system** however carefully it was measured — the reference never had to render a loading state, an error, or an empty list. → The rule holds: add it to the package with its specification and stories, never locally in a screen. Expect this to slow the first two screens noticeably.
- **Six screens is a lot of surface for a partial application.** A user switching to Sanctum loses seven routes' worth of functionality. → Out of scope to fix here, but the UI selector must describe coverage honestly, which `split-into-two-uis` owns.
- **The event editor concentrates the most risk** — debounced lookup with out-of-order responses, live composition across four fields, native pickers, a custom visibility selector. → Build it after the component library has settled, and treat its lookup race explicitly rather than assuming responses arrive in order.
- **Realtime everywhere multiplies the states to test.** Every screen now has loading, loaded, updating, disconnected and error states rather than just content. → The shell centralises connection state so screens handle their own data states only.

## Migration Plan

1. **Data audit** — enumerate every value and action across the six screens; map each to a core operation or event; record a decision for every gap. Output is a table, reviewed before anything is built.
2. **Application skeleton** — shell, navigation, colour scheme, transparency handling, overlay hosting, core connection and its loss/recovery behaviour.
3. **Notifications** — the system and its three views, wired to core events. Built early because every subsequent screen reports failures through it.
4. **Settings and connectors** — densest component usage, least uncertain data.
5. **Dashboard** — after the telemetry decisions from step 1 are settled.
6. **Events, then the event editor.**
7. **Presentations.**
8. **Completion** — screen-by-screen fidelity review against the reference, accessibility and responsive verification, both locales, and confirmation that no fixture data remains anywhere.

**Rollback:** Sanctum is only reachable when deliberately bundled, and `ui/classic` remains the default. Incomplete work ships nothing.

## Open Questions

- Does the core supply broadcast telemetry — viewer count, bitrate, dropped frames — or is that reference invention? Determines whether step 1 produces backend work or omissions.
- Does the core expose presenter client latency and last-response age, which the reference shows per connected client?
- Should Sanctum offer the active-UI selector itself, so a user who switches into it can switch back without editing configuration? Strongly implied, but it is a `split-into-two-uis` concern that this change's settings screen would host.
