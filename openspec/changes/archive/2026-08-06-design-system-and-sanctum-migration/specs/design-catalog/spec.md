## ADDED Requirements

### Requirement: Catalog route
The application SHALL provide a catalog route that renders the design system's foundations and its full component inventory. The catalog SHALL be reachable in development and SHALL NOT be linked from the application's primary navigation.

#### Scenario: Catalog reachable in development
- **WHEN** a developer navigates to the catalog route while running the app
- **THEN** the catalog renders the token foundations and the component inventory

#### Scenario: Catalog absent from primary navigation
- **WHEN** the application's navigation is inspected
- **THEN** no navigation entry links to the catalog

### Requirement: Foundations documentation
The catalog SHALL document the token foundations — colour roles, typography scale and roles, spacing scale, radii, border widths, elevation, motion durations and easings, and z-index layers — showing for each token its name and its resolved value under the currently previewed theme and colour scheme.

#### Scenario: Token values shown for the previewed theme
- **WHEN** the catalog is previewed under a given theme and colour scheme
- **THEN** each documented token displays its name and the value it resolves to under that combination

### Requirement: Component inventory with variants and states
The catalog SHALL render every design-system component across its documented variants and states, including default, hover, focus-visible, active, disabled, loading, error, and empty where each applies. Every component in the library SHALL appear in the catalog.

#### Scenario: All components present
- **WHEN** the catalog's component inventory is compared against the design system's public exports
- **THEN** every exported component appears in the catalog
- **AND** a component missing from the catalog is reported as a failure

#### Scenario: States rendered side by side
- **WHEN** a component with multiple states is viewed in the catalog
- **THEN** each documented state is rendered and labelled

### Requirement: Theme and scheme preview
The catalog SHALL allow previewing its contents under any registered theme pack in both the light and the dark colour scheme, independently of the user's own appearance settings, and SHALL support viewing schemes side by side.

#### Scenario: Preview does not change user settings
- **WHEN** a developer previews a theme or scheme in the catalog
- **THEN** the catalog contents render under the previewed combination
- **AND** the user's saved appearance settings are unchanged

#### Scenario: Light and dark compared together
- **WHEN** the side-by-side view is enabled
- **THEN** the same component is shown simultaneously in the light and dark schemes of the previewed theme

### Requirement: Viewport preview
The catalog SHALL allow previewing its contents at mobile and desktop viewport widths, so that each component's responsive behaviour and touch target sizing can be verified without resizing the application window.

#### Scenario: Component previewed at mobile width
- **WHEN** the catalog is previewed at a mobile viewport width
- **THEN** each component renders in its mobile form
- **AND** components that reflow below the mobile breakpoint are shown in their reflowed form

### Requirement: Duplicate concept detection
The catalog SHALL group components by the concept they render, so that two components covering the same concept are visible as an inconsistency.

#### Scenario: Two components rendering one concept are visible
- **WHEN** the catalog is reviewed
- **THEN** components are grouped by concept
- **AND** any concept covered by more than one component is identifiable

### Requirement: Accessibility documentation in the catalog
The catalog SHALL present, for each interactive component, its keyboard interaction map and the ARIA roles and attributes it applies, sourced from that component's written specification.

#### Scenario: Keyboard map shown
- **WHEN** an interactive component's catalog entry is viewed
- **THEN** its keyboard interaction map and ARIA semantics are displayed

### Requirement: Contrast report
The catalog SHALL compute and display a contrast report for the documented text-on-surface and interface-element token pairings across every theme and colour scheme, showing each measured ratio and whether it meets the applicable WCAG 2.2 AA threshold, and clearly flagging failures.

#### Scenario: Failing pairing is flagged
- **WHEN** a token pairing falls below its applicable AA threshold under some theme and scheme
- **THEN** the report flags that pairing with its measured ratio, threshold, theme and scheme

#### Scenario: Report covers all combinations
- **WHEN** the contrast report is generated
- **THEN** it includes every documented pairing for every registered theme in both colour schemes

### Requirement: Catalog excluded from production bundles
The catalog SHALL NOT be included in production application builds, and SHALL NOT contribute its code to the shipped bundle.

#### Scenario: Catalog absent from a production build
- **WHEN** a production build is produced and its output inspected
- **THEN** the catalog route is not present and its modules are not bundled
