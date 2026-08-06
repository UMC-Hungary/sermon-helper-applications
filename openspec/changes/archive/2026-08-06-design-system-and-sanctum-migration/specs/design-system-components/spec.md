## ADDED Requirements

### Requirement: Component library structure
The design system SHALL organise its components into primitives (single-purpose, composable building blocks) and patterns (compositions of primitives that encode a recurring layout or interaction). Application code SHALL import components from the design system's public entry point rather than reaching into internal files.

#### Scenario: Public entry point
- **WHEN** an application route imports a design-system component
- **THEN** the import resolves through the design system's public entry point
- **AND** no application file imports a design-system internal module directly

### Requirement: Component inventory covers the application
The library SHALL provide components sufficient to build every application surface without route-local styling. The inventory SHALL include, at minimum: Button, IconButton, Toggle, Checkbox, RadioGroup, Segmented control, TextField, TextArea, Select, Dialog, Sheet, Tabs, Tooltip, Toast, Badge, StatusDot, List, Row, SectionLabel, PageHeader, Stat, EmptyState, Spinner, ProgressBar, and Table.

#### Scenario: Surface built without local styles
- **WHEN** a migrated route is inspected
- **THEN** its layout and controls are composed from design-system components and tokens
- **AND** any remaining route-local styles are layout-only and use spacing tokens

#### Scenario: Missing component is added to the system
- **WHEN** a route requires a control the inventory does not provide
- **THEN** the control is added to the design system with a specification and catalog entry
- **AND** it is not implemented as a one-off inside the route

### Requirement: Written specification per component
Every component SHALL have a written specification stored alongside the library, documenting its anatomy, props API, variants, states, the tokens it consumes, its keyboard interaction map, its ARIA roles and attributes, its accessibility acceptance criteria, and usage guidance including at least one "do" and one "don't".

#### Scenario: Specification accompanies every component
- **WHEN** the component library is enumerated
- **THEN** every component has a corresponding specification document containing all the sections listed above

#### Scenario: Component without specification is rejected
- **WHEN** a component is added without its specification document
- **THEN** the design-system completeness check fails and names the undocumented component

### Requirement: Behaviour follows WAI-ARIA Authoring Practices
Interactive components SHALL implement the keyboard interaction and ARIA semantics defined by the WAI-ARIA Authoring Practices for their pattern. Visual styling SHALL be supplied entirely by tokens and SHALL NOT alter the documented behaviour.

#### Scenario: Dialog behaviour
- **WHEN** a Dialog opens
- **THEN** focus moves into the dialog, focus is trapped within it while open, `Escape` closes it, content outside is inert to assistive technology, and focus returns to the element that opened it on close

#### Scenario: Tabs behaviour
- **WHEN** a Tabs component has focus on a tab
- **THEN** the arrow keys move between tabs, `Home` and `End` jump to the first and last tab, and the selected tab is exposed with the correct roles and `aria-selected` state

#### Scenario: Toggle behaviour
- **WHEN** a Toggle receives keyboard focus
- **THEN** `Space` changes its state, its role and checked state are exposed to assistive technology, and its accessible name is present

#### Scenario: Menu and listbox behaviour
- **WHEN** a Select or menu-like component is open
- **THEN** arrow keys move the active option, `Enter` commits it, `Escape` dismisses without committing, and the active option is communicated through the appropriate `aria-activedescendant` or focus management

### Requirement: Keyboard operability and focus visibility
Every interactive component SHALL be fully operable by keyboard alone and SHALL render a focus indicator that is visible in both colour schemes and in all theme packs. Focus SHALL NOT be removed without an equally visible replacement, and the focus indicator SHALL meet WCAG 2.2 focus appearance expectations.

#### Scenario: Full keyboard traversal
- **WHEN** a user navigates a migrated surface using only the keyboard
- **THEN** every interactive element can be reached, operated and left
- **AND** the currently focused element is visually identifiable at every step

#### Scenario: Focus indicator contrast
- **WHEN** the focus indicator is measured against its adjacent background in any theme and scheme
- **THEN** the contrast ratio is at least 3:1

### Requirement: Accessible names and state exposure
Every component that conveys meaning, accepts input, or reflects a state SHALL expose an accessible name and its current state programmatically. Meaning SHALL NOT be conveyed by colour alone.

#### Scenario: Icon-only control is named
- **WHEN** a control renders only an icon
- **THEN** it exposes an accessible name describing its action

#### Scenario: Status conveyed beyond colour
- **WHEN** a status is presented
- **THEN** it is distinguishable by text or shape in addition to colour

### Requirement: Contrast compliance
Text and meaningful non-text elements rendered by design-system components SHALL meet WCAG 2.2 level AA contrast: at least 4.5:1 for body text, 3:1 for large text, and 3:1 for user-interface components and meaningful graphical objects — in every theme pack and colour scheme, measured against the token-defined surface behind them.

#### Scenario: Contrast verified across themes
- **WHEN** the contrast of every documented text-on-surface token pairing is computed for each theme and scheme
- **THEN** every pairing meets its applicable AA threshold
- **AND** any pairing that fails is reported with its measured ratio

### Requirement: No duplicated components or purposes
The design system SHALL provide exactly one component per user-interface concept. Two components SHALL NOT render the same concept, and a concept already covered by an existing component SHALL be expressed as a variant of it rather than as a new component. Where an application surface already implements a concept the system provides, that implementation SHALL be replaced rather than left alongside it.

#### Scenario: Duplicate concept is rejected
- **WHEN** a proposed component renders a concept an existing component already covers
- **THEN** it is added as a variant of the existing component instead of as a separate one

#### Scenario: Repeated ad-hoc implementations are unified
- **WHEN** the same interaction or display concept is implemented more than once across the application or the reference
- **THEN** a single design-system component replaces every occurrence

#### Scenario: No parallel systems survive migration
- **WHEN** a design-system component supersedes an existing implementation or dependency
- **THEN** the superseded implementation is removed, not retained in parallel

### Requirement: Components are responsive and usable on mobile
Every design-system component SHALL be usable from a mobile viewport through to a desktop viewport, using breakpoints defined as tokens. Components SHALL NOT require a pointer, hover or a minimum window width to be fully operable.

#### Scenario: Component works at mobile width
- **WHEN** any component is rendered at a 360px-wide viewport
- **THEN** it remains fully legible and operable without horizontal scrolling of the page

#### Scenario: No hover-only affordance
- **WHEN** a component exposes an action or information on hover
- **THEN** the same action or information is reachable by touch and by keyboard

#### Scenario: Dense content adapts
- **WHEN** a table or other multi-column component is rendered below the mobile breakpoint
- **THEN** it reflows to a form that fits the viewport without loss of information or actions

#### Scenario: Overlays suit the viewport
- **WHEN** a dialog is presented below the mobile breakpoint
- **THEN** it is presented in a form appropriate to that viewport and remains dismissable by keyboard and by touch

### Requirement: Touch target sizing
Interactive elements SHALL present a touch target of at least 44 by 44 CSS pixels on touch-capable viewports, including where the visible control is smaller. Adjacent targets SHALL be separated so that neighbouring controls are not triggered accidentally.

#### Scenario: Small control has an adequate target
- **WHEN** a control renders smaller than 44 by 44 pixels
- **THEN** its interactive target is expanded to at least 44 by 44 pixels without changing its visual size

### Requirement: Safe area insets are honoured
Components anchored to a viewport edge SHALL respect the platform's safe-area insets so that content is not obscured by a notch, rounded corner, home indicator or system gesture area.

#### Scenario: Bottom-anchored component clears the home indicator
- **WHEN** a bottom-anchored component such as the navigation bar or a sticky action bar is rendered on a device with a home indicator
- **THEN** its content and its touch targets sit above the inset and remain fully usable

### Requirement: Components are internationalisation-safe
Components SHALL accept translated content and SHALL NOT hard-code user-facing strings. Layouts SHALL remain intact when text expands, and text SHALL remain readable when the user zooms to 200%.

#### Scenario: Translated text does not break layout
- **WHEN** a component renders a label roughly twice its English length
- **THEN** the text wraps or truncates predictably without overlapping adjacent content or being clipped inaccessibly

#### Scenario: No hard-coded user-facing strings
- **WHEN** a design-system component is inspected
- **THEN** all user-facing text arrives through props, slots or the translation layer
