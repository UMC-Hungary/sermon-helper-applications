## ADDED Requirements

### Requirement: UI registry manifest
The project SHALL maintain a registry that lists the available rendering UIs, each with a stable id, display name, and the location of its build output. The current SvelteKit app SHALL be registered as the first entry.

#### Scenario: Registry enumerates available UIs
- **WHEN** the build or settings layer queries the registry
- **THEN** it returns each available UI's id, display name, and build-output location
- **AND** the existing SvelteKit app appears as a registered UI

### Requirement: Build-time UI selection
The build SHALL select which registered UI(s) are bundled into the Tauri `frontendDist`, driven by build configuration rather than hard-coded paths.

#### Scenario: Build bundles the selected UI
- **WHEN** a build is run with a UI selected via build configuration
- **THEN** that UI's static output is produced and wired as the Tauri `frontendDist`
- **AND** building a different registered UI requires only changing the build configuration, not editing Tauri source

#### Scenario: Default selection preserves current behavior
- **WHEN** no explicit UI is selected at build time
- **THEN** the build bundles the existing SvelteKit app, matching current behavior

### Requirement: Settings selector for the active UI
When more than one UI is bundled, the settings screen SHALL let the user choose the active rendering UI among the bundled options, and the choice SHALL persist across restarts.

#### Scenario: User switches the active UI
- **WHEN** multiple UIs are bundled and the user selects a different active UI in settings
- **THEN** the selection is persisted and the chosen UI is loaded on the next app start

#### Scenario: Selector hidden when only one UI is bundled
- **WHEN** only a single UI is bundled
- **THEN** the settings UI selector is hidden or shown as read-only, and that UI is active by default
