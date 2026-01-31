# Plan: Settings Page Sub-Navigation

> **Status: COMPLETED**

## Problem Statement

The settings page (`/obs-config`) currently displays 7 different settings sections in a 2-column grid layout. As features grow, this becomes overwhelming and difficult to navigate. A sub-navigation system would improve organization and user experience.

## Current State

**Settings Sections (7 total):**
1. OBS Connection Settings - WebSocket connection config
2. Mobile Companion Discovery - Server/authentication config
3. RF/IR Remote Control - Device discovery and commands
4. PPT Folder Configuration - PowerPoint folder management
5. OBS Device Configs - Display/audio/browser source setup
6. Import/Export Settings - Backup/restore
7. Update Settings - App version and updates

**Current Layout:** All sections displayed simultaneously in a 2-column grid.

## Proposed Solution

### Option A: Horizontal Tab Navigation (Recommended)

Add a horizontal tab bar at the top of the settings page that groups related settings into logical categories.

**Proposed Categories:**
| Tab | Sections Included | Rationale |
|-----|-------------------|-----------|
| **Connection** | OBS Connection, Mobile Companion Discovery | Network/connection related |
| **Devices** | RF/IR Remote Control, OBS Device Configs | Hardware/device configuration |
| **Content** | PPT Folder Configuration | Content/file management |
| **System** | Import/Export, Update Settings | App-level settings |

**Pros:**
- Familiar UI pattern (tabs)
- No routing changes needed (client-side state)
- Single page load, fast switching
- Easy to implement with existing UI components

**Cons:**
- All components still loaded (though hidden)
- URL doesn't reflect current tab (no deep linking)

### Option B: Nested Route Navigation

Use SvelteKit nested routing with a vertical sub-navigation sidebar.

**Route Structure:**
```
/obs-config/
├── +layout.svelte       (sub-nav + content area)
├── +page.svelte         (redirect to /obs-config/connection)
├── connection/+page.svelte
├── devices/+page.svelte
├── content/+page.svelte
└── system/+page.svelte
```

**Pros:**
- Deep linking support (shareable URLs)
- Only active section components loaded
- Better separation of concerns
- Browser back/forward works naturally

**Cons:**
- More files to manage
- Route transitions add slight delay
- Need to update main sidebar active state logic

### Option C: Vertical Pills/List Navigation

Similar to Option A but with a vertical navigation list on the left side of the settings area.

**Pros:**
- More space for longer category names
- Can show more categories without scrolling
- Natural for settings pages (common pattern)

**Cons:**
- Takes horizontal space from content
- Similar tradeoffs to Option A

---

## Recommended Approach: Option B (Nested Routes)

Nested routing provides the best UX with deep linking, proper navigation history, and code splitting. The implementation complexity is manageable.

### Implementation Plan

#### Phase 1: Create Route Structure

1. **Create settings layout with sub-navigation**
   - New file: `src/routes/obs-config/+layout.svelte`
   - Contains horizontal tab bar component
   - Renders child routes via `<slot />`

2. **Create sub-route pages**
   - `src/routes/obs-config/connection/+page.svelte` → OBS Connection + Discovery
   - `src/routes/obs-config/devices/+page.svelte` → RF/IR + OBS Device Configs
   - `src/routes/obs-config/content/+page.svelte` → PPT Folder Settings
   - `src/routes/obs-config/system/+page.svelte` → Import/Export + Update

3. **Update root settings page**
   - `src/routes/obs-config/+page.svelte` → Redirect to `/obs-config/connection`

#### Phase 2: Create Sub-Navigation Component

1. **Create `settings-nav.svelte` component**
   - Horizontal tab bar matching app design
   - Uses existing UI primitives (Button, etc.)
   - Active state based on current route
   - Responsive (horizontal scroll or collapse on mobile)

2. **Add i18n keys for tab labels**
   - `settings.tabs.connection`
   - `settings.tabs.devices`
   - `settings.tabs.content`
   - `settings.tabs.system`

#### Phase 3: Update Main Sidebar

1. **Fix active state detection**
   - Update `sidebar.svelte` to detect `/obs-config/*` routes
   - Change from exact match to `startsWith()` for settings

#### Phase 4: Polish & Testing

1. **Ensure responsive behavior**
   - Tab bar scrolls horizontally on mobile
   - Content layout adapts appropriately

2. **Test navigation flows**
   - Direct URL access works
   - Browser back/forward works
   - Main sidebar highlights correctly

---

## Files to Create/Modify

### New Files
| File | Purpose |
|------|---------|
| `src/routes/obs-config/+layout.svelte` | Settings layout with sub-nav |
| `src/routes/obs-config/connection/+page.svelte` | Connection settings tab |
| `src/routes/obs-config/devices/+page.svelte` | Devices settings tab |
| `src/routes/obs-config/content/+page.svelte` | Content settings tab |
| `src/routes/obs-config/system/+page.svelte` | System settings tab |
| `src/lib/components/settings-nav.svelte` | Tab navigation component |

### Modified Files
| File | Changes |
|------|---------|
| `src/routes/obs-config/+page.svelte` | Convert to redirect |
| `src/lib/components/sidebar.svelte` | Update active state logic |
| `src/lib/i18n/en.json` | Add tab label keys |
| `src/lib/i18n/hu.json` | Add tab label keys |

---

## Visual Mockup (ASCII)

```
┌─────────────────────────────────────────────────────────────┐
│  Sidebar  │                    Settings                     │
│           │  ┌───────────┬──────────┬─────────┬──────────┐  │
│  Events   │  │Connection │ Devices  │ Content │  System  │  │
│  Remote   │  └───────────┴──────────┴─────────┴──────────┘  │
│  Caption  │  ━━━━━━━━━━━━                                   │
│  Settings │                                                 │
│           │  ┌─────────────────────────────────────────────┐│
│           │  │                                             ││
│           │  │     (Active tab content here)               ││
│           │  │                                             ││
│           │  │     - OBS Connection Settings               ││
│           │  │     - Mobile Companion Discovery            ││
│           │  │                                             ││
│           │  └─────────────────────────────────────────────┘│
└───────────┴─────────────────────────────────────────────────┘
```

---

## Questions for User

1. **Tab groupings** - Are the proposed 4 categories (Connection, Devices, Content, System) acceptable, or would you prefer different groupings?

2. **Default tab** - Should "Connection" be the default landing tab, or another one?

3. **Rename route** - The current route is `/obs-config` which is a legacy name. Should we rename it to `/settings` as part of this work?
