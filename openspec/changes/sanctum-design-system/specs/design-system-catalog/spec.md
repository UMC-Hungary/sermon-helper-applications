## ADDED Requirements

### Requirement: Storybook is the component catalog
The design system SHALL use Storybook as its catalog and development environment. Storybook SHALL run against the package's own source, independently of any consuming application.

#### Scenario: Catalog runs standalone
- **WHEN** Storybook is started from the design-system package
- **THEN** it renders the library without requiring a consuming application to be built or running

#### Scenario: Catalog is not shipped to consumers
- **WHEN** a consuming application is built for production
- **THEN** Storybook and its dependencies are absent from the application's output

### Requirement: Every component has stories covering its variants and states
Every exported component SHALL have stories covering each documented variant and each documented state, including default, hover, focus-visible, active, disabled, loading, error and empty where applicable.

#### Scenario: All components have stories
- **WHEN** the catalog's stories are compared against the package's public exports
- **THEN** every exported component has at least one story
- **AND** a component without stories is reported as a failure

#### Scenario: States are individually rendered
- **WHEN** a component with multiple documented states is viewed
- **THEN** each state is rendered and labelled

### Requirement: Stories cover both colour schemes and both viewports
Stories SHALL be viewable in the light and dark colour schemes and at mobile and desktop viewport widths, so that scheme-specific and responsive defects are visible during development.

#### Scenario: Scheme switching in the catalog
- **WHEN** the colour scheme is switched in the catalog
- **THEN** every story re-renders in that scheme

#### Scenario: Viewport switching in the catalog
- **WHEN** the viewport is switched to a mobile width
- **THEN** stories render in their mobile form, including components that reflow

### Requirement: Reference comparison is part of the catalog
The catalog SHALL make each component's correspondence to the design reference reviewable, presenting the component alongside its recorded reference measurements and any recorded deviation.

#### Scenario: Component shown against its source
- **WHEN** a component with a reference counterpart is viewed
- **THEN** its recorded reference measurements and any deviations are presented with it

#### Scenario: Drift is reviewable
- **WHEN** a component's values no longer match its recorded reference measurements
- **THEN** the discrepancy is visible in the catalog

### Requirement: Accessibility checks run automatically
The catalog SHALL run automated accessibility checks over every story, and those checks SHALL run in continuous integration and fail the build on a violation.

#### Scenario: Violation fails the build
- **WHEN** a story contains an automatically detectable accessibility violation
- **THEN** the check fails and identifies the story and the violation

#### Scenario: Checks run on every story
- **WHEN** the accessibility checks run
- **THEN** they cover every story in the catalog

### Requirement: Foundations are documented in the catalog
The catalog SHALL document the token foundations — colour roles, typography, spacing, sizing, radii, borders, elevation, motion, layering and breakpoints — showing each token's name, its resolved value in the previewed scheme, and its recorded source in the reference.

#### Scenario: Token documented with its source
- **WHEN** a token is viewed in the catalog
- **THEN** its name, resolved value and recorded reference source are shown

### Requirement: Contrast report is published in the catalog
The catalog SHALL present the computed contrast report for every documented token pairing in both colour schemes, showing each measured ratio and whether it meets its applicable threshold, and flagging failures.

#### Scenario: Failing pairing is flagged
- **WHEN** a pairing falls below its threshold in either scheme
- **THEN** the report flags it with its measured ratio, threshold and scheme

### Requirement: Keyboard and ARIA documentation is presented
The catalog SHALL present each interactive component's keyboard interaction map and ARIA semantics, taken from that component's written specification.

#### Scenario: Keyboard map shown
- **WHEN** an interactive component is viewed in the catalog
- **THEN** its keyboard map and ARIA semantics are displayed
