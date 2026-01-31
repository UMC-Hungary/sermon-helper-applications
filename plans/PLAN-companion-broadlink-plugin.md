# Plan: Companion Plugin for Broadlink Actions

## Overview

Create a Bitfocus Companion module that allows users to trigger Broadlink IR/RF commands through the Companion interface by communicating with the Sermon Helper Tauri app's discovery server API.

## Architecture

```
┌─────────────────────────────────────┐     HTTP/WS      ┌─────────────────────────────────┐
│  Bitfocus Companion                 │ ◄──────────────► │  Sermon Helper (Tauri App)      │
│  ├─ companion-module-sermon-helper  │    Port 8765     │  ├─ Discovery Server            │
│  │  ├─ Actions                      │                  │  │  ├─ /api/v1/rfir/commands    │
│  │  ├─ Feedbacks                    │                  │  │  ├─ /api/v1/rfir/.../execute │
│  │  └─ Presets                      │                  │  │  └─ /ws (status updates)     │
│  └─ Stream Deck / Web UI            │                  │  └─ Broadlink UDP Protocol      │
└─────────────────────────────────────┘                  └─────────────────────────────────┘
```

## Components

### 1. Companion Module (New Package)

**Location:** Create as separate npm package `companion-module-sermon-helper`

**Structure:**
```
companion-module-sermon-helper/
├── package.json
├── tsconfig.json
├── companion/
│   ├── manifest.json          # Module metadata
│   └── HELP.md                # User documentation
├── src/
│   ├── main.ts                # Module entry point
│   ├── config.ts              # Configuration fields
│   ├── actions.ts             # Action definitions
│   ├── feedbacks.ts           # Status feedbacks
│   ├── presets.ts             # Pre-configured buttons
│   ├── variables.ts           # Variable definitions
│   └── api.ts                 # HTTP/WebSocket client
└── build/                     # Compiled output
```

### 2. Module Configuration

Users will configure:
- **Host**: Sermon Helper server IP/hostname (default: `localhost`)
- **Port**: API port (default: `8765`)
- **Auth Token**: Optional authentication token (if configured in Tauri app)
- **Auto-refresh**: Poll interval for command list updates

### 3. Actions

| Action ID | Name | Description | Options |
|-----------|------|-------------|---------|
| `execute_command` | Execute RF/IR Command | Trigger a saved command | `command_slug` (dropdown) |
| `execute_by_category` | Execute by Category | Filter and execute | `category`, `command_slug` |
| `refresh_commands` | Refresh Command List | Manually sync commands | none |

### 4. Feedbacks

| Feedback ID | Name | Description |
|-------------|------|-------------|
| `connection_status` | Connection Status | Show connected/disconnected state |
| `command_executed` | Last Command | Flash when command executes |

### 5. Variables

| Variable ID | Name | Description |
|-------------|------|-------------|
| `connection_status` | Connection Status | "Connected" / "Disconnected" |
| `last_command` | Last Executed | Name of last executed command |
| `command_count` | Total Commands | Number of available commands |

### 6. Presets (Auto-generated)

Generate preset buttons for each RF/IR command with:
- Command name as button text
- Category-based coloring
- Execute action on press

## Implementation Steps

### Phase 1: Project Setup

1. **Create module repository** using companion-module-template-ts
2. **Configure package.json** with proper metadata:
   ```json
   {
     "name": "companion-module-sermon-helper",
     "version": "1.0.0",
     "main": "dist/main.js",
     "scripts": {
       "build": "tsc",
       "dev": "tsc --watch"
     }
   }
   ```
3. **Create manifest.json** for Companion module registry

### Phase 2: Core Implementation

1. **API Client (`src/api.ts`)**
   - HTTP client for REST endpoints
   - WebSocket client for real-time updates
   - Retry logic and connection management
   - Command list caching

2. **Configuration (`src/config.ts`)**
   ```typescript
   export function GetConfigFields(): SomeCompanionConfigField[] {
     return [
       { id: 'host', type: 'textinput', label: 'Host', default: 'localhost' },
       { id: 'port', type: 'number', label: 'Port', default: 8765 },
       { id: 'authToken', type: 'textinput', label: 'Auth Token (optional)' },
       { id: 'pollInterval', type: 'number', label: 'Poll Interval (ms)', default: 5000 }
     ]
   }
   ```

3. **Actions (`src/actions.ts`)**
   - Fetch command list from `/api/v1/rfir/commands`
   - Build dynamic dropdown choices
   - Execute via `POST /api/v1/rfir/commands/:slug/execute`

4. **Main Module (`src/main.ts`)**
   - Initialize API client on `init()`
   - Cleanup WebSocket on `destroy()`
   - Refresh commands on `configUpdated()`

### Phase 3: Enhanced Features

1. **WebSocket Integration**
   - Connect to `/ws` endpoint
   - Listen for `RfIrCommandExecuted` messages
   - Update feedbacks in real-time

2. **Dynamic Command Discovery**
   - Periodically fetch updated command list
   - Regenerate action dropdowns when commands change
   - Auto-generate presets for new commands

3. **Category Support**
   - Group commands by category in UI
   - Color-coded presets per category:
     - Projector: Blue
     - Screen: Green
     - HVAC: Orange
     - Lighting: Yellow
     - Audio: Purple
     - Other: Gray

### Phase 4: Error Handling & UX

1. **Connection status indicator**
2. **Retry on connection failure**
3. **User-friendly error messages**
4. **Logging for debugging**

## API Endpoints Used

From the existing discovery server:

| Method | Endpoint | Purpose |
|--------|----------|---------|
| GET | `/api/v1/health` | Connection check |
| GET | `/api/v1/rfir/commands` | List all commands |
| GET | `/api/v1/rfir/commands/:slug` | Get command details |
| POST | `/api/v1/rfir/commands/:slug/execute` | Execute command |
| WS | `/ws` | Real-time status updates |

## Discovery Server Enhancements (Optional)

Consider adding to the Tauri app:

1. **mDNS advertisement** - Already implemented (`_sermon-helper._tcp`)
2. **OpenAPI spec** - Already available at `/api/v1/openapi.json`
3. **Command metadata** - Add icons/colors for Companion presets

## Testing Strategy

1. **Manual testing** with Companion in dev mode
2. **Mock server** for unit tests
3. **Integration tests** against running Tauri app

## Deliverables

1. `companion-module-sermon-helper` npm package
2. Documentation (HELP.md) for end users
3. README with setup instructions
4. Optional: Submit to Companion module repository

## Dependencies

**Companion Module:**
- `@companion-module/base` - Companion SDK
- `node-fetch` or built-in `fetch` - HTTP client
- `ws` - WebSocket client

**No changes required to Tauri app** - Uses existing API

## Timeline Considerations

- Phase 1 (Setup): Scaffolding and configuration
- Phase 2 (Core): Basic action execution
- Phase 3 (Enhanced): WebSocket, dynamic updates
- Phase 4 (Polish): Error handling, documentation

## User Decisions

1. **Location**: Monorepo under `packages/companion-module/`
2. **Discovery**: Both mDNS auto-discovery AND manual host configuration
3. **Scope**: RF/IR only for now (OBS control can be added later)
4. **Color Palette**:
   | Category | Color Name | Hex |
   |----------|------------|-----|
   | projector | Rosy Copper | #dd614a |
   | screen | Coral Glow | #f48668 |
   | hvac | Powder Blush | #f4a698 |
   | lighting | Dry Sage | #c5c392 |
   | audio | Muted Teal | #73a580 |
   | other | (default gray) | #808080 |
