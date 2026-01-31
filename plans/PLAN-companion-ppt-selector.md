# Plan: Companion PPT Selector Plugin Feature

## Overview

Add a PowerPoint file selector feature to the existing Companion module that allows users to browse and filter PPT files from configured folders using numeric button presses on a Stream Deck or similar controller.

## User Experience

### Concept
Users can navigate to PPT files without a keyboard by:
1. Pressing number buttons (0-9) to build a filter string
2. Viewing matching filenames on display buttons (1-3 slots)
3. Pressing a selection button when the desired file is shown
4. Using a backspace button to remove the last typed digit

### Example Flow
```
Folder contains: ["01-Welcome.pptx", "02-Worship.pptx", "10-Sermon.pptx", "11-Closing.pptx"]

Initial state: Display shows first 3 files
  [01-Welcome] [02-Worship] [10-Sermon]

User presses "1":
  Filter: "1"
  [10-Sermon] [11-Closing] [--empty--]

User presses "0":
  Filter: "10"
  [10-Sermon] [--empty--] [--empty--]

User presses file slot 1:
  Opens/stages "10-Sermon.pptx"
```

---

## Architecture

### Component Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                     Bitfocus Companion                          │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │           companion-module-sermon-helper                 │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────────────────┐  │   │
│  │  │ Actions  │  │ Feedbacks│  │ Variables            │  │   │
│  │  │ - Digit  │  │ - Slot 1 │  │ - ppt_filter         │  │   │
│  │  │ - Clear  │  │ - Slot 2 │  │ - ppt_slot_1_name    │  │   │
│  │  │ - Select │  │ - Slot 3 │  │ - ppt_slot_2_name    │  │   │
│  │  │ - Backsp │  │          │  │ - ppt_slot_3_name    │  │   │
│  │  └──────────┘  └──────────┘  │ - ppt_match_count    │  │   │
│  │                              └──────────────────────────┘   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ HTTP/WebSocket
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Tauri Backend (Rust)                         │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                  discovery_server.rs                     │   │
│  │  New Endpoints:                                          │   │
│  │  - GET  /api/v1/ppt/folders     (list configured folders)│   │
│  │  - POST /api/v1/ppt/folders     (add folder)             │   │
│  │  - DELETE /api/v1/ppt/folders/:id                        │   │
│  │  - GET  /api/v1/ppt/files       (list files, filter)     │   │
│  │  - POST /api/v1/ppt/open        (open selected file)     │   │
│  └─────────────────────────────────────────────────────────┘   │
│                              │                                  │
│                              ▼                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                   File System                            │   │
│  │  - Scans configured folders for .ppt/.pptx/.odp files   │   │
│  │  - Returns sorted file list                              │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

---

## Implementation Tasks

### Phase 1: Backend API (Rust/Tauri)

#### 1.1 Add PPT Folder Storage
- Add `ppt_folders: Vec<PptFolder>` to shared state in `discovery_server.rs`
- Create `PptFolder` struct:
  ```rust
  struct PptFolder {
      id: String,
      path: String,
      name: String,  // Display name for Companion
  }
  ```
- Add Tauri command to update folders: `update_ppt_folders(folders: Vec<PptFolder>)`

#### 1.2 Add PPT API Endpoints
- `GET /api/v1/ppt/folders` - List configured folders
- `POST /api/v1/ppt/folders` - Add a new folder
- `DELETE /api/v1/ppt/folders/{id}` - Remove a folder
- `GET /api/v1/ppt/files?folder_id=X&filter=Y` - List files with optional filter
  - Response: `{ files: [{ id, name, path, folder_id }], total: number }`
  - Filter matches files where name starts with the filter string
- `POST /api/v1/ppt/open` - Open a PPT file
  - Body: `{ file_path: string }`
  - Uses system default application (PowerPoint, LibreOffice, etc.)

#### 1.3 File System Scanning
- Scan folder for files matching: `*.ppt`, `*.pptx`, `*.odp`
- Return sorted list (alphanumeric)
- Handle permission errors gracefully
- Cache file list with 5-second TTL to avoid excessive disk reads

---

### Phase 2: Companion Plugin Updates

#### 2.1 Configuration Extensions
Add to `config.ts`:
```typescript
{
  id: 'pptDisplaySlots',
  type: 'number',
  label: 'PPT Display Slots',
  default: 3,
  min: 1,
  max: 5,
  tooltip: 'Number of file slots to display (1-5)'
}
```

#### 2.2 State Management
Add to plugin instance state:
```typescript
interface PptSelectorState {
  currentFilter: string           // e.g., "10"
  matchingFiles: PptFile[]        // Filtered file list
  selectedFolderId: string | null // Active folder
  folders: PptFolder[]            // Available folders
}
```

#### 2.3 New Actions
| Action ID | Name | Options | Behavior |
|-----------|------|---------|----------|
| `ppt_digit` | PPT: Type Digit | digit: 0-9 | Appends digit to filter, refreshes list |
| `ppt_backspace` | PPT: Backspace | - | Removes last digit from filter |
| `ppt_clear` | PPT: Clear Filter | - | Clears filter completely |
| `ppt_select_slot` | PPT: Select File | slot: 1-5 | Opens file in that slot position |
| `ppt_select_folder` | PPT: Select Folder | folder (dropdown) | Switches active folder - **dropdown populated from Tauri API** |
| `ppt_refresh` | PPT: Refresh Files | - | Re-scans folder |
| `ppt_generate_page` | PPT: Generate Button Page | pageName, displaySlots | **Auto-creates a complete Companion page with all PPT buttons** |

**Note:** The `ppt_select_folder` action dropdown is dynamically populated by fetching `/api/v1/ppt/folders` from the Tauri backend. When folders are added/removed in the Tauri app, Companion receives updates via WebSocket and refreshes the action options.

#### 2.4 New Variables
| Variable ID | Label | Example Value |
|-------------|-------|---------------|
| `ppt_filter` | PPT Filter | "10" |
| `ppt_slot_1_name` | PPT Slot 1 | "10-Sermon" |
| `ppt_slot_2_name` | PPT Slot 2 | "11-Closing" |
| `ppt_slot_3_name` | PPT Slot 3 | "" |
| `ppt_slot_4_name` | PPT Slot 4 | "" |
| `ppt_slot_5_name` | PPT Slot 5 | "" |
| `ppt_match_count` | PPT Matches | "2" |
| `ppt_folder_name` | PPT Folder | "Sunday Services" |

#### 2.5 New Feedbacks
| Feedback ID | Name | Style When True |
|-------------|------|-----------------|
| `ppt_slot_has_file` | PPT: Slot Has File | Background color change |
| `ppt_filter_active` | PPT: Filter Active | Highlight when filter not empty |

#### 2.6 Presets (Option 1: Preset Bundle)
Create preset category "PPT Selector" with individual draggable presets:

**Digit Buttons (0-9):**
- 10 presets, one per digit
- Action: `ppt_digit` with respective digit
- Label: The digit number
- Color: `0x3B82F6` (Blue)

**Control Buttons:**
- `PPT: Backspace` - action: `ppt_backspace`, label: "⌫", color: `0xEF4444` (Red)
- `PPT: Clear` - action: `ppt_clear`, label: "CLR", color: `0xF59E0B` (Amber)
- `PPT: Refresh` - action: `ppt_refresh`, label: "↻", color: `0x6B7280` (Gray)

**Display Slots:**
- 5 presets: `PPT: Slot 1` through `PPT: Slot 5`
- Text: `$(sermon-helper:ppt_slot_N_name)`
- Action: `ppt_select_slot` for slot N
- Feedback: `ppt_slot_has_file` - green when file present, gray when empty

**Status Display:**
- `PPT: Filter Status` - shows `Filter: $(sermon-helper:ppt_filter)` and `$(sermon-helper:ppt_match_count) matches`

**Folder Selection:**
- `PPT: Select Folder` - dropdown action with current folder name display

#### 2.7 Auto-Generate Page Action (Option 3: Quick Setup)

Add special action: **"PPT: Generate Button Page"**

**Behavior:**
1. User triggers action (can be placed on any button temporarily)
2. Plugin creates a new page named "PPT Selector" in Companion
3. Auto-populates with complete button layout:

```
┌─────┬─────┬─────┬─────┬─────┐
│  1  │  2  │  3  │  4  │  5  │  Row 1: Digits 1-5
├─────┼─────┼─────┼─────┼─────┤
│  6  │  7  │  8  │  9  │  0  │  Row 2: Digits 6-0
├─────┼─────┼─────┼─────┼─────┤
│  ⌫  │ CLR │  ↻  │ 📁  │STAT │  Row 3: Controls
├─────┼─────┼─────┼─────┼─────┤
│ S1  │ S2  │ S3  │ S4  │ S5  │  Row 4: Display Slots
└─────┴─────┴─────┴─────┴─────┘
```

4. Shows success toast/feedback when complete

**Implementation:**
- Uses Companion's internal API to create page and buttons programmatically
- Requires `companion.pages` and `companion.buttons` API access
- Falls back to error message if API not available

**Action Options:**
| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `pageName` | text | "PPT Selector" | Name for the generated page |
| `displaySlots` | number | 5 | How many display slots (1-5) |
| `includeAllDigits` | boolean | true | Include all 10 digit buttons |

---

### Phase 3: Frontend Settings UI (Tauri App)

#### 3.1 PPT Folders Settings Component
Create `src/lib/components/ppt-folder-settings.svelte`:
- List configured folders in a table/card layout
- Each folder shows: display name, path, file count, delete button
- "Add Folder" button opens **native folder picker** via Tauri `dialog.open()`
- Editable display name input for each folder (used in Companion dropdown)
- Real-time file count preview (scans folder on add)
- Save/sync to Tauri store and backend state

#### 3.2 UI Mockup
```
┌─────────────────────────────────────────────────────────┐
│ PPT Folders                                      [+ Add]│
├─────────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────────────────────────┐ │
│ │ 📁 Sermons                                    [🗑️] │ │
│ │    C:\Church\Presentations\Sermons                  │ │
│ │    12 files                                         │ │
│ └─────────────────────────────────────────────────────┘ │
│ ┌─────────────────────────────────────────────────────┐ │
│ │ 📁 Worship Sets                               [🗑️] │ │
│ │    C:\Church\Presentations\Worship                  │ │
│ │    8 files                                          │ │
│ └─────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

#### 3.3 Integration
- Add new route or section in settings page
- Use Tauri `dialog` plugin for native folder picker
- Sync folder configuration with backend via `update_ppt_folders()` command
- Backend broadcasts folder changes to connected Companion instances via WebSocket

---

## File Changes Summary

### New Files
| File | Purpose |
|------|---------|
| `packages/companion-module-sermon-helper/src/ppt-selector.ts` | PPT selector state and logic |
| `src/lib/components/ppt-folder-settings.svelte` | Frontend folder config UI |

### Modified Files
| File | Changes |
|------|---------|
| `src-tauri/src/discovery_server.rs` | Add PPT API endpoints, folder storage |
| `packages/.../src/main.ts` | Initialize PPT selector state |
| `packages/.../src/actions.ts` | Add PPT actions |
| `packages/.../src/variables.ts` | Add PPT variables |
| `packages/.../src/feedbacks.ts` | Add PPT feedbacks |
| `packages/.../src/presets.ts` | Add PPT preset buttons |
| `packages/.../src/config.ts` | Add display slots config |
| `packages/.../src/api.ts` | Add PPT API methods |
| `packages/.../src/types.ts` | Add PPT type definitions |

---

## API Specification

### GET /api/v1/ppt/folders
**Response:**
```json
{
  "folders": [
    { "id": "uuid-1", "path": "C:/Church/Sermons", "name": "Sermons" },
    { "id": "uuid-2", "path": "C:/Church/Worship", "name": "Worship Sets" }
  ]
}
```

### GET /api/v1/ppt/files
**Query Parameters:**
- `folder_id` (required): Folder UUID
- `filter` (optional): Numeric prefix filter

**Response:**
```json
{
  "files": [
    { "id": "1", "name": "10-Sermon.pptx", "path": "C:/Church/Sermons/10-Sermon.pptx" },
    { "id": "2", "name": "11-Closing.pptx", "path": "C:/Church/Sermons/11-Closing.pptx" }
  ],
  "total": 2,
  "filter": "1"
}
```

### POST /api/v1/ppt/open
**Request:**
```json
{
  "file_path": "C:/Church/Sermons/10-Sermon.pptx",
  "start_presenter": true
}
```

**Response:**
```json
{ "success": true }
```

**Behavior:**
1. Opens the file with system default application
2. Waits briefly for application to load (~2-3 seconds)
3. Triggers presenter mode via platform-specific automation:
   - **Windows**: PowerPoint COM automation or SendKeys (F5)
   - **macOS**: AppleScript to control Keynote/PowerPoint

### WebSocket Messages (New)

**Server → Client:**
```json
{
  "type": "PptFoldersChanged",
  "folders": [
    { "id": "uuid-1", "path": "C:/Church/Sermons", "name": "Sermons" }
  ]
}
```

**Server → Client:**
```json
{
  "type": "PptFileOpened",
  "file": { "name": "10-Sermon.pptx", "path": "C:/Church/Sermons/10-Sermon.pptx" },
  "success": true,
  "presenterStarted": true
}
```

---

## Design Decisions

1. **File opening behavior**: Open with system default app, then trigger presenter mode via PowerPoint COM automation (Windows) or AppleScript (macOS)

2. **Folder configuration**: Configured in **Tauri app** with native folder picker UI. Companion fetches available folders from API and user selects active folder via action.

3. **Multi-folder support**: Folder selection is a separate Companion action - numpad is only for filtering files within the selected folder

4. **File name display**: User configures how many display buttons they want (1-5). Long names will be truncated with ellipsis

5. **Filter type**: Numeric-only filter - files should be named with numeric prefixes (e.g., "01-Welcome.pptx")

6. **Pagination**: No pagination - show first 5 matches sorted by relevance (closest numeric match first)

---

## Presenter Mode Implementation

### Windows (Primary Target)
Since PowerPoint is the most common presentation software on Windows:

**Option A: PowerShell + COM Automation (Recommended)**
```powershell
$ppt = New-Object -ComObject PowerPoint.Application
$ppt.Visible = $true
$presentation = $ppt.Presentations.Open("C:\path\to\file.pptx")
$presentation.SlideShowSettings.Run()
```

**Option B: SendKeys Fallback**
- Open file with `opener` crate (system default)
- Wait 2-3 seconds for app to load
- Send F5 key via Windows API to start slideshow

### macOS (Future Support)
**AppleScript for Keynote:**
```applescript
tell application "Keynote"
    open file "path/to/file.key"
    start slideshow
end tell
```

### Rust Implementation
- Create new module `src-tauri/src/presentation.rs`
- Use `std::process::Command` to run PowerShell scripts
- Fallback to `opener` crate + simulated keypress if COM fails

---

## Implementation Order

1. **Backend API** - Rust endpoints for folder/file management
2. **Backend Presenter Mode** - PowerShell/COM automation for slideshow
3. **Plugin types & API client** - TypeScript interfaces and HTTP methods
4. **Plugin state management** - PPT selector state handling
5. **Plugin actions** - Digit, select, clear, folder actions
6. **Plugin variables & feedbacks** - Display state
7. **Plugin presets** - Individual button presets (Option 1)
8. **Plugin page generator** - Auto-generate complete page action (Option 3)
9. **Frontend settings** - Folder configuration UI with native picker
10. **Testing & polish** - End-to-end testing

---

## Estimated Scope

- **Backend (Rust)**: ~300 lines new code (includes presentation automation)
- **Plugin (TypeScript)**: ~500 lines new code (includes page generator)
- **Frontend (Svelte)**: ~150 lines new code
- **Total**: ~950 lines of new code across 12+ files

### New Rust Modules
- `src-tauri/src/ppt.rs` - PPT folder/file management
- `src-tauri/src/presentation.rs` - Presenter mode automation

### New Plugin Files
- `src/ppt-selector.ts` - PPT state management and filtering logic
- `src/ppt-page-generator.ts` - Auto-generate Companion page with buttons
