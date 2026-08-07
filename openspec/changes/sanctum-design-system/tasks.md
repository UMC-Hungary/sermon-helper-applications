## 1. Measure the reference

No tokens are authored in this group. The output is a generated, re-runnable record of what the reference actually uses.

- [x] 1.1 Write `scripts/extract-reference.mjs` — parse every `.svelte` `<style>` block across the reference's 28 components and 6 screens, plus `global.css`, emitting one row per declaration: source file, selector, property, value, and **enclosing media query**
- [x] 1.2 Emit `tokens/measurements.md` (committed, human-readable, grouped by component) and a machine-readable form for the scale derivation and later fidelity checks
- [x] 1.3 Capture the reference's palette from `global.css` for both `.theme-light` and `.theme-dark`
- [x] 1.4 Classify every measurement by breakpoint context — holds at all widths, overridden at `≥760px`, overridden again at `≥1360px` — so per-breakpoint tokens are identified from evidence rather than assumption
- [x] 1.5 Verify the extractor against a hand-checked sample (at minimum `Row`, `SectionLabel`, `PageHeader`) to confirm it is not dropping or misattributing declarations
- [x] 1.6 Produce the distinct value set per category — spacing, sizing, type, tracking, weight, radius, border, duration; this set, not a grid, is the scale
- [x] 1.7 By hand, group the extracted values into semantic roles and name them; tooling supplies the facts, naming is a judgement made against them
- [x] 1.8 Confirm the extractor re-runs cleanly and reproduces identical output, so it can be used later to diff built components against their recorded sources

## 2. Tokens

- [x] 2.1 Author `tokens/tokens.json` from the measured set, each token recording its source measurement; no value rounded, no step added to complete a progression
- [x] 2.2 Author `themes/sanctum.json` light scheme from the measured light palette
- [x] 2.3 Author the dark scheme from the measured dark palette, independently verified
- [x] 2.4 Carry over `build-tokens.mjs`, `check-tokens.mjs`, `check-contrast.mjs`; emit `:root` + scheme blocks only — no `data-design` attribute, no second theme pack
- [x] 2.5 Run the contrast check over the measured palette and record every failing pairing before correcting anything
- [x] 2.6 Correct only the failing values, recording each deviation with its reason (`--ink-faint` on `--card` measures ~1.9:1 and carries text in the reference)
- [x] 2.7 Define the interactive accent as a distinct hue, ≥60° from the live and error colours, and add the hue-separation validator
- [x] 2.8 Add breakpoint and 44px touch-target tokens
- [x] 2.9 Commit the generated CSS; wire drift, completeness, hue-separation and contrast checks into CI

## 3. Package scaffold

- [x] 3.1 Create `packages/design-system/` with its own `package.json` and a single public entry point
- [x] 3.2 Carry over the four accessibility actions — `focus-trap`, `roving-tabindex`, `dismissable`, `live-region` — and their contract docs unchanged
- [x] 3.3 Add self-hosted woff2 subsets for Cormorant Garamond (400/500/600 + 400 italic), Inter Tight (400/500/600/700), Geist Mono (400/500), latin + latin-ext
- [x] 3.4 Declare `@font-face` with per-role fallback stacks; verify Hungarian glyphs (`ő`, `ű`, `Ő`, `Ű`) and zero network font requests
- [x] 3.5 Set up Storybook against the package source, with the accessibility addon, viewport switching at the measured breakpoints, and a colour-scheme toggle
- [x] 3.6 Agree the per-component specification template — anatomy, props, variants, states, tokens consumed, keyboard map, ARIA, acceptance criteria, reference correspondence, recorded deviations
- [x] 3.7 Add the check that fails on any export lacking a specification or a story
- [x] 3.8 Confirm Storybook is a development dependency only and cannot reach a consuming application's production output

## 4. Components with a reference counterpart

Build in this order; each is measured against its source, specified, given stories, and reviewed before the next begins.

- [x] 4.1 Row — `padding: 14px 24px`, `min-height: 56px`, 15px title, 12px meta, 14px leading gap; clickable and danger variants; `aria-current` for navigational rows
- [x] 4.2 List, SectionLabel (10px mono, 2px tracking, uppercase, optional hint), PageHeader (eyebrow, serif display, back, trailing slots)
- [x] 4.3 Dot, StatusDot, TextIcon, DateBlock, Glyph (brand marks), Lockup, FlameMark, Icon set
- [x] 4.4 Toggle, Field, Segmented (glyph + label + hint per option), Stat, OverviewCell
- [x] 4.5 Form family — numbered form section, labelled borderless input, native date/time input, toggle row, scripture reference input; add a visible resting boundary and a compliant focus indicator, recorded as a deviation
- [x] 4.6 Overlays — Dialog, Sheet, Toast, ToastOverlay, notification centre, notification bell — composed from the carried-over accessibility actions
- [x] 4.7 StickyActionBar; navigation bar (the reference's tab bar), honouring safe-area insets
- [x] 4.8 Presentation controls — transport dock, slide queue slots, slide search, presenter support panel
- [x] 4.9 Discovery panel and device list
- [x] 4.10 Do not build the reference's phone frame, simulated status bar, or tweaks panel

## 5. Components the reference lacks

Designed in the reference's idiom — hairline rules, square corners, mono micro-labels, serif for display only — and reviewed against neighbouring reference components.

- [x] 5.1 Tabs — one accessible component replacing the reference's four hand-rolled treatments
- [x] 5.2 Select — a real listbox replacing the reference's roleless button-and-div
- [x] 5.3 Table with narrow-viewport reflow
- [x] 5.4 Button, IconButton, Checkbox, RadioGroup
- [x] 5.5 TextField, TextArea, FormField wrapper — label association, `aria-describedby` error text, `aria-invalid`
- [x] 5.6 Skeleton, EmptyState, ErrorState with retry
- [x] 5.7 Badge, Spinner, ProgressBar, Tooltip
- [x] 5.8 Audit the finished inventory for duplicated concepts and collapse each to one component with variants

## 6. Verification

- [x] 6.1 Every component reviewed side by side against its reference counterpart; discrepancies corrected or recorded as deliberate deviations with reasons
- [x] 6.1b Run the mechanical fidelity diff — built component values against their extracted source measurements — and confirm every difference is a recorded deviation, not drift
- [x] 6.2 Every component keyboard operable with visible focus meeting 3:1, accessible names present, no colour-only meaning
- [x] 6.3 Every component verified at a 360px viewport and at desktop width, with 44px touch targets
- [x] 6.4 Contrast, completeness, hue-separation and token-drift checks green
- [x] 6.5 Storybook accessibility checks green across every story, running in CI
- [x] 6.6 Every export has a specification, stories covering its variants and states, and its recorded reference measurements

## 7. Disentangle the classic UI

- [x] 7.1 Revert the classic UI's shell, settings and notification surfaces off the discarded components onto their own styling — **nothing to revert**: the archived change was never merged, so `main` has no `src/lib/ds/` and no surface imports it
- [x] 7.2 Delete `src/lib/ds/` and the in-app `/design` catalog route — **already absent on `main`**; the discarded work lives only on the unmerged `design-system-sanctum` branch
- [x] 7.3 Confirm the classic UI builds, `pnpm check` and `pnpm lint` are clean, and its e2e suite passes
- [x] 7.4 Confirm no consuming application yet depends on the design-system package — it ships nothing until `split-into-two-uis` consumes it
