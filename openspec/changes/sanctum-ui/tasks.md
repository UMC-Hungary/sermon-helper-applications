**Prerequisites:** `sanctum-design-system` (delivers `packages/design-system`) and `split-into-two-uis` (delivers `ui/sanctum` registered, and `packages/core-client`) must both be complete.

## 1. Data audit

No screens are built in this group. The output is a reviewed mapping from every reference element to a core capability.

- [ ] 1.1 Enumerate every value displayed and every action offered across all six reference screens
- [ ] 1.2 Map each to a core HTTP operation or WebSocket event exposed by `packages/core-client`
- [ ] 1.3 Resolve whether the core supplies broadcast telemetry — viewer count, bitrate, dropped-frame percentage
- [ ] 1.4 Resolve whether the core exposes presenter client latency and last-response age
- [ ] 1.5 Record a decision for every gap — implement the core capability, omit the element, or present it as unavailable — with its reason
- [ ] 1.6 Separate out any decisions that require core work, so they can be scheduled rather than discovered mid-screen
- [ ] 1.7 Review the mapping before screen work begins

## 2. Application skeleton

- [ ] 2.1 Shell layout with one navigation implementation over one set of destinations, presenting as a bottom bar below the mobile breakpoint and side navigation above it
- [ ] 2.2 Route structure following the reference's arrangement — dashboard, events, event editor reached from events, presentations, settings, connectors reached from settings
- [ ] 2.3 Colour scheme control (light/dark/auto) with persistence and the pre-paint application that prevents a mis-schemed first frame
- [ ] 2.4 Window transparency handling with tinted surfaces, opaque fallbacks, and preserved window drag and system controls
- [ ] 2.5 Overlay hosting for sheets and notifications, with correct stacking
- [ ] 2.6 Core connection through `packages/core-client`, with connection-loss messaging, readable stale content, and recovery without a manual reload
- [ ] 2.7 Locale setup against the shared catalogues; add Sanctum-specific keys for both supported locales
- [ ] 2.8 Confirm no device frame, simulated status bar or demonstration panel is present

## 3. Notifications

Built before the screens, because every screen reports failures through it.

- [ ] 3.1 Notification model — tier, persistence, actions, grouping, remediation steps, read and dismissed state — fed by core events
- [ ] 3.2 Transient notification display: capped count, newest first, dismissable, persistent items that do not self-dismiss
- [ ] 3.3 Notification content: source identity, title, body, technical detail presented as such, connection state with in-progress distinguishable from settled
- [ ] 3.4 Actions with a distinguished primary, resolving the notification when activated
- [ ] 3.5 Remediation steps, hidden by default, disclosure control stating which way it acts
- [ ] 3.6 Grouped notifications itemising each affected source within one notification
- [ ] 3.7 Notification centre: full list, clear-all, plain empty state, marking listed items read on open
- [ ] 3.8 Unread indicator carrying the count and the highest active severity, opening the centre
- [ ] 3.9 Announce to assistive technology with urgency matching severity; never steal focus; keyboard operable throughout
- [ ] 3.10 Wire connector failure, recovery, authentication expiry and reconnection events from the core

## 4. Settings and connectors

- [ ] 4.1 Settings overview summary — mode, account connection counts, enabled job count — from live state
- [ ] 4.2 Interface language selection, applied immediately and persisted
- [ ] 4.3 Application mode display and change, with consequences explained before it takes effect
- [ ] 4.4 Sign-in accounts: signed-in, awaiting-sign-in and missing-credentials states; sign-in flow; account management
- [ ] 4.5 Scheduled jobs: list with name, schedule and purpose; enable/disable; add by name and expression; reject invalid expressions with an explanation
- [ ] 4.6 Web presenter preference, sent to the core
- [ ] 4.7 Appearance control surfaced in settings
- [ ] 4.8 Version display and update check, offering an available update
- [ ] 4.9 Connector inventory grouped by category, with the live/ready/unavailable summary
- [ ] 4.10 Connector enable/disable; unsupported connectors marked, configuration retained, not enableable
- [ ] 4.11 Expand-in-place connector configuration with per-connector field sets; secrets never shown readable; save through the core
- [ ] 4.12 Encoder connect/disconnect, save-and-reconnect, destination selection gated on connection, streaming address display
- [ ] 4.13 Device discovery: scan, in-progress indication, discovered device list, empty result communicated
- [ ] 4.14 Operator summary — destination, encoder readiness, counts, streaming target, device layer, account state
- [ ] 4.15 Live status updates from core events without manual refresh

## 5. Dashboard

Requires the telemetry decisions from group 1.

- [ ] 5.1 Current broadcast state: live/off-air, indicator animating only while live, elapsed running time, broadcast name and context
- [ ] 5.2 Telemetry per the recorded decisions — sourced, omitted or shown unavailable; never invented
- [ ] 5.3 Absent-state handling when nothing is live, showing telemetry as absent rather than as zero measurements
- [ ] 5.4 Next scheduled event with date, time and destinations; empty state when nothing is scheduled
- [ ] 5.5 Quick actions — new event, immediate broadcast, presentations, connectors — with the connector action reflecting live counts
- [ ] 5.6 Loading and error states; live updates when a broadcast starts or stops

## 6. Events

- [ ] 6.1 Event list with date, time, title and destinations; header summarising period and count
- [ ] 6.2 Filters — upcoming, live, past, drafts — with the active one indicated
- [ ] 6.3 Search, with an empty state when nothing matches
- [ ] 6.4 Live and draft markers, the live indicator animating
- [ ] 6.5 Selection detail with date, time, destinations, title, status and an open-for-editing action
- [ ] 6.6 Default selection: the live event if one exists, otherwise the first listed
- [ ] 6.7 Loading, error and empty states

## 7. Event editor

- [ ] 7.1 Create and edit modes; existing values populate on edit; back returns to events
- [ ] 7.2 Persistent action area with cancel and save; cancel discards; save failure reports and retains entered values
- [ ] 7.3 Details: title with enforced maximum, live count and approach warning; date and time via native pickers showing readable long form; speaker
- [ ] 7.4 Scripture: primary and secondary references, ranked and labelled
- [ ] 7.5 Debounced passage lookup with translation display; unrecognised references reported without discarding input; cleared references clearing their display
- [ ] 7.6 Guard against out-of-order lookup responses so a superseded result cannot overwrite a later one
- [ ] 7.7 Description with live count against its limit
- [ ] 7.8 Generated title preview, recomposing live from date, primary reference, title and speaker; per-field populated indicators; length warning; guidance when nothing is composed yet
- [ ] 7.9 Publishing privacy — public, unlisted, private — each explaining its effect
- [ ] 7.10 Recording: automatic upload with its effect explained; recording visibility including the deferred unlisted-then-public option
- [ ] 7.11 Create and update through the core; new events appear in the list

## 8. Presentations

- [ ] 8.1 Mode selection between web presenter and external application bridge, with mode-specific panels shown only in their mode, and a loaded-deck indication
- [ ] 8.2 Transport: first, previous, next, last, stop/unload, and start in external mode; accessible names on every control
- [ ] 8.3 Position against total, loaded deck name, status describing mode and readiness; bounds respected; controls requiring a deck unavailable without one
- [ ] 8.4 Deck search with results showing name and folder; opening a deck loads it and resets position; empty state on no match
- [ ] 8.5 Preload queue: fixed slots, filled and empty states, open and clear per slot, loaded slot distinguished, queueing unavailable when full
- [ ] 8.6 Presenter address with copy and copy confirmation reverting after a short interval
- [ ] 8.7 Current slide preview with position; waiting state when nothing is loaded
- [ ] 8.8 Connected client list with label, client description, connection age and last-response age; total count; empty state; live updates on connect and disconnect
- [ ] 8.9 Presentation folder configuration: view, change, persist, and reflect in deck search
- [ ] 8.10 Transport commands and position changes over the core's realtime channel

## 9. Completion

- [ ] 9.1 Screen-by-screen fidelity review against the reference, using the archived `design-coverage.md` inventory as the checklist; record any deliberate departure with its reason
- [ ] 9.2 Confirm no fixture data anywhere — no sample events, files, slides, clients, connectors, jobs, accounts or notifications
- [ ] 9.3 Confirm the reference's `data.js` and fabricated notification scenarios were not ported
- [ ] 9.4 Accessibility pass per screen: keyboard operable, logical focus order, visible focus, accessible names, AA contrast in both schemes
- [ ] 9.5 Responsive pass per screen: usable at 360px with no horizontal page scroll, 44px touch targets, safe-area insets honoured
- [ ] 9.6 Both locales verified for layout integrity and readability at 200% zoom
- [ ] 9.7 Confirm every visual element comes from `packages/design-system` and every core interaction from `packages/core-client`
- [ ] 9.8 Confirm any control added during screen work went into the design-system package with a specification and stories, not into a screen
- [ ] 9.9 Confirm `/presenter` and `/caption` are untouched and unaffected by Sanctum's appearance settings
- [ ] 9.10 Confirm no regression to the core, HTTP/WS contracts, OpenAPI, Bruno or Companion
