## ADDED Requirements

### Requirement: Sanctum has its own shell and structure
Sanctum SHALL define its own navigation, layout and screen inventory, independent of any other registered UI. It SHALL NOT be a theme, skin or variant of another UI, and no other UI SHALL need to change for it to work.

#### Scenario: Structure is independent
- **WHEN** Sanctum is compared with another registered UI
- **THEN** its navigation, screen inventory and layout may differ entirely
- **AND** neither UI's structure constrains the other's

#### Scenario: Selectable as a registered UI
- **WHEN** a build bundles Sanctum alongside another UI
- **THEN** Sanctum can be chosen as the active UI and loads on next start

### Requirement: Responsive navigation from one definition
The shell SHALL present its destinations as a bottom navigation bar below the mobile breakpoint and as a persistent side navigation at or above it. Both presentations SHALL derive from one navigation implementation over one set of destination definitions.

#### Scenario: Mobile presentation
- **WHEN** the shell renders below the mobile breakpoint
- **THEN** destinations are presented as a bottom navigation bar
- **AND** it respects the platform's bottom safe-area inset

#### Scenario: Desktop presentation
- **WHEN** the shell renders at or above the mobile breakpoint
- **THEN** destinations are presented as side navigation

#### Scenario: One implementation
- **WHEN** the shell's navigation is inspected
- **THEN** both presentations are produced by a single implementation over a single set of destinations

#### Scenario: Current destination is exposed
- **WHEN** a destination is active
- **THEN** it is identified as current to assistive technology
- **AND** it is distinguishable by more than colour alone

### Requirement: Screens are reachable as the reference arranges them
Navigation SHALL follow the design reference's arrangement: the dashboard, events and presentations as primary destinations, with settings as a primary destination from which connector management is reached. A screen SHALL be reachable by the same path the reference implies.

#### Scenario: Connectors reached through settings
- **WHEN** the user opens settings
- **THEN** connector management is reachable from it
- **AND** it is not presented as a separate primary destination

#### Scenario: Event editor reached from events
- **WHEN** the user creates or opens an event for editing
- **THEN** the editor is reached from the events screen, with a back affordance returning there

### Requirement: Colour scheme control
The shell SHALL offer light, dark and automatic colour schemes. Automatic SHALL follow the host or system appearance and update live. The choice SHALL persist across restarts and SHALL be applied before the first painted frame.

#### Scenario: Explicit scheme overrides the system
- **WHEN** the user selects light or dark
- **THEN** that scheme applies and persists when the system appearance changes

#### Scenario: Automatic follows the system live
- **WHEN** automatic is selected and the system appearance changes
- **THEN** the interface switches scheme without a restart

#### Scenario: No mis-schemed first frame
- **WHEN** the application starts with dark previously selected
- **THEN** the first painted frame is already dark

### Requirement: Window transparency preserved and themed
Where the desktop shell provides a translucent window, Sanctum SHALL preserve that translucency and tint its surfaces with its own palette. It SHALL render opaque surfaces where translucency is unavailable or the system requests reduced transparency, and SHALL keep the platform's window drag and control affordances working.

#### Scenario: Translucency retained and tinted
- **WHEN** Sanctum runs in a translucent desktop window
- **THEN** the window remains translucent with its surfaces tinted by Sanctum's palette
- **AND** the window remains draggable and its system controls unobstructed

#### Scenario: Opaque fallback
- **WHEN** translucency is unavailable or reduced transparency is requested
- **THEN** surfaces render opaque and the interface remains legible in both schemes

### Requirement: Shell hosts overlays
The shell SHALL host the application's overlays — transient notifications, the notification centre, and modal sheets — so that a screen can raise one without owning its placement or stacking.

#### Scenario: Overlay raised from a screen
- **WHEN** a screen raises a modal sheet or a notification
- **THEN** the shell presents it above screen content in the correct stacking order
- **AND** the screen does not position or layer it itself

### Requirement: Core connection state is handled
The shell SHALL establish the core connection through the shared client package and SHALL communicate connection loss to the user, keeping already-loaded content readable while disconnected and recovering without a manual reload.

#### Scenario: Connection lost
- **WHEN** the core connection drops
- **THEN** the user is informed
- **AND** already-loaded content remains readable

#### Scenario: Connection restored
- **WHEN** the connection is re-established
- **THEN** displayed data returns to a current state without a manual reload

### Requirement: Prototype scaffolding is excluded
The shell SHALL NOT reproduce the design reference's device frame, simulated system status bar, or prototype-only demonstration controls.

#### Scenario: Scaffolding absent
- **WHEN** Sanctum is inspected
- **THEN** no device frame, simulated status bar or demonstration control panel is present
