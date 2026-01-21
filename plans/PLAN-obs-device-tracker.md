# OBS Device & Source Configuration Feature Plan

## Overview

This feature adds comprehensive OBS device and source management with:
- Automatic detection of displays and audio devices from OBS
- Configurable required devices with auto-assignment to OBS sources
- Browser source URL templates with event variable interpolation
- 5-minute polling integration for continuous validation
- Dynamic sidebar system status items
- Auto-refresh and manual refresh capabilities for browser sources

---

## Data Structures

### 1. New Types (`src/lib/types/obs-devices.ts`)

```typescript
// Display or audio device from OBS
export interface ObsDevice {
  itemName: string;      // Display name from OBS
  itemValue: string;     // Unique identifier (display_uuid or device_id)
}

// Configured device/source mapping
export interface ObsDeviceConfig {
  id: string;                    // Unique config ID
  type: 'display' | 'audio';     // Device type
  name: string;                  // User-friendly name (for sidebar)
  required: boolean;             // Show error if not available
  targetSourceName: string;      // OBS source to assign to
  expectedValue: string;         // Expected device_id or display_uuid
  propertyName: string;          // 'device_id' for audio, 'display_uuid' for display
}

// Browser source configuration
export interface ObsBrowserSourceConfig {
  id: string;
  name: string;                  // User-friendly name
  required: boolean;             // Currently unused (no system status for browser)
  targetSourceName: string;      // OBS browser source name
  urlTemplate: string;           // URL with ${textus} and ${lekcio} placeholders
}

// Current status of a configured device
export interface ObsDeviceStatus {
  configId: string;
  available: boolean;            // Device exists in OBS property list
  assigned: boolean;             // OBS source has correct setting
  lastChecked: number;           // Timestamp
  error?: string;
}

// Current status of a browser source
export interface ObsBrowserSourceStatus {
  configId: string;
  currentUrl: string;            // Current URL in OBS
  expectedUrl: string;           // Expected URL based on event
  matches: boolean;              // URLs match
  lastChecked: number;
  refreshPending: boolean;       // Waiting for refresh confirmation
  refreshSuccess?: boolean;      // Last refresh result
}

// All OBS device configurations (stored in app settings)
export interface ObsDevicesSettings {
  devices: ObsDeviceConfig[];
  browserSources: ObsBrowserSourceConfig[];
}
```

---

## Implementation Steps

### Phase 1: Core Infrastructure

#### 1.1 Create OBS Devices Store (`src/lib/stores/obs-devices-store.ts`)

- Add `obsDevicesSettings` to `AppSettings` interface
- Create derived stores for device and browser source configs
- Implement CRUD operations: `addDevice`, `updateDevice`, `removeDevice`
- Same for browser sources: `addBrowserSource`, `updateBrowserSource`, `removeBrowserSource`

#### 1.2 Extend OBS WebSocket Class (`src/lib/utils/obs-websocket.ts`)

Add new methods to `LocalOBSWebSocket`:

```typescript
// Get available devices for a source
async getInputPropertyItems(
  inputName: string,
  propertyName: string
): Promise<ObsDevice[]>

// Get current input settings
async getInputSettings(inputName: string): Promise<Record<string, unknown>>

// Set input settings (auto-assign)
async setInputSettings(
  inputName: string,
  settings: Record<string, unknown>
): Promise<void>

// Refresh browser source
async refreshBrowserSource(inputName: string): Promise<void>

// Get all inputs of a specific kind
async getInputList(inputKind?: string): Promise<string[]>
```

#### 1.3 Create OBS Device Status Store (`src/lib/stores/obs-device-status-store.ts`)

Runtime status (not persisted):

```typescript
export const obsDeviceStatuses = writable<Map<string, ObsDeviceStatus>>(new Map());
export const obsBrowserStatuses = writable<Map<string, ObsBrowserSourceStatus>>(new Map());

// Derived: all required devices that are failing
export const failingRequiredDevices = derived(
  [obsDeviceStatuses, obsDevicesSettings],
  ([$statuses, $settings]) => {
    return $settings.devices
      .filter(d => d.required && !$statuses.get(d.id)?.available);
  }
);
```

---

### Phase 2: OBS Polling Integration

#### 2.1 Create OBS Device Checker (`src/lib/utils/obs-device-checker.ts`)

Main validation logic:

```typescript
export async function checkAllObsDevices(): Promise<void> {
  const settings = get(obsDevicesSettings);

  for (const device of settings.devices) {
    await checkDevice(device);
  }

  for (const browser of settings.browserSources) {
    await checkBrowserSource(browser);
  }
}

async function checkDevice(config: ObsDeviceConfig): Promise<ObsDeviceStatus> {
  // 1. GetInputSettings to get current value
  // 2. GetInputPropertiesListPropertyItems to get available options
  // 3. Check if expectedValue exists in options
  // 4. If available but not assigned, call SetInputSettings
  // 5. Update obsDeviceStatuses store
}

async function checkBrowserSource(config: ObsBrowserSourceConfig): Promise<void> {
  // 1. GetInputSettings to get current URL
  // 2. Get upcoming/today event's textus and lekcio
  // 3. Interpolate URL template
  // 4. Compare URLs
  // 5. If mismatch, attempt SetInputSettings with new URL
  // 6. Re-fetch and validate
  // 7. Update obsBrowserStatuses store
}
```

#### 2.2 Integrate with Refresh Store (`src/lib/stores/refresh-store.ts`)

Modify `sync()` to also run OBS device checks:

```typescript
async sync() {
  // Existing YouTube sync...

  // Add OBS device checking
  if ($systemStore.obs) {
    await checkAllObsDevices();
  }
}
```

---

### Phase 3: System Status Integration

#### 3.1 Extend SystemStatus Type (`src/lib/stores/types.ts`)

```typescript
export type SystemStatus = {
  // ... existing fields

  // Dynamic OBS device statuses (keyed by config ID)
  obsDevices: Record<string, boolean>;
}
```

#### 3.2 Update System Store (`src/lib/stores/system-store.ts`)

Derive OBS device statuses into system store:

```typescript
export const systemStore = derived(
  [nonObsSystemStore, obsWebSocket.obsStatus, failingRequiredDevices],
  ([$nonObs, $obsStatus, $failing]) => ({
    // ... existing
    obsDevices: Object.fromEntries(
      // Map device configs to their availability status
    )
  })
);
```

#### 3.3 Update Sidebar (`src/lib/components/sidebar.svelte`)

Add dynamic status rows for required devices:

```svelte
{#each $requiredDeviceConfigs as device (device.id)}
  <div class="flex items-center justify-between py-2">
    <span class="text-sm text-muted-foreground">{device.name}</span>
    {#if $obsDeviceStatuses.get(device.id)?.available}
      <CheckCircle2 class="h-4 w-4 text-green-600" />
    {:else}
      <XCircle class="h-4 w-4 text-red-600" />
    {/if}
  </div>
{/each}
```

#### 3.4 Update Error Messages (`src/lib/components/ui/error-messages.svelte`)

Generate dynamic error messages for failing required devices:

```typescript
$: deviceErrors = $failingRequiredDevices.map(device => ({
  id: `obs-device-${device.id}`,
  title: `${device.name} Not Found`,
  description: `Required ${device.type} device is not available in OBS.`,
  // ... error details
}));
```

---

### Phase 4: Configuration UI

#### 4.1 Create OBS Devices Settings Page (`src/routes/obs-devices/+page.svelte`)

Main configuration page with sections:

**Displays Section:**
- List of configured display devices
- Add new display config button
- For each: name input, required checkbox, OBS source select, device value select

**Audio Devices Section:**
- Same pattern as displays

**Browser Sources Section:**
- List of configured browser sources
- Add new browser source button
- For each: name input, required checkbox (disabled/info only), OBS source select, URL template textarea

#### 4.2 OBS Device Config Form Component (`src/lib/components/obs-device-config-form.svelte`)

Reusable form for device configuration:

```svelte
<script>
  export let config: ObsDeviceConfig;
  export let availableDevices: ObsDevice[];
  export let availableSources: string[];
</script>

<div class="space-y-4">
  <div>
    <Label>Name (shown in sidebar)</Label>
    <Input bind:value={config.name} />
  </div>

  <div class="flex items-center gap-2">
    <Checkbox bind:checked={config.required} />
    <Label>Required (show error if not available)</Label>
  </div>

  <div>
    <Label>OBS Source</Label>
    <Select bind:value={config.targetSourceName}>
      {#each availableSources as source}
        <option value={source}>{source}</option>
      {/each}
    </Select>
  </div>

  <div>
    <Label>Device</Label>
    <Select bind:value={config.expectedValue}>
      {#each availableDevices as device}
        <option value={device.itemValue}>{device.itemName}</option>
      {/each}
    </Select>
  </div>
</div>
```

#### 4.3 Browser Source Config Form (`src/lib/components/obs-browser-source-form.svelte`)

```svelte
<div class="space-y-4">
  <div>
    <Label>Name</Label>
    <Input bind:value={config.name} />
  </div>

  <div>
    <Label>OBS Browser Source</Label>
    <Select bind:value={config.targetSourceName}>
      {#each browserSources as source}
        <option value={source}>{source}</option>
      {/each}
    </Select>
  </div>

  <div>
    <Label>URL Template</Label>
    <Textarea
      bind:value={config.urlTemplate}
      placeholder="http://example.com/${textus}?lekcio=${lekcio}"
    />
    <p class="text-sm text-muted-foreground mt-1">
      Available variables: ${textus}, ${lekcio}
    </p>
  </div>
</div>
```

---

### Phase 5: Browser Source Refresh UI

#### 5.1 Update Event Display (Events Page / Sidebar)

Show browser source status for upcoming event:

```svelte
{#if $todayEvent && $obsBrowserStatuses.size > 0}
  <div class="space-y-2">
    <h4>OBS Browser Sources</h4>
    {#each $browserSourceConfigs as config (config.id)}
      {@const status = $obsBrowserStatuses.get(config.id)}
      <div class="flex items-center justify-between">
        <span>{config.name}</span>
        {#if status?.matches}
          <CheckCircle2 class="h-4 w-4 text-green-600" />
        {:else if status?.refreshPending}
          <Loader2 class="h-4 w-4 animate-spin" />
        {:else}
          <button onclick={() => refreshBrowserSource(config.id)}>
            <RefreshCw class="h-4 w-4 text-amber-600" />
          </button>
        {/if}
      </div>
    {/each}
  </div>
{/if}
```

#### 5.2 Refresh Logic

```typescript
async function refreshBrowserSource(configId: string): Promise<void> {
  const config = $browserSourceConfigs.find(c => c.id === configId);
  if (!config) return;

  // 1. Set refreshPending = true
  // 2. Calculate expected URL from today's event
  // 3. Call SetInputSettings with new URL
  // 4. Wait brief moment
  // 5. Re-fetch URL from OBS with GetInputSettings
  // 6. Compare and set refreshSuccess
  // 7. Set refreshPending = false
  // 8. If success, show checkmark; if fail, keep refresh icon
}
```

---

### Phase 6: Localization

#### 6.1 Add Translation Keys (`src/lib/locales/en.json` and `hu.json`)

```json
{
  "obsDevices": {
    "title": "OBS Devices",
    "displays": "Displays",
    "audioDevices": "Audio Devices",
    "browserSources": "Browser Sources",
    "addDisplay": "Add Display",
    "addAudio": "Add Audio Device",
    "addBrowser": "Add Browser Source",
    "name": "Name",
    "required": "Required",
    "requiredHint": "Show error in sidebar if not available",
    "obsSource": "OBS Source",
    "device": "Device",
    "urlTemplate": "URL Template",
    "urlTemplateHint": "Available variables: ${textus}, ${lekcio}",
    "save": "Save",
    "delete": "Delete",
    "noDevicesConfigured": "No devices configured"
  },
  "sidebar": {
    "systemStatus": {
      "browserSourceSync": "Browser Source"
    }
  },
  "errors": {
    "obsDevice": {
      "title": "{name} Not Found",
      "description": "Required {type} device is not available in OBS."
    }
  }
}
```

---

## File Changes Summary

### New Files
1. `src/lib/types/obs-devices.ts` - Type definitions
2. `src/lib/stores/obs-devices-store.ts` - Configuration storage
3. `src/lib/stores/obs-device-status-store.ts` - Runtime status
4. `src/lib/utils/obs-device-checker.ts` - Validation logic
5. `src/routes/obs-devices/+page.svelte` - Settings page
6. `src/lib/components/obs-device-config-form.svelte` - Device form
7. `src/lib/components/obs-browser-source-form.svelte` - Browser form

### Modified Files
1. `src/lib/utils/obs-websocket.ts` - Add OBS API methods
2. `src/lib/utils/app-settings-store.ts` - Add obsDevicesSettings to AppSettings
3. `src/lib/stores/refresh-store.ts` - Add OBS checking to sync()
4. `src/lib/stores/system-store.ts` - Add dynamic device statuses
5. `src/lib/stores/types.ts` - Extend SystemStatus type
6. `src/lib/components/sidebar.svelte` - Add dynamic status rows
7. `src/lib/components/ui/error-messages.svelte` - Add dynamic device errors
8. `src/routes/events/+page.svelte` - Add browser source status display
9. `src/lib/locales/en.json` - Add translations
10. `src/lib/locales/hu.json` - Add Hungarian translations

---

## Flow Diagrams

### Device Check Flow (Every 5 Minutes)
```
refreshStore.sync()
    │
    ├─► YouTube sync (existing)
    │
    └─► checkAllObsDevices()
            │
            ├─► For each device config:
            │       │
            │       ├─► GetInputSettings(sourceName)
            │       ├─► GetInputPropertiesListPropertyItems(sourceName, propertyName)
            │       │
            │       ├─► Device in list? ─► No ─► Mark unavailable, show error
            │       │         │
            │       │         Yes
            │       │         │
            │       │         └─► Assigned correctly? ─► No ─► SetInputSettings()
            │       │                    │
            │       │                   Yes
            │       │                    │
            │       └─────────────────► Mark available ✓
            │
            └─► For each browser source config:
                    │
                    ├─► GetInputSettings(sourceName) → current URL
                    ├─► Get todayEvent → textus, lekcio
                    ├─► Interpolate urlTemplate
                    │
                    └─► URLs match? ─► No ─► SetInputSettings(new URL)
                             │                    │
                            Yes                   └─► Re-fetch & validate
                             │                             │
                             └─────────────────────────► Update status
```

### Browser Source Refresh Flow (Manual)
```
User clicks refresh icon
    │
    └─► refreshBrowserSource(configId)
            │
            ├─► Set refreshPending = true
            ├─► Calculate expected URL from todayEvent
            ├─► SetInputSettings({ url: expectedUrl })
            ├─► Wait 500ms
            ├─► GetInputSettings() → actualUrl
            │
            └─► actualUrl === expectedUrl?
                    │
                    ├─► Yes: Show checkmark ✓
                    │
                    └─► No: Keep refresh icon (user can retry)
```

---

## Edge Cases & Error Handling

1. **OBS Not Connected**: Skip all device checks, show OBS connection error only
2. **Source Not Found**: Log warning, mark device as unavailable
3. **Device Disconnected**: Mark unavailable, show error if required
4. **Network URL Issues**: Catch fetch errors, keep refresh icon visible
5. **No Upcoming Event**: Skip browser source URL comparison (no textus/lekcio)
6. **Empty URL Template**: Skip that browser source check
7. **Concurrent Refreshes**: Use refreshPending flag to prevent double-refresh

---

## Testing Checklist

- [ ] Add display device config
- [ ] Add audio device config
- [ ] Add browser source config with URL template
- [ ] Verify 5-minute polling triggers device checks
- [ ] Verify sidebar shows required device status
- [ ] Verify error banner shows for missing required device
- [ ] Verify auto-assignment works when device available but not assigned
- [ ] Verify browser source URL comparison with event data
- [ ] Verify manual refresh icon appears on URL mismatch
- [ ] Verify checkmark appears after successful refresh
- [ ] Verify re-fetch validates the actual OBS state
