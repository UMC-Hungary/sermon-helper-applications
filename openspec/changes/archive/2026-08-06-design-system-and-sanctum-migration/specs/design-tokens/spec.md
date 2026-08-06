## ADDED Requirements

### Requirement: Layered token architecture
The design system SHALL express all visual values as tokens organised in three layers: primitive tokens (raw values such as a colour ramp step or a spacing step), semantic tokens (role-based aliases such as surface, text, border, accent, status), and component tokens (values scoped to a single component). Components SHALL consume semantic or component tokens only, never primitive tokens and never literal values.

#### Scenario: Component consumes only role-based tokens
- **WHEN** any design-system component's styles are inspected
- **THEN** every colour, spacing, radius, border, shadow, font and duration value resolves through a semantic or component token
- **AND** no literal colour, pixel radius or shadow value appears in the component's styles

#### Scenario: Retheming requires no component edits
- **WHEN** a new theme pack supplies a complete set of semantic token values
- **THEN** every component adopts the new appearance with no change to any component file

### Requirement: Token source of truth and CSS emission
Tokens SHALL be authored in a single machine-readable source in W3C Design Tokens Community Group format, and CSS custom properties SHALL be generated from that source rather than hand-maintained. The generated CSS SHALL be committed to the repository so that no token build step is required for an ordinary application build.

#### Scenario: Generated CSS matches the token source
- **WHEN** the token generator is run against the committed token source
- **THEN** the emitted CSS is byte-identical to the committed generated CSS
- **AND** a check available in CI fails when the two diverge

#### Scenario: Application build needs no token generation
- **WHEN** the application is built without running the token generator
- **THEN** the build succeeds using the committed generated CSS

### Requirement: Token coverage
The token set SHALL cover colour (surfaces, text, borders, accent, status, and glass surface tints), typography (family, size, weight, line-height, letter-spacing, and text-transform for micro-labels), spacing, sizing, border radius, border width, elevation, motion (duration and easing), z-index layering, responsive breakpoints, and minimum touch target size.

#### Scenario: Breakpoints are tokens
- **WHEN** a component or surface adapts to viewport width
- **THEN** it does so using a breakpoint token rather than a literal width value

#### Scenario: Every visual dimension is tokenised
- **WHEN** a new component is authored against the system
- **THEN** it can express its full appearance without introducing a value outside the token categories above

### Requirement: Interactive accent is distinct from status colours
Each theme pack SHALL define an interactive accent token that is visually distinct from its live, error, warning and success status colours. The accent SHALL differ from the error and live colours by at least 60 degrees of hue, SHALL remain distinguishable from them under common colour-vision deficiencies, and SHALL meet 4.5:1 contrast as text and 3:1 as a focus indicator or interface boundary against its own surfaces, in both colour schemes. The accent SHALL govern focus indication, active navigation state, selection and links.

#### Scenario: Accent is not confusable with an error
- **WHEN** an accent-coloured interactive element and an error-coloured element are shown in the same view
- **THEN** they are distinguishable by hue, and the accent does not read as an error or a live-broadcast indication

#### Scenario: Accent separation is validated
- **WHEN** a theme pack's accent is within 60 degrees of hue of its error or live colour
- **THEN** validation fails and reports the measured separation

#### Scenario: Accent legibility in both schemes
- **WHEN** the accent is measured against the surfaces it appears on in each colour scheme
- **THEN** it meets 4.5:1 as text and 3:1 as a focus indicator or interface boundary

### Requirement: Theme packs with light and dark schemes
The system SHALL provide theme packs, each defining a complete set of semantic token values for both a light and a dark colour scheme. Two packs SHALL be provided and both SHALL remain permanently available and user-selectable: `sanctum` (the new design direction, default) and `classic` (the pre-migration appearance). A theme pack SHALL be considered valid only when it defines every semantic token in both schemes.

#### Scenario: Theme pack completeness is validated
- **WHEN** a theme pack omits a semantic token in either colour scheme
- **THEN** validation fails and identifies the missing token and scheme

#### Scenario: Both schemes available per theme
- **WHEN** either theme pack is active
- **THEN** both its light and its dark scheme render a complete interface with no unresolved custom properties

### Requirement: Theme and scheme applied via root attributes
The active theme and colour scheme SHALL be applied as attributes on the document root element (`data-design` for the theme pack, `data-theme` for the colour scheme), and all token values SHALL be selected by those attributes. Changing either attribute SHALL restyle the running application without a reload.

#### Scenario: Live theme switch
- **WHEN** the `data-design` attribute on the document root changes to another registered theme
- **THEN** the interface adopts that theme's tokens immediately without a page reload or component remount

#### Scenario: Live scheme switch
- **WHEN** the `data-theme` attribute on the document root changes between `light` and `dark`
- **THEN** the interface adopts that scheme's tokens immediately

### Requirement: Glass surfaces are theme-owned tokens
Window transparency SHALL be preserved and its surface tints SHALL be supplied by the active theme. Each theme pack SHALL define the translucent tints for the sidebar, cards and content pane, and the blur/saturation filter values, for both colour schemes. The macOS liquid-glass window effect SHALL continue to apply unchanged.

#### Scenario: Glass retinted by theme
- **WHEN** the Sanctum theme is active on macOS
- **THEN** the window remains translucent with the desktop visible through it
- **AND** the sidebar, card and content surfaces are tinted with Sanctum's colours rather than the previous neutral greys

#### Scenario: Non-glass platform fallback preserved
- **WHEN** the platform does not support the glass window effect and the root carries `data-glass='false'`
- **THEN** the active theme supplies near-opaque surface values and disables backdrop filters
- **AND** the interface remains fully legible in both colour schemes

#### Scenario: Reduced transparency honoured
- **WHEN** the operating system reports `prefers-reduced-transparency: reduce`
- **THEN** the active theme supplies fully opaque surface tokens and no backdrop filter is applied

### Requirement: Bundled typefaces
The typefaces required by a theme pack SHALL be self-hosted and bundled with the application. The interface SHALL NOT depend on a network font request at runtime, and SHALL declare a platform fallback stack for each font role.

#### Scenario: Typography renders offline
- **WHEN** the application starts with no network connectivity
- **THEN** all text renders in the theme's intended typefaces

#### Scenario: No external font requests
- **WHEN** application network activity is inspected during startup
- **THEN** no request is made to an external font host

### Requirement: Motion tokens honour reduced motion
All animation and transition durations SHALL be expressed as motion tokens, and when the operating system reports `prefers-reduced-motion: reduce` those tokens SHALL resolve to values that eliminate non-essential motion.

#### Scenario: Reduced motion suppresses transitions
- **WHEN** the operating system reports `prefers-reduced-motion: reduce`
- **THEN** components complete state changes without animated transitions
- **AND** no component becomes unusable or loses a state indication as a result
