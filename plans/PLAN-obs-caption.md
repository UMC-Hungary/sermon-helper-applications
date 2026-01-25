# Plan: OBS Caption Editor Feature

## Overview

Implement an OBS caption editor within the Tauri app that allows users to create embeddable captions for OBS browser sources. The caption HTML will be served via the existing Discovery Server HTTP service.

## Reference Implementation Analysis

Based on the existing implementation at `nyiregyhazimetodista.hu/obs-caption-editor.html`:
- **Form fields:** type, title, bold text, light text, color, logo visibility
- **Output:** URL with query parameters pointing to a caption HTML page
- **Usage:** URL is used as OBS Browser Source

## Feature Requirements

1. **Caption Editor Form** - SvelteKit page with form inputs
2. **SVG Logo Input** - User can paste/upload SVG logo content
3. **Caption HTML Endpoint** - Extend Discovery Server to serve embeddable HTML
4. **Persistent Settings** - Store caption configuration using existing store system

---

## Implementation Tasks

### Task 1: Create Caption Editor Route

**File:** `src/routes/obs-caption/+page.svelte`

Create a new page with form fields:

| Field | Type | Description |
|-------|------|-------------|
| `type` | select | "caption" (150px) or "full" (1080px) |
| `title` | text | Main heading text |
| `boldText` | text | Bold/emphasized content |
| `lightText` | text | Secondary/light content |
| `color` | select | Theme color ("black", "red", or custom) |
| `showLogo` | toggle | Logo visibility |
| `svgLogo` | textarea | SVG logo content (raw SVG markup) |

**Output Display:**
- Generated URL for OBS Browser Source
- Copy-to-clipboard button
- Recommended dimensions (width: 1920, height: 150/1080)

---

### Task 2: Create Caption Store

**File:** `src/lib/stores/captionStore.ts`

```typescript
interface CaptionSettings {
  type: 'caption' | 'full';
  title: string;
  boldText: string;
  lightText: string;
  color: 'black' | 'red' | string;
  showLogo: boolean;
  svgLogo: string; // Raw SVG content
}
```

- Use existing 3-tier storage pattern (Tauri store → localStorage → in-memory)
- Persist settings across sessions
- Provide reactive store for form binding

---

### Task 3: Extend Discovery Server with Caption Endpoint

**File:** `src-tauri/src/discovery_server.rs`

Add new endpoint: `GET /caption`

**Query Parameters:**
- `type` - caption type
- `title` - URL-encoded title
- `bold` - URL-encoded bold text
- `light` - URL-encoded light text
- `color` - color theme
- `showLogo` - "visible" or "hidden"

**Response:** Standalone HTML page with inline CSS

```html
<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <style>
    /* Caption styling - matches original implementation */
  </style>
</head>
<body>
  <div class="caption-container">
    <div class="logo">{SVG if visible}</div>
    <div class="content">
      <h1 class="title">{title}</h1>
      <span class="bold">{boldText}</span>
      <span class="light">{lightText}</span>
    </div>
  </div>
</body>
</html>
```

---

### Task 4: Add SVG Logo Storage Endpoint

**File:** `src-tauri/src/discovery_server.rs`

Add endpoint to store/retrieve SVG logo:
- `POST /api/v1/caption/logo` - Store SVG content
- `GET /api/v1/caption/logo` - Retrieve stored SVG

Store SVG in shared state or Tauri store for persistence.

---

### Task 5: Add Tauri Commands for Caption Settings

**File:** `src-tauri/src/caption_commands.rs` (new)

Commands to expose to frontend:
- `save_caption_settings` - Persist caption configuration
- `load_caption_settings` - Load saved configuration
- `save_caption_logo` - Store SVG logo
- `load_caption_logo` - Retrieve SVG logo

Register in `lib.rs` invoke handler.

---

### Task 6: Update Sidebar Navigation

**File:** `src/lib/components/sidebar.svelte`

Add navigation item:
```svelte
<NavItem href="/obs-caption" icon={CaptionIcon}>
  OBS Caption
</NavItem>
```

---

## File Structure

```
src/
├── routes/
│   └── obs-caption/
│       └── +page.svelte          # Caption editor form
├── lib/
│   └── stores/
│       └── captionStore.ts       # Caption settings store

src-tauri/src/
├── caption_commands.rs           # New: Tauri commands
├── discovery_server.rs           # Modified: Add /caption endpoint
├── lib.rs                        # Modified: Register commands
```

---

## Generated URL Format

```
http://{local-ip}:8765/caption?type=caption&title=Welcome&bold=Speaker%20Name&light=Topic&color=black&showLogo=visible
```

User copies this URL into OBS as a Browser Source with dimensions 1920x150 (or 1920x1080 for full type).

---

## Caption HTML Styling (Reference)

Based on original implementation, the caption should:
- Use flexbox layout with logo on left, text content on right
- Support configurable background colors (black, red)
- White text on dark backgrounds
- Proper font sizing hierarchy (title > bold > light)
- Transparent background option for OBS overlay
- Responsive width (100vw)

---

## Implementation Order

1. **Task 2** - Create caption store (foundation)
2. **Task 5** - Add Tauri commands (backend support)
3. **Task 3** - Add Discovery Server caption endpoint
4. **Task 4** - Add SVG logo storage endpoint
5. **Task 1** - Create caption editor UI
6. **Task 6** - Update sidebar navigation

---

## Out of Scope (Future)

- Live preview in editor (mentioned by user)
- Animation/transition effects
- Multiple caption presets
- Real-time caption updates via WebSocket

---

## Dependencies

- Existing Discovery Server infrastructure (Axum on port 8765)
- Existing Tauri store plugin
- Existing UI components (button, card, input, select, switch)
