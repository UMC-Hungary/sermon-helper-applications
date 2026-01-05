# Sermon Helper Tauri - Codebase Documentation

**Last Updated:** 2026-01-05
**Project Type:** Tauri 2 Desktop Application with SvelteKit Frontend
**Purpose:** Church streaming control application (Svelte port of React version)

## 📋 Project Overview

This is a desktop application for managing church livestreams and presentations. It's being migrated from a React/Next.js web app (`/home/church-control-app`) to a Tauri desktop app with SvelteKit.

### Tech Stack
- **Frontend:** SvelteKit 2.9.0 + Svelte 5.0.0 + TypeScript 5.6.2
- **Backend:** Tauri 2 + Rust
- **Styling:** Tailwind CSS 4.1.18
- **Icons:** lucide-svelte 0.562.0
- **Package Manager:** pnpm

## 🏗️ Project Structure

```
sermon-helper-tauri/
├── src/
│   ├── lib/
│   │   ├── components/
│   │   │   ├── ui/                    # Reusable UI primitives
│   │   │   │   ├── error-messages.svelte   ✅ Complete & synced with React
│   │   │   │   ├── alert.svelte           ✅ Complete
│   │   │   │   ├── alert-title.svelte     ✅ Complete
│   │   │   │   ├── alert-description.svelte ✅ Complete
│   │   │   │   ├── badge.svelte           ✅ Complete
│   │   │   │   ├── button.svelte          ✅ Complete
│   │   │   │   ├── card.svelte            ✅ Complete
│   │   │   │   ├── dialog.svelte          🔧 Stub (uses native HTML dialog)
│   │   │   │   ├── scroll-area.svelte     ✅ Complete
│   │   │   │   ├── separator.svelte       ✅ Complete
│   │   │   │   └── toaster.svelte         ❌ Placeholder only
│   │   │   ├── dashboard-view.svelte      ⏳ Stub with ErrorMessages
│   │   │   ├── bible-editor-view.svelte   ⏳ Stub with ErrorMessages
│   │   │   ├── youtube-schedule-view.svelte ⏳ Stub with ErrorMessages
│   │   │   ├── youtube-events-view.svelte  ⏳ Stub with ErrorMessages
│   │   │   ├── obs-settings-view.svelte   ⏳ Stub with ErrorMessages
│   │   │   └── sidebar.svelte             ✅ Complete
│   │   ├── stores/
│   │   │   ├── types.ts              # SystemStatus type definition
│   │   │   └── app-state.ts          # Initial state values
│   │   ├── types.ts                  # Extended type definitions
│   │   └── utils.ts                  # cn() utility for class merging
│   └── routes/
│       ├── +layout.svelte            # Root layout (simplified)
│       └── +page.svelte              # Main entry point with view routing
├── src-tauri/                        # Rust backend
│   ├── src/
│   │   ├── main.rs                   # Tauri entry point
│   │   └── lib.rs                    # Tauri commands
│   ├── Cargo.toml                    # Rust dependencies
│   └── tauri.conf.json               # Tauri configuration
├── static/                           # Static assets
├── package.json                      # Node dependencies
└── vite.config.js                    # Vite configuration
```

## ✅ Recent Work Completed (2026-01-05)

### Error Messages Component Synchronization
**Status:** Fully synchronized with React version at `/home/church-control-app`

#### Changes Made:
1. **error-messages.svelte** (220 lines) - Complete rewrite
   - Fixed SystemStatus type to use flat boolean structure
   - Replaced inline SVG with lucide-svelte icons (AlertCircle, Info, RefreshCw)
   - Implemented proper UI component composition (Alert, AlertTitle, AlertDescription, Badge, Button)
   - Added native HTML5 `<dialog>` modal for detailed troubleshooting steps
   - Matched all error message text word-for-word with React version
   - Implemented image display with error handling
   - All CSS classes match React exactly

2. **Alert Components** - Updated to match React grid layout
   - `alert.svelte`: Grid layout with icon support, data-slot attributes
   - `alert-title.svelte`: Proper column positioning (col-start-2)
   - `alert-description.svelte`: Grid layout with flex support for buttons

3. **Button Component** - Fixed class composition
   - Removed incorrect default classes from baseClasses
   - Added data-slot="button" attribute
   - Fixed TypeScript errors with reactive variant/size class lookups
   - Now generates identical CSS to React version

4. **Badge Component** - TypeScript fixes
   - Added reactive variant class lookup to prevent undefined index errors

5. **View Components** - Restructured to match React
   - ErrorMessages now rendered inside each view (not in root page)
   - All views wrapped with `<div class="p-4 lg:p-8 space-y-6 pt-20 lg:pt-8">`
   - ErrorMessages is first child in each view
   - Views: dashboard-view, bible-editor-view, youtube-schedule/events-view, obs-settings-view

6. **Layout Simplification**
   - `+layout.svelte`: Simplified to just import CSS and render slot
   - `+page.svelte`: Handles all view routing and ErrorMessages removed from here

## 🔑 Key Type Definitions

### SystemStatus (Flat Structure)
```typescript
// src/lib/stores/types.ts
export type SystemStatus = {
  obs: boolean;
  rodeInterface: boolean;
  mainDisplay: boolean;
  secondaryDisplay: boolean;
  airplayDisplay: boolean;
  displayAlignment: boolean;
  youtubeLoggedIn: boolean;
}
```

**Note:** There's also a nested version in `src/lib/types.ts` - the flat version in `stores/types.ts` is the correct one matching React.

## 🎯 Error Messages System

### Error Definitions
5 error types with detailed troubleshooting steps:

1. **airplayDisplay** - AirPlay Display Not Connected (6 steps + image)
2. **displayAlignment** - Display Alignment Incorrect (6 steps + image)
3. **obs** - OBS Not Running (5 steps + image)
4. **rodeInterface** - Rode Audio Interface Not Connected (6 steps + image)
5. **youtubeLoggedIn** - YouTube Not Logged In (5 steps)

### Error Message Interface
```typescript
interface ErrorMessage {
  id: string;
  title: string;
  description: string;
  status: keyof SystemStatus;  // Maps to boolean flag
  detailedSteps: string[];
  imageUrl?: string;           // Reference images for troubleshooting
}
```

### Component Hierarchy
```
View Component
└── ErrorMessages
    ├── Header (AlertCircle icon, "System Issues Detected", Badge, Re-check button)
    └── Error List (space-y-3)
        └── Alert (destructive variant) for each error
            ├── AlertCircle icon
            ├── AlertTitle
            └── AlertDescription
                ├── Error description text
                └── "Read More" Button → Opens dialog
                    └── Native HTML5 <dialog>
                        ├── Error title & description
                        ├── Reference image (if available)
                        └── Ordered list of troubleshooting steps
```

## 🎨 Styling Conventions

### Tailwind CSS
- Using Tailwind v4 with custom CSS variables for theming
- CSS variable pattern: `var(--spacing)`, `var(--foreground)`, etc.
- Dark mode support via `dark:` prefix

### Class Utility
```typescript
// src/lib/utils.ts
import { clsx } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}
```
**Usage:** Merges Tailwind classes intelligently, later classes override earlier ones

### Component Patterns
- All UI components accept `className` prop for custom styling
- Use `data-slot` attributes for scoped styling (matching React pattern)
- Reactive statements (`$:`) for computed classes

## ⚠️ Known Issues & Warnings

### Build Status
- ✅ 0 TypeScript errors
- ⚠️ 15 warnings (all about unused export properties in stub view components)
- These warnings are expected and will resolve when views are fully implemented

### Incomplete Components
1. **Dialog Component** (`ui/dialog.svelte`)
   - Currently a simplified stub
   - ErrorMessages uses native HTML5 `<dialog>` instead
   - Future: Could be replaced with proper Radix-style dialog

2. **Toaster Component** (`ui/toaster.svelte`)
   - Placeholder only, not implemented
   - React version uses Sonner for toast notifications

3. **View Components**
   - All views are stubs with "TODO: implement" placeholders
   - ErrorMessages integration is complete
   - Props and event handlers are defined but unused

## 📚 Reference: React Version

**Location:** `/home/church-control-app`

### Key Files to Reference:
- `components/error-messages.tsx` - Source of truth for error messages (188 lines)
- `components/ui/alert.tsx` - Alert component implementation (67 lines)
- `components/ui/button.tsx` - Button variants and sizes (60 lines)
- `components/dashboard-view.tsx` - Dashboard implementation reference (284 lines)
- `app/page.tsx` - Main app structure (108 lines)

## 🔄 State Management

### Current Approach
- Local state in `+page.svelte`
- Props passed down to view components
- No global store implementation yet

### SystemStatus Default Values
```typescript
// All false except these:
obs: true
rodeInterface: true
mainDisplay: true
secondaryDisplay: true

// These show errors by default:
airplayDisplay: false
displayAlignment: false
youtubeLoggedIn: false
```

## 🚀 Next Steps / TODO

### High Priority
1. **Implement Dashboard View**
   - Reference: `/home/church-control-app/components/dashboard-view.tsx`
   - Sermon title management (OBS integration)
   - Bible text inputs (Textus/Leckio)
   - PPT generation buttons
   - Stream control buttons
   - Navigation cards to other views

2. **Implement Bible Editor View**
   - Reference: `/home/church-control-app/components/bible-editor-view.tsx`
   - Bible verse search and selection
   - Text display and editing
   - Copy to clipboard functionality

3. **Implement YouTube Schedule View**
   - Reference: `/home/church-control-app/components/youtube-schedule-view.tsx`
   - YouTube login flow
   - Event scheduling form
   - Live stream management

### Medium Priority
4. **Implement Toast Notifications**
   - Replace toaster.svelte placeholder
   - Add toast hook similar to React's use-toast.ts
   - User feedback for actions

5. **Backend Integration**
   - Implement Tauri commands for system checks
   - OBS WebSocket integration
   - Audio device detection
   - Display detection
   - YouTube API integration

6. **Proper Dialog Component**
   - Consider using melt-ui or bits-ui for Svelte
   - Replace native `<dialog>` in error-messages if needed

### Low Priority
7. **Testing Setup**
   - Add Vitest for unit tests
   - Playwright for E2E tests

8. **Documentation**
   - Add JSDoc comments to complex functions
   - Create user manual

## 🔧 Development Commands

```bash
# Install dependencies
pnpm install

# Run dev server (Vite only)
pnpm dev

# Run Tauri app in development
pnpm tauri dev

# Build for production
pnpm tauri build

# Type checking
pnpm check

# Format code
pnpm format
```

## 📝 Git Workflow

### Current Status
- Initialized on 2026-01-05
- Branch: `main`
- Initial commit: `b76b2d6` - "Initial commit: Sermon Helper Tauri app with Svelte"

### .gitignore Coverage
- ✅ node_modules, build artifacts
- ✅ .svelte-kit, .env files
- ✅ src-tauri/target (Rust builds)
- ✅ IDE files (.idea, .vscode/*)
- ✅ OS files (.DS_Store, Thumbs.db)

## 🤝 Related Projects

### Church Control App (React Version)
- **Path:** `/home/church-control-app`
- **Status:** Source of truth for features
- **Tech:** Next.js 16 + React 19 + TypeScript
- **Purpose:** Reference implementation

## 💡 Important Notes

1. **Type Definition Duplication**
   - `src/lib/stores/types.ts` has flat SystemStatus (✅ correct)
   - `src/lib/types.ts` has nested SystemStatus (⚠️ outdated)
   - TODO: Remove or consolidate type definitions

2. **Component Prop Patterns**
   - Event handlers: `export let onEventName: () => void`
   - Use `export let` for props that will be used
   - Use `export const` for props that are just API surface (per Svelte warning)

3. **CSS Class Matching**
   - All UI components' CSS classes must match React version exactly
   - Use same class ordering for consistency
   - Test by comparing rendered HTML

4. **Dialog Implementation**
   - Current: Native HTML5 `<dialog>` element
   - Works well for error messages modal
   - Browser support: Modern browsers only

## 📞 Support & Questions

When working on this codebase:
- Always compare with React version at `/home/church-control-app`
- Check CLAUDE_NOTES.md for latest status
- Run `pnpm check` before committing
- Test error messages display with different SystemStatus states

---

**Document Version:** 1.0
**Maintained By:** Claude Code
**Last Modified:** 2026-01-05
