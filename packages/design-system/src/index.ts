/**
 * The package's only public entry point. Consumers import from here; nothing reaches into
 * `src/primitives` or `src/patterns` directly, so a component's internals stay internal.
 */

// Primitives
export { default as Dot } from './primitives/Dot.svelte';
export { default as Icon } from './primitives/Icon.svelte';
export { default as List } from './primitives/List.svelte';
export { default as Row } from './primitives/Row.svelte';
export { default as SectionLabel } from './primitives/SectionLabel.svelte';
export { default as StatusDot } from './primitives/StatusDot.svelte';

// Accessibility behaviours, carried over unchanged from the previous design system.
export { default as LiveRegion } from './a11y/LiveRegion.svelte';
export { dismissable } from './a11y/dismissable.js';
export { focusTrap } from './a11y/focus-trap.js';
export { announce, assertiveMessage, politeMessage } from './a11y/live-region.js';
export { rovingTabindex } from './a11y/roving-tabindex.js';

export type { IconName } from './primitives/Icon.svelte';
export type { Status } from './primitives/StatusDot.svelte';
export type { DismissableOptions } from './a11y/dismissable.js';
export type { RovingOptions } from './a11y/roving-tabindex.js';
