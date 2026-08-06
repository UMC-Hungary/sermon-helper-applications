## ADDED Requirements

### Requirement: Appearance settings section
The settings screen SHALL provide an Appearance section containing a design-theme selector and a colour-scheme selector. Both controls SHALL be labelled, keyboard operable, and translated in every supported locale.

#### Scenario: Appearance section present
- **WHEN** the user opens the settings screen
- **THEN** an Appearance section is shown containing a design-theme selector and a colour-scheme selector

#### Scenario: Controls are keyboard operable and labelled
- **WHEN** the user reaches the Appearance controls by keyboard
- **THEN** each control exposes an accessible name, its current value, and can be changed without a pointer

### Requirement: Design theme selection
The design-theme selector SHALL list every registered theme pack with a display name and a short description, and SHALL indicate which is active. Selecting a theme SHALL apply it immediately.

#### Scenario: Theme applied immediately
- **WHEN** the user selects a different design theme
- **THEN** the entire interface adopts that theme without a restart, reload or navigation
- **AND** the selector shows the newly selected theme as active

#### Scenario: Both packs offered
- **WHEN** the design-theme selector is shown
- **THEN** both the Sanctum and the classic theme packs are offered as permanent options

#### Scenario: Selector hidden when only one theme exists
- **WHEN** only one theme pack is registered
- **THEN** the design-theme selector is hidden and that theme is active by default

### Requirement: Appearance settings do not affect projection output
Changing the design theme or the colour scheme SHALL have no effect on the presenter output view or the caption overlay.

#### Scenario: Output unaffected by appearance changes
- **WHEN** the user changes the design theme or the colour scheme while a presentation or caption output is being displayed
- **THEN** the output surface's appearance does not change

### Requirement: Light, dark and auto colour scheme
The colour-scheme selector SHALL offer exactly three choices — Light, Dark and Auto — and SHALL apply the choice immediately. Under Auto, the colour scheme SHALL follow the operating-system appearance and SHALL update live when the operating system changes it. Under Light or Dark, the explicit choice SHALL override the operating-system appearance.

#### Scenario: Explicit light selection
- **WHEN** the user selects Light while the operating system is set to dark
- **THEN** the interface renders in the active theme's light scheme
- **AND** it remains light when the operating system appearance changes

#### Scenario: Explicit dark selection
- **WHEN** the user selects Dark while the operating system is set to light
- **THEN** the interface renders in the active theme's dark scheme
- **AND** it remains dark when the operating system appearance changes

#### Scenario: Auto follows the system live
- **WHEN** the user selects Auto and the operating-system appearance subsequently changes
- **THEN** the interface switches to the matching colour scheme without a restart

#### Scenario: Auto outside the desktop host
- **WHEN** Auto is active in a context where the desktop host appearance is unavailable, such as a browser
- **THEN** the colour scheme follows the `prefers-color-scheme` media query

### Requirement: Persistence of appearance choices
The selected design theme and colour scheme SHALL persist across application restarts. They SHALL be written both to browser-local storage, for a flash-free first paint, and to the desktop host's settings store, mirroring the pattern already used for the interface locale.

#### Scenario: Choices survive restart
- **WHEN** the user selects a theme and a colour scheme and restarts the application
- **THEN** the same theme and colour scheme are active on next launch

#### Scenario: Host store unavailable
- **WHEN** the desktop host settings store cannot be reached
- **THEN** the choice is still applied and stored locally, and no error is surfaced to the user

### Requirement: No unstyled or mis-schemed first paint
The stored appearance SHALL be applied to the document root before the first paint of application content, so that the interface never renders in the wrong colour scheme or without tokens and then corrects itself.

#### Scenario: No flash on launch
- **WHEN** the application launches with Dark previously selected
- **THEN** the first painted frame is already dark
- **AND** no light-scheme or unstyled frame is shown beforehand

### Requirement: Appearance selection is independent of active-UI selection
The Appearance settings SHALL be separate from the existing active-UI selector. Changing the design theme or colour scheme SHALL NOT change which rendering UI is bundled or active, and SHALL NOT require a restart.

#### Scenario: Selectors remain distinct
- **WHEN** the user changes the design theme
- **THEN** the active rendering UI is unchanged and no restart is requested
- **AND** the active-UI selector continues to reflect its own independent selection
