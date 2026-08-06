# Row

The single line of a list: a title, an optional second line, an optional trailing value, and
whatever leading mark or trailing control the screen needs. It is the design's most-used
component and the one the previous attempt got most visibly wrong.

## Anatomy

```
┌──────────────────────────────────────────────────────────┐
│ [icon]  Title                        detail  [control] › │
│ 14px    Meta line                                        │
└──────────────────────────────────────────────────────────┘
  ↑ 24px gutter, 14px block padding, 56px minimum height
```

- **icon** — optional leading slot: a `Dot`, `TextIcon`, `Glyph` or `DateBlock`.
- **body** — title and meta, the only part that flexes.
- **detail** — a trailing value that never wraps.
- **control** — optional trailing slot: a `Toggle`, `Badge` or button.
- **chevron** — shown on interactive rows only.

## Props

| Prop | Type | Default | Meaning |
| --- | --- | --- | --- |
| `title` | `string` | `''` | The first line. Ignored when `children` is given. |
| `meta` | `string` | `''` | The second line. Truncates with an ellipsis. |
| `detail` | `string` | `''` | Trailing value, never wraps. |
| `chevron` | `boolean` | `true` | Shows the affordance on interactive rows. Has no effect on a static row. |
| `danger` | `boolean` | `false` | Title in the danger colour. |
| `last` | `boolean` | `false` | Drops the bottom rule. |
| `onclick` | `(e) => void` | — | Makes the row a `<button>`. |
| `href` | `string` | — | Makes the row an `<a>`. Takes precedence over `onclick`. |
| `current` | `'page' \| 'step' \| 'true' \| false` | `false` | Sets `aria-current`. |
| `disabled` | `boolean` | `false` | Button rows only. |
| `icon`, `control`, `children` | `Snippet` | — | Leading slot, trailing slot, title content. |

## Variants

| Variant | When to use |
| --- | --- |
| Static | The row shows something; nothing happens when it is pressed. |
| Button (`onclick`) | The row performs an action in place. |
| Link (`href`) | The row navigates. Prefer this over `onclick` for navigation, so the row is a link. |
| Danger | A destructive action. Always paired with wording that says so. |
| Last | Final row of a list, where the list's own bottom rule already closes it. |

## States

Default, hover, focus-visible, active, disabled, current. Each has a story.

## Tokens consumed

| Token | Used for |
| --- | --- |
| `--ui-gutter` | Horizontal padding |
| `--c-row-padding-block` | Vertical padding |
| `--c-row-min-height` | Minimum height |
| `--ui-stack-loose` | Gap between the leading slot and the body |
| `--ui-border-hairline`, `--border-hairline` | Bottom rule |
| `--motion-fast` | Background transition |
| `--type-body-*` | Title |
| `--type-caption-size`, `--c-row-meta-gap` | Meta line |
| `--type-body-sm-size`, `--c-row-detail-gap` | Detail |
| `--surface-hover` | Hover tint |
| `--text-primary`, `--text-muted`, `--text-faint` | Title, meta and detail, chevron |
| `--status-error` | Danger title |
| `--accent`, `--ui-focus-width`, `--ui-focus-offset` | Focus indicator |

## Keyboard

| Key | Behaviour |
| --- | --- |
| `Tab` | Reaches an interactive row; a static row is not a tab stop. |
| `Enter` | Activates a link or button row. |
| `Space` | Activates a button row. |

## ARIA

An interactive row is a real `<a>` or `<button>`, so its role, name and disabled state come from
the element rather than from attributes. `aria-current` marks the row for the page or step the
user is on. A static row carries no role: it is content, not a control.

The leading and trailing slots are the caller's responsibility to name — an icon-only control
placed in `control` must carry its own accessible name.

## Accessibility acceptance criteria

- Reachable, operable and leavable by keyboard alone, with a focus indicator at 3:1 in both schemes.
- A row's accessible name is its title; a meta line that carries meaning belongs in the name too.
- Danger is never signalled by colour alone.
- At 360px the title wraps and the meta truncates; nothing overlaps and the row grows rather than clipping.
- The row's own 56px minimum height exceeds the 44px touch target; trailing controls must meet it themselves.

## Reference correspondence

Source: `src/components/primitives/Row.svelte`.

| Property | Reference | Implemented |
| --- | --- | --- |
| `padding` | `14px 24px` | `var(--c-row-padding-block) var(--ui-gutter)` → `14px 24px` |
| `min-height` | `56px` | `var(--c-row-min-height)` → `56px` |
| `gap` | `14px` | `var(--ui-stack-loose)` → `14px` |
| `border-bottom` | `1px solid var(--hairline)` | `var(--ui-border-hairline) solid var(--border-hairline)` |
| `transition` | `background 120ms` | `background var(--motion-fast)` → `120ms` |
| title `font-size` | `15px` | `var(--type-body-size)` → `15px` |
| title `letter-spacing` | `-0.1px` | `var(--type-body-track)` → `-0.1px` |
| title `font-weight` | `400` | `var(--type-body-weight)` → `400` |
| title `line-height` | `1.24` | `var(--type-body-leading)` → `1.24` |
| meta `font-size` | `12px` | `var(--type-caption-size)` → `12px` |
| meta `margin-top` | `4px` | `var(--c-row-meta-gap)` → `4px` |
| detail `font-size` | `14px` | `var(--type-body-sm-size)` → `14px` |
| detail `margin-top` | `1px` | `var(--c-row-detail-gap)` → `1px` |

Every line above is re-checked mechanically by `scripts/check-fidelity.mjs`; this table is the
readable form of `tokens/fidelity.json`, not a second source of truth.

## Recorded deviations

| What | Why |
| --- | --- |
| A focus indicator was added. | The reference has none, on this component or nearly any other. Without it the row cannot be operated by keyboard at all. It is drawn inset so it does not overlap the row above. |
| `href` produces a link rather than a button. | The reference makes every clickable row a `<button>`, including the ones that navigate. A navigating row is a link, and shipping it as a button loses the middle-click, the context menu and the announced role. |
| The chevron is hidden on static rows. | The reference defaults `chevron` to `true` and renders it on non-clickable rows, where it promises an affordance that is not there. |
| Danger colour comes from `--status-error`, not `--live`. | The two are the same measured value; the alias exists so a future divergence does not silently change what "danger" means. |
