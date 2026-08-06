## Why

The Sanctum design exists only as a prototype at `~/workspace/ui/sermon-helper-svelte` — six screens running entirely on fixture data, with no backend, no accessibility and a phone-frame mockup around it. It is a design, not an application.

Two changes prepare the ground for turning it into one. `sanctum-design-system` delivers the component vocabulary, measured from the reference. `split-into-two-uis` delivers the workspace those components can be assembled in — `ui/sanctum` as a registered peer of `ui/classic`, with `packages/core-client` as the boundary to the core. Neither builds a screen.

This change builds the application: every screen and every feature the reference presents, running on live data from the real core.

## What Changes

- **Build `ui/sanctum` as a working application** — its own shell, navigation and layout, composed from `packages/design-system` and reaching the core only through `packages/core-client`.
- **Implement all six reference screens in full**: dashboard, events, event editor, presentations, settings and connectors. Every control, state, section and affordance the reference presents is implemented, including the ones easy to overlook — the event editor's live auto-title preview with its per-field tag indicators, the 500ms debounced scripture lookup, the presenter client list with connection and pong ages, the preload queue's five slots, the connectors' expand-in-place detail with per-connector field sets.
- **Replace every fixture with live data.** The reference's `data.js` — events, file library, connector settings, slides, clients, folders — is deleted, not ported. Each screen is wired to core operations over HTTP and to core events over WebSocket.
- **Implement the tiered notification system** the reference designs: severity tiers, persistence, per-notification actions, grouped notifications, expandable remediation steps, a notification centre and an unread indicator carrying the worst active tier.
- **Resolve every backend gap explicitly.** Where the reference displays something the core cannot supply — its dashboard shows viewer count, bitrate and dropped-frame percentage — the gap is implemented, omitted, or shown as unavailable. Nothing is rendered as a plausible-looking invented value.
- **Exclude the prototype's scaffolding**: the 402×874 phone frame, the simulated iOS status bar and the tweaks panel are mockup apparatus, not product.
- Sanctum covers the reference and no more. Queues, logs, OBS devices, OBS caption, RF/IR, connect and setup remain reachable only in `ui/classic`. **Not** a feature-parity exercise.

## Capabilities

### New Capabilities
- `sanctum-shell`: The application shell — responsive navigation, layout, colour-scheme control, window transparency handling, and the hosting of overlays — as the frame every screen renders inside.
- `sanctum-dashboard`: The at-a-glance screen: current broadcast state with live telemetry, the next scheduled event, and quick actions.
- `sanctum-events`: Event browsing and scheduling — the filtered, searchable event list with its selection detail, and the full event editor including scripture lookup, auto-title composition, privacy and recording options.
- `sanctum-presentations`: Presentation control — deck loading, slide transport, the preload queue, presenter distribution with connected-client visibility, and presentation folder configuration.
- `sanctum-settings`: Application configuration and connector management — language, mode, accounts, scheduled jobs, appearance, updates, and per-connector configuration with live status and remediation.
- `sanctum-notifications`: The tiered notification system — transient notifications, the notification centre, and the unread indicator, driven by core events.

### Modified Capabilities
<!-- None. This change adds screens to a UI that `split-into-two-uis` establishes; it changes no existing capability's requirements. -->

## Impact

- **`ui/sanctum`**: routes, layouts, screen composition, view state and the wiring to core operations and events. No component definitions — those come from the design-system package.
- **Dependencies**: `packages/design-system` for every visual element; `packages/core-client` for every core interaction. No direct `fetch`, `WebSocket` or desktop-host use.
- **Possible core work**: whatever the reference-to-core audit determines is missing. Broadcast telemetry is the known candidate; the audit precedes screen work so this is scoped before it becomes a blocker.
- **Locales**: Sanctum-specific strings added to the shared catalogues; both supported locales covered.
- **Prerequisites**: `sanctum-design-system` and `split-into-two-uis` must both be complete. This change assembles what they deliver.
- **Not affected**: the Rust core's existing contracts, OpenAPI, Bruno, Companion, `ui/classic`, and the `/presenter` and `/caption` projection surfaces.
