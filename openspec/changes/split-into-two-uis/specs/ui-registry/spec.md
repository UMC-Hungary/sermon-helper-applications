## MODIFIED Requirements

### Requirement: UI registry manifest
The project SHALL maintain a registry that lists the available rendering UIs, each with a stable id, display name, and the location of its build output. Every rendering UI SHALL be registered, and each SHALL resolve to a self-contained application directory under `ui/` rather than to the repository root.

#### Scenario: Registry enumerates available UIs
- **WHEN** the build or settings layer queries the registry
- **THEN** it returns each available UI's id, display name, and build-output location
- **AND** every registered UI resolves to its own application directory

#### Scenario: Both UIs are registered
- **WHEN** the registry is inspected
- **THEN** the relocated existing application and the Sanctum UI each appear as a registered entry with a distinct id

### Requirement: Build-time UI selection
The build SHALL select which registered UI(s) are bundled into the Tauri `frontendDist`, driven by build configuration rather than hard-coded paths. Selection SHALL identify a UI by its registry id.

#### Scenario: Build bundles the selected UI
- **WHEN** a build is run with a UI selected via build configuration
- **THEN** that UI's static output is produced and wired as the Tauri `frontendDist`
- **AND** building a different registered UI requires only changing the build configuration, not editing Tauri source

#### Scenario: Default selection is the registry's default id
- **WHEN** no explicit UI is selected at build time
- **THEN** the build bundles the UI named by the registry's default id
- **AND** the resulting bundle behaves as it did before the UIs were relocated

#### Scenario: Both UIs bundled together
- **WHEN** a build selects both registered UIs
- **THEN** each is built and staged under its own path
- **AND** the bundle opens the UI the user last chose, falling back to the default

### Requirement: Settings selector for the active UI
When more than one UI is bundled, the settings screen SHALL let the user choose the active rendering UI among the bundled options, and the choice SHALL persist across restarts. Each UI that offers the selector SHALL implement it in its own way; the selector SHALL NOT require a shared component.

#### Scenario: User switches the active UI
- **WHEN** multiple UIs are bundled and the user selects a different active UI in settings
- **THEN** the selection is persisted and the chosen UI is loaded on the next app start

#### Scenario: Selector hidden when only one UI is bundled
- **WHEN** only a single UI is bundled
- **THEN** the settings UI selector is hidden or shown as read-only, and that UI is active by default

#### Scenario: Selection survives switching between UIs
- **WHEN** the user switches to another UI and then switches back
- **THEN** each UI reads and writes the same persisted selection, so the choice is honoured regardless of which UI made it
