## ADDED Requirements

### Requirement: Every rendering UI is a self-contained application
Each rendering UI SHALL live in its own directory under `ui/` and SHALL be a complete application in its own right, owning its dependencies, build configuration, source tree and static assets. No rendering UI SHALL live at the repository root, and no UI SHALL be privileged by its location.

#### Scenario: UI owns its own configuration
- **WHEN** a rendering UI's directory is inspected
- **THEN** it contains its own dependency manifest and build configuration
- **AND** it can be built from its own directory without depending on configuration held at the repository root

#### Scenario: No UI at the repository root
- **WHEN** the repository root is inspected
- **THEN** no rendering UI source tree or UI build configuration is present there

#### Scenario: UIs are peers
- **WHEN** two rendering UIs are registered
- **THEN** neither is reachable by a path that implies precedence over the other

### Requirement: UIs do not depend on one another
A rendering UI SHALL NOT import from another rendering UI. Code required by more than one UI SHALL live in a shared package that both depend on explicitly.

#### Scenario: Cross-UI import is rejected
- **WHEN** a file in one rendering UI imports from another rendering UI's directory
- **THEN** the check fails and identifies the importing file and the imported path

#### Scenario: Shared code is depended on explicitly
- **WHEN** a UI uses shared code
- **THEN** it declares that package as a dependency and imports it by package name rather than by relative path

### Requirement: UIs may differ in structure and implementation
Rendering UIs SHALL be free to differ in their navigation, information architecture, screen inventory, component implementation and styling. A UI SHALL NOT be required to offer the same features as another, and the shared package SHALL NOT constrain a UI's framework choice.

#### Scenario: UIs differ in structure
- **WHEN** two registered UIs are compared
- **THEN** they may present different navigation, different screens and different layouts without either being considered incomplete

#### Scenario: Partial feature coverage is valid
- **WHEN** a registered UI implements a subset of the features another UI offers
- **THEN** it is still a valid registered UI and builds and runs normally

### Requirement: Workspace tooling spans the UIs and shared packages
The repository SHALL declare a package workspace covering every UI and every shared package, so dependencies resolve consistently and a shared package is consumed from source without a publish step.

#### Scenario: Shared package resolves from source
- **WHEN** a UI is built or type-checked
- **THEN** it resolves the shared package from the workspace
- **AND** a change to the shared package's source is visible to the UI without publishing or copying

#### Scenario: Checks run across the workspace
- **WHEN** the repository's lint and type checks are run
- **THEN** they cover every UI and every shared package

### Requirement: Build and desktop shell are wired through the registry
The Tauri build SHALL obtain its frontend output through the UI registry rather than from a hard-coded UI path. Building a different registered UI, or several, SHALL require only build configuration, not edits to desktop-shell configuration.

#### Scenario: Default build resolves through the registry
- **WHEN** a build runs with no UI explicitly selected
- **THEN** the registry's default UI is built and staged as the desktop shell's frontend
- **AND** no desktop-shell configuration names a UI directory directly

#### Scenario: Selecting a different UI needs no shell edit
- **WHEN** a build selects a different registered UI
- **THEN** that UI is built and staged
- **AND** no desktop-shell configuration file is modified

#### Scenario: Development server serves the selected UI
- **WHEN** the desktop shell is run in development with a UI selected
- **THEN** it loads that UI's development server

### Requirement: Existing application is preserved as a registered UI
The application that exists before this restructure SHALL be preserved as a registered UI, relocated without functional change. Its behaviour, routes and appearance SHALL be the same after relocation as before.

#### Scenario: Relocation is behaviour-preserving
- **WHEN** the existing application is relocated into its UI directory
- **THEN** its routes, features and appearance are unchanged
- **AND** its existing tests pass without weakening or deleting assertions

#### Scenario: It remains buildable and registered
- **WHEN** the relocated application is built through the registry
- **THEN** it produces a working bundle and appears as a registered UI
