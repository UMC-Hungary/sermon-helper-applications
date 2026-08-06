## Context

The archived `design-system-and-sanctum-migration` change delivered 9,993 lines of design system that does not look like the design it implements. The failure is diagnosable and specific. Comparing the built `Row` against the reference's:

| | Reference | Built | |
| --- | --- | --- | --- |
| padding | `14px 24px` | `var(--space-4)` → `12px` | lost the 24px gutter entirely |
| min-height | `56px` | `var(--target-min)` → `44px` | 21% shorter |
| title | `15px` | `var(--font-size-sm)` | smaller |
| meta | `12px` | `var(--font-size-xs)` | smaller |
| leading gap | `14px` | `var(--space-4)` → `12px` | tighter |

The token scale was authored first as a conventional progression — `space-1..12` on a 4px grid, a `t-shirt` type scale — and each component was then fitted to the nearest available step. No single substitution is dramatic; the accumulation is. The reference's rhythm is built on 14px, 24px, 56px, 34px and 10px-at-2px-tracking, and a 4px grid cannot express most of those. The result is a system that is internally consistent and externally wrong.

What is being kept is what had no visual opinion and worked: the four accessibility actions (`focus-trap`, `roving-tabindex`, `dismissable`, `live-region`) and the token generator and validators (`build-tokens.mjs`, `check-tokens.mjs`, `check-contrast.mjs`) — roughly 700 lines. Everything visual is rebuilt.

The reference is a Svelte 4 prototype at `~/workspace/ui/sermon-helper-svelte`: 28 components across 6 screens, mobile-first at 402px with reflows at `≥760px` and `≥1360px`, warm parchment palette, Cormorant Garamond display, Inter Tight body, Geist Mono micro-labels, hairline rules, square corners, no shadows. It is also not accessible — one focus rule across 28 components, 17 total ARIA attributes, its only dialog has no focus trap, and tab-like controls are hand-rolled four separate times. Its appearance is the target; its markup is not.

## Goals / Non-Goals

**Goals:**
- A component library that reproduces the reference faithfully enough that a reviewer comparing the two sees no difference worth remarking on.
- A token scale derived from measurement, where every reference value survives intact.
- Storybook as the working surface, so drift is visible while building rather than after integrating.
- The accessibility the reference lacks, without altering the appearance it has.
- A package that can be finished, reviewed and signed off before any UI depends on it.

**Non-Goals:**
- Correcting the existing components. Their values came from the wrong source; correcting them one at a time would preserve the structural assumptions that caused the drift.
- Serving the classic UI. It is frozen and reverts to its own styling.
- A second theme pack or a cross-design selector. One design, two colour schemes.
- Implementing screens. This package delivers the vocabulary; `split-into-two-uis` builds the screens.
- Reproducing the reference's markup, its phone chrome, or its prototype-only tweaks panel.

## Decisions

### Measure first, then derive the scale

This inverts what was done before, and it is the single decision that matters most.

The first task is not authoring tokens — it is extracting every numeric value the reference uses into a table: which component, which property, what value. Only then is the scale derived, as the set of values actually observed. If the reference uses 14px and 24px and never 16px or 20px, the scale has 14 and 24 and not 16 or 20. Named steps come from usage, not arithmetic.

The rule the spec encodes: **a value that appears in the reference appears in the scale, unrounded.** A one-off value is still a token; it is not approximated by a neighbour. Each token records the measurement it came from, so it can be re-verified and so a reviewer can see the provenance.

Contrast is the one sanctioned exception. The reference's `--ink-faint #b8b2a4` on `--card #f6f2e8` is roughly 1.9:1 and is used for text; that fails AA and gets darkened. Such a deviation must be recorded with its reason, which keeps "we changed it because it was inaccessible" distinct from "we changed it because our grid said so".

*Alternative considered:* keep a conventional scale and accept approximation, on the theory that internal consistency matters more than fidelity. That is exactly what was tried, and it produced a system the author of the design rejected on sight.

### A standalone package, so it can be finished before it is used

`packages/design-system/`, a workspace peer of `packages/core-client/`:

```
packages/design-system/
  package.json
  .storybook/
  tokens/
    measurements.md        the extraction table — provenance for every value
    tokens.json            authored source
    themes/sanctum.json    light + dark
    generated/tokens.css   committed
  src/
    primitives/  patterns/  a11y/   (a11y carried over)
    index.ts                        public entry point
  docs/          one specification per component
  stories/       one story file per component
  static/fonts/  self-hosted woff2 subsets
scripts/         build-tokens, check-tokens, check-contrast (carried over)
```

Building it inside the app was what made it impossible to judge: it was only ever seen through half-migrated screens. As a package with Storybook it is reviewable on its own terms, against the reference, before a single screen consumes it. That is what makes "prerequisite" meaningful rather than nominal.

### Storybook replaces the in-app catalog

I argued against Storybook when this was an in-app design system, on the grounds that it renders in a browser iframe rather than the real glass window. That reasoning does not survive the package split: the design system no longer has an application to live inside, and the qualities that matter now — every component visible in isolation, variants and states enumerated, schemes and viewports switchable, accessibility checks automated per story — are exactly Storybook's.

The addon set stays small: accessibility checks (failing CI on violations), viewport switching for the mobile and desktop breakpoints, and a scheme toggle. Storybook is a development dependency of the package alone, so no consuming UI carries it.

The catalog also carries the fidelity comparison: each story presents the component alongside its recorded reference measurements. Drift becomes visible where the work happens, rather than after a screen is assembled.

### Reproduce appearance, rebuild markup

Every component is measured from its reference counterpart and rebuilt against the WAI-ARIA contracts. Concretely: the reference's four hand-rolled tab treatments collapse into one accessible Tabs; its borderless `.select` becomes a real listbox with arrow-key handling and focus return; its only dialog gains a focus trap and focus restoration; every interactive component gains a focus indicator, of which the reference has essentially none.

Where the reference has no counterpart — table, skeleton, empty and error states, progress, badge, tooltip, radio group — the component is designed in the reference's idiom: hairline rules, square corners, mono micro-labels, serif reserved for display type. These are the components the consuming screens need and the prototype never had to build.

### Classic reverts rather than adapts

The classic UI's shell, settings and notification surfaces currently import the discarded components. Since classic is frozen and will not consume the new package, those surfaces revert to their own styling. This is unglamorous work with no user-visible benefit, and it must happen or classic stops building.

## Risks / Trade-offs

- **Fidelity is judged by eye, and eyes disagree.** The measurement table and the catalog's side-by-side comparison make it as objective as it can be, but final acceptance is still a review. → Review each component against the reference as it is built, not in a batch at the end, so a systematic misreading is caught on component three rather than component thirty.
- **A measured scale is less tidy than a generated one.** It will have irregular steps and one-off values, and will look wrong to anyone expecting a 4px grid. → That is the point, and the spec says so explicitly, so it is not "corrected" later by someone tidying up.
- **9,300 lines are being discarded, including work that was correct.** → Accepted. The components were derived from the wrong values; auditing each to find the sound ones costs more than rebuilding from a correct scale.
- **Storybook is a substantial dependency** with its own upgrade burden. → Confined to the package as a development dependency; no runtime cost to any UI.
- **The reference covers 28 components; the screens need more.** The invented ones have no source to be faithful to and are where the design's coherence is most at risk. → Build them only when a screen needs one, in the reference's idiom, reviewed against neighbouring reference components.
- **Reverting classic is unrewarding work that blocks nothing user-visible** and will be tempting to skip. → It is a task in this change, not the next one, because classic will not build otherwise.

## Migration Plan

1. **Extract measurements** from all 28 reference components into `tokens/measurements.md` — component, property, value. No authoring yet.
2. **Derive the scale** from that table; author tokens; carry over the generator and validators; commit the generated CSS.
3. **Scaffold the package** with Storybook, the carried-over accessibility actions, and the bundled fonts.
4. **Build components in reference order** — Row, List, SectionLabel, PageHeader, Toggle, Field, Segmented, Stat, OverviewCell, DateBlock, Dot, Glyph, then the form family, then overlays — each with its specification, stories and reference comparison, reviewed before the next begins.
5. **Build the components the reference lacks**, in its idiom.
6. **Revert the classic UI** off the discarded components.
7. **Sign-off**: every component reviewed against the reference, contrast and completeness checks green, Storybook accessibility checks green in CI.

**Rollback:** the package is not consumed by anything until `split-into-two-uis` consumes it, so incomplete work ships nothing. The only coupling to the running product is step 6, which restores classic to how it was before this work began.

## Resolved Questions

### Responsive behaviour is whatever the reference does, per breakpoint

No responsive steps are invented. Where the reference overrides a value inside a media query, that override is reproduced; where a component holds the same value at every width, it holds the same value at every width. A component that does not resize is not a gap to be filled — it is the design.

This makes the extraction breakpoint-aware rather than flat. Every measurement is recorded with the media-query context it appears in, so the table distinguishes three cases: a value that holds everywhere, a value overridden at `≥760px`, and a value overridden again at `≥1360px`. The reference reflows *layout* at those thresholds — `display: contents` collapsing to two-column grids, gutters changing, sticky asides appearing — while most component-level sizing stays constant. Recording the context is what proves that rather than assuming it.

Two consequences worth stating: a token may carry per-breakpoint values, and the extraction output is the authority on which ones do. Nothing is promoted to a responsive token because it "looks like it should be".

### The measurement table is generated by tooling

A script parses the reference's stylesheets rather than a human reading them. The reference is 28 components plus 6 screens, each with its own `<style>` block, plus `global.css` — by hand that is a long afternoon and a guaranteed transcription error or two, and it produces a table nobody can re-run.

The extractor walks every `.svelte` file's `<style>` block and `global.css`, and emits one row per declaration: source file, selector, property, value, and enclosing media query. Its output is `tokens/measurements.md` (human-readable, committed, reviewable) plus a machine-readable form the scale derivation and the later fidelity checks consume.

Being re-runnable is the point that matters most. It means the measurement table can be regenerated if the reference changes, and — more usefully — that a built component's values can be diffed against its recorded source mechanically. The failure that produced the previous attempt was a value drifting from 14px to 12px with nobody noticing; that becomes a check rather than a review.

What tooling cannot do is read intent. It cannot tell that a `2px` letter-spacing on a 10px uppercase label is a deliberate micro-label idiom while a `0.5px` elsewhere is incidental, or which of two similar paddings is structural. So the pipeline is: extract mechanically, then group and name by hand. The tool produces the facts; the naming of semantic roles is a human judgement made against those facts rather than from memory.

## Open Questions

- None outstanding.
