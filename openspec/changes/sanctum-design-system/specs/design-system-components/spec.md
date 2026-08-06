## ADDED Requirements

### Requirement: Components reproduce the design reference faithfully
Every component that has a counterpart in the design reference SHALL reproduce it. Its dimensions, spacing, type sizes, weights, tracking, borders and colours SHALL match the reference's measured values. A component SHALL be verified against its source before it is considered complete, and any deviation SHALL be deliberate and recorded with a reason.

#### Scenario: Component matches its source
- **WHEN** a component with a reference counterpart is completed
- **THEN** its measured values match the reference's
- **AND** the comparison is recorded

#### Scenario: Deviation carries a reason
- **WHEN** a component departs from the reference's appearance
- **THEN** the departure and its justification are recorded in the component's specification
- **AND** a departure without a recorded reason is treated as a defect

#### Scenario: Approximation is rejected
- **WHEN** a component's value differs from its source only because a nearer scale step was chosen
- **THEN** it is corrected to the measured value

#### Scenario: Fidelity is verified mechanically
- **WHEN** a component is checked against its reference counterpart
- **THEN** its implemented values are diffed against the extracted source measurements by tooling
- **AND** every reported difference is either corrected or recorded as a deliberate deviation

### Requirement: Component library structure
Components SHALL be organised as primitives and composed patterns, exported through a single public entry point. Consumers SHALL import from that entry point and SHALL NOT reach into internal files.

#### Scenario: Public entry point
- **WHEN** a consumer imports a component
- **THEN** the import resolves through the package's public entry point
- **AND** no consumer imports an internal module directly

### Requirement: Component inventory
The library SHALL provide every component the design reference defines, and every additional component the consuming screens require that the reference does not provide. The inventory SHALL include at minimum: Button, IconButton, Toggle, Checkbox, RadioGroup, Segmented, TextField, TextArea, Select, Dialog, Sheet, Tabs, Tooltip, Toast, Badge, StatusDot, Spinner, ProgressBar, Skeleton, EmptyState, ErrorState, List, Row, SectionLabel, PageHeader, Stat, OverviewCell, DateBlock, Table, and the reference's form-section, labelled-input and reference-input patterns.

#### Scenario: Reference components are all present
- **WHEN** the reference's component inventory is compared against the library
- **THEN** every reference component has an implementation

#### Scenario: Missing vocabulary is added to the library
- **WHEN** a consuming screen requires a control the library does not provide
- **THEN** the control is added to the library with a specification and stories
- **AND** it is not implemented locally in the screen

### Requirement: Exactly one component per concept
The library SHALL provide exactly one component per user-interface concept. A concept already covered SHALL be expressed as a variant rather than as a new component.

#### Scenario: Duplicate concept is rejected
- **WHEN** a proposed component renders a concept an existing component already covers
- **THEN** it is added as a variant of the existing component instead

#### Scenario: Repeated reference treatments are unified
- **WHEN** the reference implements one interaction pattern several times with differing markup
- **THEN** a single component replaces every occurrence

### Requirement: Reference appearance without reference markup
Components SHALL take the reference's appearance and SHALL NOT reproduce its markup or interaction implementation. Where the reference implements a control without the roles, keyboard support or focus indication its pattern requires, the compliant implementation SHALL be used.

#### Scenario: Inaccessible reference control is not copied
- **WHEN** the reference implements a control without the semantics its pattern requires
- **THEN** the library implements the compliant behaviour
- **AND** the component still matches the reference's appearance

### Requirement: Behaviour follows WAI-ARIA Authoring Practices
Interactive components SHALL implement the keyboard interaction and ARIA semantics defined by the WAI-ARIA Authoring Practices for their pattern.

#### Scenario: Dialog behaviour
- **WHEN** a Dialog opens
- **THEN** focus moves into it, is trapped while open, `Escape` closes it, outside content is inert to assistive technology, and focus returns to the opener on close

#### Scenario: Tabs behaviour
- **WHEN** focus is on a tab
- **THEN** arrow keys move between tabs, `Home` and `End` jump to first and last, and roles and selected state are exposed correctly

#### Scenario: Select behaviour
- **WHEN** a Select is open
- **THEN** arrow keys move the active option, `Enter` commits, `Escape` dismisses without committing, and the active option is communicated to assistive technology

#### Scenario: Toggle behaviour
- **WHEN** a Toggle has focus
- **THEN** `Space` changes its state, and its role, checked state and accessible name are exposed

### Requirement: Keyboard operability and focus visibility
Every interactive component SHALL be fully operable by keyboard and SHALL render a focus indicator visible in both colour schemes, meeting at least 3:1 contrast against its adjacent background.

#### Scenario: Full keyboard traversal
- **WHEN** a component is operated by keyboard alone
- **THEN** it can be reached, operated and left
- **AND** the focused element is visually identifiable throughout

### Requirement: Accessible names and state exposure
Every component that conveys meaning, accepts input or reflects state SHALL expose an accessible name and its current state programmatically. Meaning SHALL NOT be conveyed by colour alone.

#### Scenario: Icon-only control is named
- **WHEN** a control renders only an icon
- **THEN** it exposes an accessible name describing its action

#### Scenario: Status conveyed beyond colour
- **WHEN** a status is presented
- **THEN** it is distinguishable by text or shape as well as colour

### Requirement: Components are responsive and touch-ready
Every component SHALL be usable from a mobile viewport through to a desktop viewport, SHALL NOT require hover or a pointer to be fully operable, and SHALL present touch targets of at least 44 by 44 pixels on touch-capable viewports.

#### Scenario: Component works at mobile width
- **WHEN** a component is rendered at a 360px-wide viewport
- **THEN** it remains legible and operable without horizontal page scrolling

#### Scenario: Dense content adapts
- **WHEN** a table or multi-column component is rendered below the mobile breakpoint
- **THEN** it reflows to fit without losing information or actions

#### Scenario: Small control has an adequate target
- **WHEN** a control renders smaller than 44 by 44 pixels
- **THEN** its interactive target is expanded to at least that size without changing its visual size

### Requirement: Written specification per component
Every component SHALL have a written specification documenting its anatomy, props, variants, states, tokens consumed, keyboard map, ARIA semantics, accessibility acceptance criteria, its correspondence to the reference, and any recorded deviation.

#### Scenario: Specification accompanies every component
- **WHEN** the library is enumerated
- **THEN** every component has a specification containing all of these sections

#### Scenario: Undocumented component fails the check
- **WHEN** a component is added without its specification
- **THEN** the completeness check fails and names it

### Requirement: Components are internationalisation-safe
Components SHALL accept translated content, SHALL NOT hard-code user-facing strings, and SHALL remain intact when text expands or the user zooms to 200%.

#### Scenario: Longer text does not break layout
- **WHEN** a component renders a label roughly twice its English length
- **THEN** the text wraps or truncates predictably without overlapping or clipping adjacent content

#### Scenario: No hard-coded user-facing strings
- **WHEN** a component is inspected
- **THEN** all user-facing text arrives through props, slots or the translation layer
