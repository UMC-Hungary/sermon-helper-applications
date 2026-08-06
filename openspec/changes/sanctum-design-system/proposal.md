## Why

The archived `design-system-and-sanctum-migration` change built a design system that does not look like the design it was meant to implement. The cause is structural, not cosmetic: it defined an abstract spacing and type scale first — 4/8/12/16/20px steps — and then snapped the reference's measured values onto the nearest step. The reference's `Row` is `padding: 14px 24px` with `min-height: 56px`, a 15px title and a 12px meta line; the built component became 12px padding, 44px min-height, and smaller type. Every component drifted a little in the same way, and the accumulated drift is why the result reads as approximately-right rather than right.

A design system is worth having only if it reproduces the design faithfully. That requires the opposite discipline — measure the reference, derive the scale from what is actually there, and verify each component against its source rather than against a grid. It also requires a catalog that makes deviation visible while building, which the in-app catalog did not do well enough.

Building it inside the application also made it impossible to finish and judge independently. As its own package with Storybook, it can be completed, reviewed and signed off before any UI depends on it.

## What Changes

- **Discard the existing component library and token scale.** All 39 components and the entire token scale in `src/lib/ds/` are removed rather than corrected, because the values were derived from the wrong source. **BREAKING** for the classic UI's shell, settings and notification surfaces, which currently import them.
- **Keep only what has no visual opinion**: the four accessibility behaviour actions (`focus-trap`, `roving-tabindex`, `dismissable`, `live-region`) and the token build and validation scripts (`build-tokens.mjs`, `check-tokens.mjs`, `check-contrast.mjs`). These were not the problem — roughly 700 lines kept, 9,300 rebuilt.
- **Create `packages/design-system/`** as a standalone workspace package: tokens, components, and Storybook. It is a prerequisite, completed and reviewed before any UI consumes it.
- **Derive every token from measurement.** The token scale is extracted from the design reference at `~/workspace/ui/sermon-helper-svelte` — its actual paddings, sizes, heights, tracking and weights — rather than from a generic grid. A value that appears in the reference appears in the scale; the scale does not round it.
- **Extract by tooling, not by hand.** A script parses the reference's `<style>` blocks and `global.css`, recording every declaration with its source, selector, property, value and enclosing media query. The record is committed and re-runnable, which also makes it possible to diff a built component's values against its source mechanically — turning the drift that broke the previous attempt into a check rather than a review.
- **Reproduce the reference's responsive behaviour exactly.** Per-breakpoint values are carried where the reference overrides them at `≥760px` or `≥1360px`, and held constant where it holds them constant. No responsive step is invented for a value the reference does not vary.
- **Rebuild the components to match the reference**, one at a time, each verified against its source. The library covers the reference's own components and the vocabulary the reference lacks but the screens need — table, select, tabs, skeleton, empty and error states, progress, badge, radio group, tooltip.
- **Adopt Storybook** as the catalog, replacing the in-app `/design` route. Every component gets stories covering its variants and states, in both colour schemes, at mobile and desktop widths, with accessibility checks running in CI.
- **Keep the accessibility contract.** Components follow the WAI-ARIA Authoring Practices for their pattern, meet WCAG 2.2 AA contrast in both schemes, are fully keyboard operable with visible focus, and carry a written specification. The reference itself has one focus rule in 28 components and 17 total ARIA attributes, so its appearance is reproduced while its markup is not.
- **Ship one design with light and dark schemes.** No second theme pack, no cross-design selector.

## Capabilities

### New Capabilities
- `design-tokens`: A token architecture whose values are measured from the design reference rather than fitted to an abstract scale, covering colour, typography, spacing, sizing, radius, borders, elevation, motion, layering, breakpoints and touch targets, for light and dark schemes, with machine-checkable completeness and contrast validation.
- `design-system-components`: An accessible component library that reproduces the design reference faithfully, styled exclusively through tokens, with a written specification and an accessibility contract per component, and exactly one component per concept.
- `design-system-catalog`: A Storybook catalog covering every component across its variants, states, colour schemes and viewports, with automated accessibility and visual verification, usable as the review surface for fidelity against the reference.

### Modified Capabilities
<!-- None. The archived change's specs were not published to openspec/specs/, so these capabilities are being defined for the first time here. -->

## Impact

- **New package**: `packages/design-system/` with its own `package.json`, build, Storybook configuration, token source and generated CSS, component source, per-component specifications and stories.
- **Removed**: `src/lib/ds/` components, tokens, docs and the `/design` catalog route; the `classic` token pack and the `data-design` attribute.
- **Dependencies added**: Storybook and its accessibility addon, scoped to the design-system package so no UI carries them at runtime.
- **Classic UI**: its shell, settings and notification surfaces currently import the discarded components and must be reverted to their own styling, since classic is frozen and will not consume the new package.
- **Fonts**: the reference's three typefaces (Cormorant Garamond, Inter Tight, Geist Mono) are bundled as self-hosted woff2 subsets including latin-ext for Hungarian; the reference loads them from a font CDN, which the offline desktop app cannot.
- **CI**: token drift, theme completeness, contrast and Storybook accessibility checks run on the package.
- **Prerequisite for** `split-into-two-uis`, whose Sanctum UI consumes this package instead of relocating the discarded one.
- **Not affected**: the Rust core, HTTP/WS contracts, OpenAPI, Bruno, Companion, and the `/presenter` and `/caption` projection surfaces.
