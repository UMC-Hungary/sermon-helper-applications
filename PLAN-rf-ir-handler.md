# Plan: RF/IR Handler Feature

## Overview

Add Broadlink RF/IR device support to control projectors, motorized screens, and other IR/RF-controlled devices. This feature includes device discovery, learning mode, command management, and API access.

---

## Phase 1: Type Definitions & Data Structures

### 1.1 Create RF/IR Type Definitions

**File:** `src/lib/types/rf-ir.ts`

```typescript
export interface BroadlinkDevice {
  id: string;              // UUID
  name: string;            // User-friendly name (e.g., "Living Room RM4")
  type: string;            // Device type hex (e.g., "0x5f36")
  model: string;           // Model name (e.g., "RM4 Pro")
  host: string;            // IP address
  mac: string;             // MAC address (format: aa:bb:cc:dd:ee:ff)
  lastSeen: number;        // Timestamp
  isDefault: boolean;      // Use as default device
}

export interface RfIrCommand {
  id: string;              // UUID
  name: string;            // Display name (e.g., "Projector Power On")
  slug: string;            // URL-safe identifier (e.g., "projector-power-on")
  deviceId: string;        // Associated Broadlink device ID
  code: string;            // Hex-encoded IR/RF signal data
  type: 'ir' | 'rf';       // Signal type
  category: string;        // Category for grouping (e.g., "projector", "screen", "hvac")
  createdAt: number;       // Timestamp
  updatedAt: number;       // Timestamp
}

export interface RfIrSettings {
  enabled: boolean;
  autoDiscovery: boolean;
  discoveryTimeout: number;      // Seconds (default: 5)
  devices: BroadlinkDevice[];
  commands: RfIrCommand[];
}

export interface LearnModeState {
  active: boolean;
  deviceId: string | null;
  type: 'ir' | 'rf' | null;
  learnedCode: string | null;
  error: string | null;
}

export const DEFAULT_RF_IR_SETTINGS: RfIrSettings = {
  enabled: false,
  autoDiscovery: true,
  discoveryTimeout: 5,
  devices: [],
  commands: []
};
```

### 1.2 Slug Generation Utility

**File:** `src/lib/utils/slug.ts`

```typescript
export function generateSlug(name: string, existingSlugs: string[]): string {
  // Convert to lowercase, replace spaces/special chars with hyphens
  let base = name
    .toLowerCase()
    .trim()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');

  // Ensure uniqueness
  let slug = base;
  let counter = 1;
  while (existingSlugs.includes(slug)) {
    slug = `${base}-${counter}`;
    counter++;
  }
  return slug;
}
```

---

## Phase 2: Rust Backend - Broadlink Integration

### 2.1 Add Broadlink Dependencies

**File:** `src-tauri/Cargo.toml`

```toml
[dependencies]
# ... existing deps
tokio = { version = "1", features = ["full", "process"] }
serde_json = "1"
```

### 2.2 Create Broadlink Module

**File:** `src-tauri/src/broadlink.rs`

Key functions to implement:

```rust
// Device discovery - runs Python script or native implementation
pub async fn discover_devices(timeout: u32) -> Result<Vec<BroadlinkDevice>, String>

// Enter learn mode on device
pub async fn enter_learn_mode(device: &BroadlinkDevice, signal_type: &str) -> Result<(), String>

// Check if code was learned
pub async fn check_learned_code(device: &BroadlinkDevice) -> Result<Option<String>, String>

// Exit learn mode
pub async fn exit_learn_mode(device: &BroadlinkDevice) -> Result<(), String>

// Send IR/RF code
pub async fn send_code(device: &BroadlinkDevice, code: &str) -> Result<(), String>
```

**Implementation approach:** Use the Python `broadlink` library via subprocess, similar to sermon-helper-service. Create a Python script bundled with the app:

**File:** `src-tauri/scripts/broadlink_bridge.py`

```python
#!/usr/bin/env python3
import sys
import json
import broadlink

def discover(timeout=5):
    """Discover Broadlink devices on network"""
    devices = broadlink.discover(timeout=timeout)
    result = []
    for d in devices:
        if d.auth():
            result.append({
                'type': hex(d.devtype),
                'model': d.model,
                'host': d.host[0],
                'mac': ':'.join(format(x, '02x') for x in d.mac),
                'name': d.name or d.model
            })
    print(json.dumps(result))

def learn(host, mac, devtype, signal_type='ir'):
    """Enter learn mode and wait for signal"""
    device = broadlink.gendevice(int(devtype, 16), (host, 80), bytes.fromhex(mac.replace(':', '')))
    device.auth()
    if signal_type == 'rf':
        device.sweep_frequency()
        # Wait and check for frequency
        # Then enter RF learn mode
    else:
        device.enter_learning()
    # Poll for learned data
    for _ in range(30):  # 30 second timeout
        time.sleep(1)
        try:
            data = device.check_data()
            if data:
                print(json.dumps({'code': data.hex()}))
                return
        except:
            pass
    print(json.dumps({'error': 'Learning timeout'}))

def send(host, mac, devtype, code):
    """Send IR/RF code to device"""
    device = broadlink.gendevice(int(devtype, 16), (host, 80), bytes.fromhex(mac.replace(':', '')))
    device.auth()
    device.send_data(bytes.fromhex(code))
    print(json.dumps({'success': True}))

if __name__ == '__main__':
    cmd = sys.argv[1]
    if cmd == 'discover':
        discover(int(sys.argv[2]) if len(sys.argv) > 2 else 5)
    elif cmd == 'learn':
        learn(sys.argv[2], sys.argv[3], sys.argv[4], sys.argv[5] if len(sys.argv) > 5 else 'ir')
    elif cmd == 'send':
        send(sys.argv[2], sys.argv[3], sys.argv[4], sys.argv[5])
```

### 2.3 Register Tauri Commands

**File:** `src-tauri/src/broadlink_commands.rs`

```rust
#[tauri::command]
pub async fn broadlink_discover(timeout: Option<u32>) -> Result<Vec<BroadlinkDevice>, String>

#[tauri::command]
pub async fn broadlink_learn(
    device_id: String,
    signal_type: String  // "ir" or "rf"
) -> Result<String, String>  // Returns learned hex code

#[tauri::command]
pub async fn broadlink_send(
    device_id: String,
    code: String
) -> Result<(), String>

#[tauri::command]
pub async fn broadlink_cancel_learn() -> Result<(), String>
```

Register in `src-tauri/src/lib.rs`:
```rust
.invoke_handler(tauri::generate_handler![
    // ... existing handlers
    broadlink_commands::broadlink_discover,
    broadlink_commands::broadlink_learn,
    broadlink_commands::broadlink_send,
    broadlink_commands::broadlink_cancel_learn,
])
```

---

## Phase 3: Extend Discovery Server API

### 3.1 Add RF/IR API Endpoints

**File:** `src-tauri/src/discovery_server.rs`

Add new routes:

```rust
// List all commands (with optional category filter)
GET  /api/v1/rfir/commands
GET  /api/v1/rfir/commands?category=projector

// Execute command by slug
POST /api/v1/rfir/commands/:slug/execute

// Get specific command
GET  /api/v1/rfir/commands/:slug

// List devices
GET  /api/v1/rfir/devices

// Device discovery (trigger scan)
POST /api/v1/rfir/devices/discover

// Learn new command
POST /api/v1/rfir/learn
Body: { "deviceId": "...", "type": "ir|rf", "name": "...", "category": "..." }

// Cancel learning
DELETE /api/v1/rfir/learn
```

### 3.2 WebSocket Messages

Add new message types:

```rust
pub enum WsMessage {
    // ... existing messages

    // RF/IR events
    RfIrCommandExecuted { slug: String, success: bool },
    RfIrLearnStarted { device_id: String, signal_type: String },
    RfIrLearnComplete { code: String },
    RfIrLearnFailed { error: String },
    RfIrDevicesUpdated { devices: Vec<BroadlinkDevice> },
}
```

### 3.3 Swagger/OpenAPI Documentation

**File:** `src-tauri/src/openapi.rs`

Generate OpenAPI 3.0 spec using `utoipa` crate:

```rust
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        health_check,
        get_status,
        get_obs_status,
        start_stream,
        stop_stream,
        start_record,
        stop_record,
        // RF/IR endpoints
        list_rfir_commands,
        execute_rfir_command,
        get_rfir_command,
        list_rfir_devices,
        discover_rfir_devices,
        start_rfir_learn,
        cancel_rfir_learn,
    ),
    components(schemas(
        SystemStatus,
        ObsStatus,
        BroadlinkDevice,
        RfIrCommand,
        LearnRequest,
        ExecuteResponse,
    )),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "obs", description = "OBS control endpoints"),
        (name = "rfir", description = "RF/IR remote control endpoints"),
    )
)]
pub struct ApiDoc;
```

**Add Swagger UI route:**

```rust
// Serve OpenAPI JSON
GET /api/v1/openapi.json -> Returns OpenAPI spec

// Serve Swagger UI
GET /api/docs -> Swagger UI HTML page
GET /api/docs/* -> Swagger UI static assets
```

Use `utoipa-swagger-ui` crate for embedded Swagger UI.

---

## Phase 4: Frontend Store & Services

### 4.1 RF/IR Settings Store

**File:** `src/lib/stores/rf-ir-store.ts`

```typescript
import { writable, derived } from 'svelte/store';
import { appSettingsStore } from '$lib/utils/app-settings-store';
import { generateSlug } from '$lib/utils/slug';
import type { RfIrSettings, RfIrCommand, BroadlinkDevice, LearnModeState } from '$lib/types/rf-ir';

// Main settings store
export const rfIrSettings = writable<RfIrSettings>(DEFAULT_RF_IR_SETTINGS);

// Learn mode state
export const learnModeState = writable<LearnModeState>({
  active: false,
  deviceId: null,
  type: null,
  learnedCode: null,
  error: null
});

// Derived stores
export const rfIrCommands = derived(rfIrSettings, $s => $s.commands);
export const rfIrDevices = derived(rfIrSettings, $s => $s.devices);
export const commandsByCategory = derived(rfIrCommands, $commands => {
  const grouped = new Map<string, RfIrCommand[]>();
  for (const cmd of $commands) {
    const category = cmd.category || 'uncategorized';
    if (!grouped.has(category)) grouped.set(category, []);
    grouped.get(category)!.push(cmd);
  }
  return grouped;
});

// Store operations
export const rfIrStore = {
  async load() {
    const settings = await appSettingsStore.get('rfIrSettings');
    if (settings) rfIrSettings.set(settings);
  },

  async save(settings: RfIrSettings) {
    rfIrSettings.set(settings);
    await appSettingsStore.set('rfIrSettings', settings);
  },

  // Device operations
  async addDevice(device: Omit<BroadlinkDevice, 'id' | 'lastSeen'>) {
    rfIrSettings.update(s => ({
      ...s,
      devices: [...s.devices, {
        ...device,
        id: crypto.randomUUID(),
        lastSeen: Date.now()
      }]
    }));
    await this.persist();
  },

  async updateDevice(id: string, updates: Partial<BroadlinkDevice>) {
    rfIrSettings.update(s => ({
      ...s,
      devices: s.devices.map(d => d.id === id ? { ...d, ...updates } : d)
    }));
    await this.persist();
  },

  async removeDevice(id: string) {
    rfIrSettings.update(s => ({
      ...s,
      devices: s.devices.filter(d => d.id !== id)
    }));
    await this.persist();
  },

  async setDefaultDevice(id: string) {
    rfIrSettings.update(s => ({
      ...s,
      devices: s.devices.map(d => ({ ...d, isDefault: d.id === id }))
    }));
    await this.persist();
  },

  // Command operations
  async addCommand(command: Omit<RfIrCommand, 'id' | 'slug' | 'createdAt' | 'updatedAt'>) {
    let settings: RfIrSettings;
    rfIrSettings.subscribe(s => settings = s)();

    const slug = generateSlug(command.name, settings.commands.map(c => c.slug));
    const now = Date.now();

    rfIrSettings.update(s => ({
      ...s,
      commands: [...s.commands, {
        ...command,
        id: crypto.randomUUID(),
        slug,
        createdAt: now,
        updatedAt: now
      }]
    }));
    await this.persist();
    return slug;
  },

  async updateCommand(id: string, updates: Partial<Omit<RfIrCommand, 'id' | 'slug'>>) {
    rfIrSettings.update(s => {
      const cmd = s.commands.find(c => c.id === id);
      if (!cmd) return s;

      // Regenerate slug if name changed
      let newSlug = cmd.slug;
      if (updates.name && updates.name !== cmd.name) {
        const otherSlugs = s.commands.filter(c => c.id !== id).map(c => c.slug);
        newSlug = generateSlug(updates.name, otherSlugs);
      }

      return {
        ...s,
        commands: s.commands.map(c => c.id === id
          ? { ...c, ...updates, slug: newSlug, updatedAt: Date.now() }
          : c
        )
      };
    });
    await this.persist();
  },

  async removeCommand(id: string) {
    rfIrSettings.update(s => ({
      ...s,
      commands: s.commands.filter(c => c.id !== id)
    }));
    await this.persist();
  },

  getCommandBySlug(slug: string): RfIrCommand | undefined {
    let cmd: RfIrCommand | undefined;
    rfIrSettings.subscribe(s => {
      cmd = s.commands.find(c => c.slug === slug);
    })();
    return cmd;
  },

  async persist() {
    let settings: RfIrSettings;
    rfIrSettings.subscribe(s => settings = s)();
    await appSettingsStore.set('rfIrSettings', settings);
  }
};
```

### 4.2 Broadlink Service

**File:** `src/lib/utils/broadlink-service.ts`

```typescript
import { invoke } from '@tauri-apps/api/core';
import { rfIrStore, learnModeState } from '$lib/stores/rf-ir-store';
import type { BroadlinkDevice, RfIrCommand } from '$lib/types/rf-ir';

export const broadlinkService = {
  async discoverDevices(timeout: number = 5): Promise<BroadlinkDevice[]> {
    const devices = await invoke<BroadlinkDevice[]>('broadlink_discover', { timeout });
    return devices;
  },

  async startLearning(deviceId: string, type: 'ir' | 'rf'): Promise<void> {
    learnModeState.set({
      active: true,
      deviceId,
      type,
      learnedCode: null,
      error: null
    });

    try {
      const code = await invoke<string>('broadlink_learn', {
        deviceId,
        signalType: type
      });
      learnModeState.update(s => ({ ...s, learnedCode: code, active: false }));
    } catch (error) {
      learnModeState.update(s => ({
        ...s,
        error: error instanceof Error ? error.message : String(error),
        active: false
      }));
    }
  },

  async cancelLearning(): Promise<void> {
    await invoke('broadlink_cancel_learn');
    learnModeState.set({
      active: false,
      deviceId: null,
      type: null,
      learnedCode: null,
      error: null
    });
  },

  async sendCommand(command: RfIrCommand): Promise<void> {
    await invoke('broadlink_send', {
      deviceId: command.deviceId,
      code: command.code
    });
  },

  async executeBySlug(slug: string): Promise<void> {
    const command = rfIrStore.getCommandBySlug(slug);
    if (!command) throw new Error(`Command not found: ${slug}`);
    await this.sendCommand(command);
  }
};
```

---

## Phase 5: Settings UI Components

### 5.1 RF/IR Settings Main Component

**File:** `src/lib/components/rf-ir-settings.svelte`

Structure:
```svelte
<Card>
  <CardHeader>
    <CardTitle>RF/IR Remote Control</CardTitle>
    <CardDescription>Configure Broadlink devices for IR/RF control</CardDescription>
  </CardHeader>
  <CardContent>
    <!-- Enable/Disable toggle -->

    <!-- Device Section -->
    <RfIrDeviceList />

    <!-- Commands Section -->
    <RfIrCommandList />

    <!-- Import from sermon-helper-service -->
    <RfIrImport />
  </CardContent>
</Card>
```

### 5.2 Device List Component

**File:** `src/lib/components/rf-ir-device-list.svelte`

Features:
- List discovered/configured devices
- "Discover Devices" button
- Add device manually (host, mac, type)
- Set default device
- Remove device
- Device status indicator (online/offline)

### 5.3 Command List Component

**File:** `src/lib/components/rf-ir-command-list.svelte`

Features:
- List commands grouped by category
- Run command button (play icon)
- Edit command (opens dialog)
- Delete command (with confirmation)
- Copy slug (for API usage)
- Category filter dropdown
- Search/filter by name

### 5.4 Learn Mode Component

**File:** `src/lib/components/rf-ir-learn-dialog.svelte`

Dialog flow:
1. Select device (or use default)
2. Select signal type (IR/RF)
3. Enter command name and category
4. Click "Start Learning"
5. Show "Waiting for signal..." with animation
6. On success: show preview, confirm save
7. On timeout/error: show error, retry option

### 5.5 Manual Code Entry Component

**File:** `src/lib/components/rf-ir-code-entry-dialog.svelte`

Features:
- Text input for hex code (paste from other sources)
- Device selector
- Signal type selector
- Name and category inputs
- Validate hex format before save

### 5.6 Import Component

**File:** `src/lib/components/rf-ir-import.svelte`

Features:
- Import from file (select .on/.off files from sermon-helper-service)
- Bulk import from directory
- Preview before import
- Conflict resolution (skip/replace/rename)

---

## Phase 6: Settings Page Integration

### 6.1 Update Settings Page

**File:** `src/routes/obs-config/+page.svelte`

Add RF/IR settings component to the right column:

```svelte
<div class="col-span-1 flex flex-col gap-4">
  <!-- Existing components -->
  <ObsDeviceConfigs />
  <ImportExportSettings />
  <UpdateSettings />

  <!-- New RF/IR component -->
  <RfIrSettings />
</div>
```

### 6.2 Add to AppSettings Type

**File:** `src/lib/utils/app-settings-store.ts`

```typescript
interface AppSettings {
  // ... existing fields
  rfIrSettings: RfIrSettings;
}

const defaultSettings: AppSettings = {
  // ... existing defaults
  rfIrSettings: DEFAULT_RF_IR_SETTINGS
};
```

### 6.3 Add to Import/Export

**File:** `src/lib/utils/import-export-service.ts`

Add `rfIrSettings` to the list of exportable keys.

---

## Phase 7: Command Execution UI

### 7.1 Create RF/IR Control Page

**File:** `src/routes/rf-ir/+page.svelte`

Dedicated page for running RF/IR commands:
- Category tabs/sidebar
- Command grid with large buttons
- Quick access to frequently used
- Feedback on execution (success/failure toast)

### 7.2 Add to Sidebar Navigation

**File:** `src/lib/components/sidebar.svelte`

Add navigation item for RF/IR control page:
```svelte
<NavItem href="/rf-ir" icon={Remote}>Remote Control</NavItem>
```

---

## Phase 8: Auto-Start Service

### 8.1 Update App Initialization

**File:** `src/routes/+layout.svelte` (or initialization code)

```typescript
onMount(async () => {
  // Load discovery settings
  const settings = await appSettingsStore.get('discoverySettings');

  // Auto-start if enabled and auth token exists
  if (settings?.autoStart && settings?.authToken) {
    try {
      await invoke('start_discovery_server', {
        port: settings.port,
        authToken: settings.authToken,
        instanceName: settings.instanceName
      });
      console.log('Discovery server auto-started');
    } catch (error) {
      console.error('Failed to auto-start discovery server:', error);
    }
  }

  // Load RF/IR settings
  await rfIrStore.load();
});
```

---

## Phase 9: API Documentation UI

### 9.1 Swagger UI Integration

Embed Swagger UI in the web interface:

**Route:** `/api/docs` (served by discovery server)

Uses `utoipa-swagger-ui` to serve:
- Interactive API documentation
- Try-it-out functionality
- Schema visualization
- Authentication support (Bearer token input)

### 9.2 API Docs Link in Settings

Add link to API documentation in Discovery Settings component:
```svelte
{#if serverRunning}
  <Button variant="outline" href={`http://localhost:${port}/api/docs`}>
    View API Documentation
  </Button>
{/if}
```

---

## Implementation Order

1. **Phase 1:** Type definitions and slug utility
2. **Phase 2:** Rust backend Broadlink integration
3. **Phase 4:** Frontend store and services
4. **Phase 5:** UI components (settings)
5. **Phase 6:** Settings page integration
6. **Phase 3:** API endpoints and WebSocket messages
7. **Phase 7:** Command execution UI page
8. **Phase 8:** Auto-start functionality
9. **Phase 9:** Swagger documentation

---

## File Summary

### New Files to Create

**Types:**
- `src/lib/types/rf-ir.ts`

**Utils:**
- `src/lib/utils/slug.ts`
- `src/lib/utils/broadlink-service.ts`

**Stores:**
- `src/lib/stores/rf-ir-store.ts`

**Components:**
- `src/lib/components/rf-ir-settings.svelte`
- `src/lib/components/rf-ir-device-list.svelte`
- `src/lib/components/rf-ir-command-list.svelte`
- `src/lib/components/rf-ir-learn-dialog.svelte`
- `src/lib/components/rf-ir-code-entry-dialog.svelte`
- `src/lib/components/rf-ir-import.svelte`

**Routes:**
- `src/routes/rf-ir/+page.svelte`

**Rust Backend:**
- `src-tauri/src/broadlink.rs`
- `src-tauri/src/broadlink_commands.rs`
- `src-tauri/src/openapi.rs`
- `src-tauri/scripts/broadlink_bridge.py`

### Files to Modify

- `src-tauri/Cargo.toml` - Add dependencies
- `src-tauri/src/lib.rs` - Register commands
- `src-tauri/src/discovery_server.rs` - Add RF/IR endpoints
- `src/lib/utils/app-settings-store.ts` - Add rfIrSettings
- `src/lib/utils/import-export-service.ts` - Add rfIrSettings export
- `src/routes/obs-config/+page.svelte` - Add RF/IR settings component
- `src/lib/components/sidebar.svelte` - Add navigation item
- `src/routes/+layout.svelte` - Add auto-start logic

---

## API Endpoint Summary

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| GET | `/api/v1/rfir/commands` | List all commands | Yes |
| GET | `/api/v1/rfir/commands/:slug` | Get command by slug | Yes |
| POST | `/api/v1/rfir/commands/:slug/execute` | Execute command | Yes |
| GET | `/api/v1/rfir/devices` | List devices | Yes |
| POST | `/api/v1/rfir/devices/discover` | Discover devices | Yes |
| POST | `/api/v1/rfir/learn` | Start learning mode | Yes |
| DELETE | `/api/v1/rfir/learn` | Cancel learning | Yes |
| GET | `/api/v1/openapi.json` | OpenAPI spec | No |
| GET | `/api/docs` | Swagger UI | No |

---

## Default Commands to Import

From `sermon-helper-service/src/broadlink-cli/`:
- `NEC.on` → "Projector Power On" (category: projector)
- `NEC.off` → "Projector Power Off" (category: projector)
- `ROLL.up` → "Screen Up" (category: screen)
- `ROLL.down` → "Screen Down" (category: screen)
- `ROLL.stop` → "Screen Stop" (category: screen)
- `CASCADE.temp_up` → "AC Temp Up" (category: hvac)
- `CASCADE.temp_down` → "AC Temp Down" (category: hvac)
