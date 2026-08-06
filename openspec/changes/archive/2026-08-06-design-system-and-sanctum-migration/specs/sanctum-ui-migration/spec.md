## ADDED Requirements

### Requirement: Design reference coverage is audited before migration
Every application surface SHALL be assessed against the design reference and recorded as fully covered, partially covered, or uncovered. Every element of user-interface vocabulary the reference does not provide SHALL be recorded as a gap, and every reference pattern that conflicts with currently supported behaviour SHALL be recorded as a breaking change with a decision.

#### Scenario: Coverage is recorded for every surface
- **WHEN** the migration begins
- **THEN** each application route and shared component carries a coverage verdict against the design reference
- **AND** each uncovered surface names the design-system components it will be composed from

#### Scenario: Gap is resolved in the system before the surface needs it
- **WHEN** a surface requires user-interface vocabulary the reference does not provide
- **THEN** that component is designed and added to the design system with a specification and catalog entry
- **AND** it exists before the surface that consumes it is migrated

#### Scenario: Breaking change carries a decision
- **WHEN** a reference pattern conflicts with behaviour the application supports today
- **THEN** it is recorded with its impact and an explicit decision to adopt, adapt or reject it
- **AND** a rejected or adapted pattern states the reason

### Requirement: Reference appearance is reproduced without reproducing its markup
Migration SHALL take the reference's visual language and SHALL NOT reproduce its markup or interaction implementation. Where the reference implements a control without the roles, keyboard support or focus indication required by the design system, the design system's compliant implementation SHALL be used instead, even where this changes the control's behaviour relative to the reference.

#### Scenario: Inaccessible reference control is not copied
- **WHEN** the reference implements a control without the roles, keyboard handling or focus indication its pattern requires
- **THEN** the design system's compliant component is used
- **AND** the resulting surface matches the reference's appearance while meeting the accessibility requirements

#### Scenario: Repeated reference treatments are unified
- **WHEN** the reference implements the same interaction pattern more than once with differing markup
- **THEN** a single design-system component replaces all of those occurrences

#### Scenario: Deliberate deviations are recorded
- **WHEN** the design system departs from the reference's appearance to satisfy an accessibility, localisation or platform requirement
- **THEN** the deviation and its reason are recorded in the affected component's specification

### Requirement: Navigation presents four primary destinations
Primary navigation SHALL offer exactly four destinations: dashboard, events, presentations and settings. Every other application surface SHALL be reached from the settings hub or from within one of the four destinations. Live events SHALL be reached as a filter within events rather than as its own destination. Route paths SHALL NOT change as part of this restructure.

#### Scenario: Primary navigation is reduced to four
- **WHEN** the application shell is rendered
- **THEN** primary navigation offers exactly the four destinations above
- **AND** no other surface appears in primary navigation

#### Scenario: Secondary surfaces reachable from the settings hub
- **WHEN** the user opens settings
- **THEN** connectors, connect, OBS caption, OBS devices, RF/IR, queues, logs and setup are each reachable from it in a grouped structure

#### Scenario: Live events reached through events
- **WHEN** the user selects the live filter within events
- **THEN** live event content is shown without navigating to a separate destination

#### Scenario: Paths are unchanged
- **WHEN** a surface is reached after the restructure
- **THEN** its route path is the same as before the restructure

### Requirement: Responsive application shell
The shell SHALL adapt to the viewport using breakpoint tokens. Below the mobile breakpoint it SHALL present the four destinations as a bottom navigation bar with no sidebar; at or above it, as a sidebar. Both presentations SHALL be driven by one navigation implementation, not two parallel ones. Desktop window chrome behaviour SHALL apply only where it is relevant.

#### Scenario: Mobile navigation
- **WHEN** the shell is rendered below the mobile breakpoint
- **THEN** the four destinations are presented as a bottom navigation bar
- **AND** no sidebar is rendered
- **AND** the bar respects the platform's bottom safe-area inset

#### Scenario: Desktop navigation
- **WHEN** the shell is rendered at or above the mobile breakpoint
- **THEN** the four destinations are presented in the sidebar
- **AND** the sidebar retains its translucent surface and the platform's window drag and traffic-light behaviour

#### Scenario: One navigation implementation
- **WHEN** the shell's navigation is inspected
- **THEN** both presentations are produced by a single navigation implementation and a single set of destination definitions

#### Scenario: Shell adopts the active theme
- **WHEN** the design theme changes
- **THEN** the navigation, content pane and window chrome adopt the new theme's tokens
- **AND** the window remains draggable by its title area and the traffic lights remain unobstructed

#### Scenario: Navigation state is accessible
- **WHEN** a navigation destination is active
- **THEN** it is identified as current to assistive technology, and is distinguishable by more than colour alone

### Requirement: Every migrated surface is usable on mobile
Every migrated surface SHALL be usable from a mobile viewport through to a desktop viewport. No migrated surface SHALL require horizontal page scrolling, a pointer, or a minimum window width to be operable.

#### Scenario: Surface works at mobile width
- **WHEN** a migrated surface is rendered at a 360px-wide viewport
- **THEN** its content and every action remain reachable and operable
- **AND** the page does not scroll horizontally

#### Scenario: Verified at both ends of the range
- **WHEN** a surface's migration is completed
- **THEN** it is verified at a mobile viewport and at a desktop viewport, in both colour schemes

### Requirement: A single unified notification system
The application SHALL have exactly one notification system, providing tiered severity, persistence, per-notification actions, grouping and expandable remediation detail, presented as transient notifications, a notification centre and an unread indicator. Existing parallel notification mechanisms SHALL be removed rather than retained alongside it. A dedicated errors destination SHALL NOT coexist with the notification centre.

#### Scenario: One source, several views
- **WHEN** a notification is raised
- **THEN** it appears through the single notification system
- **AND** the transient notification, the notification centre and the unread indicator are all views over the same source

#### Scenario: Superseded mechanisms removed
- **WHEN** the unified system is in place
- **THEN** the previous toast dependency is removed and no call site raises notifications through it
- **AND** the separate errors destination no longer exists as its own surface

#### Scenario: Remediation behaviour preserved
- **WHEN** a connector error is raised
- **THEN** its message, remediation detail, re-check action and fix flow remain available through the notification centre
- **AND** no capability available on the previous errors page is lost

### Requirement: Every application surface is migrated
Every application route and shared component SHALL be migrated onto the design system. The migration SHALL cover the dashboard, events (list, detail, create, edit), live events, presentations, OBS caption configuration, OBS devices, connect, queues, logs, errors, RF/IR, settings and setup routes, together with all shared components under the connectors, events, recordings, presentations, connect, settings and layout groups. The presenter output view and the caption overlay are excluded.

#### Scenario: Migration completeness
- **WHEN** the application's routes and shared components are enumerated
- **THEN** each one is either migrated onto the design system or explicitly recorded as out of scope with a reason

#### Scenario: Surfaces absent from the design reference
- **WHEN** a route has no counterpart in the original design reference
- **THEN** its surface is composed from existing design-system primitives and patterns
- **AND** any new component it requires is added to the design system rather than to the route

### Requirement: No ad-hoc styling in migrated surfaces
A migrated route or component SHALL NOT contain literal colour, typography, radius, shadow or border values. Route-local styles SHALL be limited to layout, expressed with spacing and sizing tokens.

#### Scenario: Migrated surface passes the styling check
- **WHEN** a migrated file is checked for literal visual values
- **THEN** no literal colour, font-size, font-family, border-radius, box-shadow or border-width value is found

#### Scenario: Unmigrated surfaces are exempt until migrated
- **WHEN** the styling check runs while migration is in progress
- **THEN** it applies only to surfaces recorded as migrated, and reports the remaining unmigrated surfaces

### Requirement: Behaviour preserved across migration
Migrating a surface SHALL NOT change its functional behaviour. Data loading, WebSocket and HTTP interactions, form validation, error handling, navigation and translated content SHALL behave as they did before migration.

#### Scenario: Functional parity after migration
- **WHEN** a migrated route is exercised through its existing user flows
- **THEN** it performs the same operations, sends the same core requests, and shows the same outcomes as before migration

#### Scenario: Existing tests continue to pass
- **WHEN** the project's checks and end-to-end tests are run after a route is migrated
- **THEN** they pass without weakening or deleting existing assertions

### Requirement: Incremental migration keeps the application usable
The migration SHALL proceed surface by surface, and the application SHALL remain fully usable at every step. Migrated and unmigrated surfaces SHALL coexist without visual or functional breakage, with the `classic` theme available as a fallback appearance while migration is in progress.

#### Scenario: Mixed state remains usable
- **WHEN** some routes are migrated and others are not
- **THEN** every route still renders correctly and remains operable in both colour schemes

### Requirement: Accessibility verified per migrated surface
Each migrated surface SHALL be verified for keyboard operability, correct focus order, visible focus, accessible names for all controls, and AA contrast, in both colour schemes.

#### Scenario: Surface accessibility check
- **WHEN** a surface's migration is completed
- **THEN** it is verified to be fully keyboard operable with a logical focus order and visible focus
- **AND** every control exposes an accessible name
- **AND** its text and interface elements meet AA contrast in both colour schemes

### Requirement: Projection output surfaces are excluded from the migration
The presenter output view and the caption overlay SHALL NOT be migrated, themed or otherwise altered by this change. They SHALL NOT consume design-system tokens, SHALL NOT receive the theme or colour-scheme attributes, and their appearance SHALL be unaffected by any appearance setting. The operator-facing presentation control screen and caption configuration screen SHALL be migrated as ordinary application surfaces.

#### Scenario: Theme change does not reach projection output
- **WHEN** the user changes the design theme or colour scheme
- **THEN** the presenter output view and the caption overlay render exactly as before, in their fixed presentation design
- **AND** no theme or colour-scheme attribute is applied to their documents

#### Scenario: Control screens are still migrated
- **WHEN** the presentations control screen and the caption configuration screen are migrated
- **THEN** they adopt the design system like any other application surface
- **AND** the output surfaces they control remain untouched

#### Scenario: Reduced-transparency and dark tokens do not leak into composited output
- **WHEN** the caption overlay is composited over video
- **THEN** its appearance is unchanged by the application's colour scheme, transparency or reduced-transparency handling

### Requirement: Both theme packs remain permanently available
Both theme packs SHALL remain available and user-selectable after the migration completes. The `classic` pack SHALL NOT be removed, and the design-theme selector SHALL remain visible. Both packs SHALL be shipped in a single application build; the application SHALL NOT be duplicated into separate per-design builds.

#### Scenario: Classic still selectable after migration
- **WHEN** every in-scope surface has been migrated
- **THEN** both theme packs are still offered in the appearance settings
- **AND** selecting classic renders every migrated surface in the pre-migration appearance

#### Scenario: Single build carries both designs
- **WHEN** the application is built
- **THEN** one build contains both theme packs
- **AND** no separate per-design application build is produced

#### Scenario: Stored selection of an unregistered theme
- **WHEN** the persisted design theme names a theme that is not registered
- **THEN** the application falls back to the default registered theme without error
