# PLAN-APS-Integration.md

## Overview
Replace the current direct PowerPoint file opening mechanism with Auto Presentation Switcher (APS) integration for professional presentation control through Companion.

## Current State Analysis
- **Current System**: Direct file opening using `open::that()` in Rust discovery server
- **File Types**: PowerPoint files (`.ppt`, `.pptx`, `.odp`)
- **Control**: Basic open + F5 key for presenter mode (Windows only)
- **Limitations**: No professional slide control, limited platform support

## Target State
- **New System**: APS API integration for full presentation control
- **Benefits**: Professional slide navigation, media control, cross-platform support
- **Control**: Slide-by-slide navigation, media playback, presentation switching

## Implementation Plan

### Phase 1: APS API Integration Foundation

#### 1.1 Add APS API Client
- **Location**: `src/lib/utils/aps-api-client.ts`
- **Purpose**: TCP client for APS API v2 communication
- **Features**: 
  - Length-prefixed JSON messaging
  - Connection management
  - Command sending
  - Feedback handling

#### 1.2 Update Discovery Server
- **Location**: `src-tauri/src/discovery_server.rs`
- **Changes**:
  - Remove `open::that()` PowerPoint opening
  - Add APS TCP client integration
  - Implement APS command forwarding
  - Add APS connection status endpoint

#### 1.3 Configuration Management
- **Location**: `src/lib/components/aps-settings.svelte`
- **Features**:
  - APS server host/port configuration
  - Connection status display
  - Test connection functionality
  - Settings persistence via Tauri store

### Phase 2: Core Presentation Control

#### 2.1 Update PPT Open Endpoint
- **Endpoint**: `POST /api/v1/ppt/open`
- **New Behavior**: Send `OpenStart_Presentation` command to APS
- **Parameters**:
  ```json
  {
    "file_path": "path/to/presentation.pptx",
    "slideNr": 1,
    "isFullscreen": true
  }
  ```

#### 2.2 Add Slide Navigation Endpoints
- **Endpoint**: `POST /api/v1/ppt/next`
- **Endpoint**: `POST /api/v1/ppt/previous`
- **Endpoint**: `POST /api/v1/ppt/goto-slide`
- **APS Commands**: `PowerPoint_Next`, `PowerPoint_Previous`, `PowerPoint_Go`

#### 2.3 Update Companion Module
- **Location**: `packages/companion-module-sermon-helper/`
- **New Actions**:
  - Next/Previous slide
  - Go to specific slide
  - Presentation info feedback
  - Connection status feedback

### Phase 3: Advanced Features

#### 3.1 Media Control Integration
- **PowerPoint Media**: Play/pause/stop embedded videos
- **Endpoints**: Media control via APS API
- **Feedback**: Media position and state

#### 3.2 Presentation Slot Management
- **Slot System**: Use APS presentation slots (1-20)
- **Folder Watching**: APS folder monitoring instead of custom scanning
- **File Mapping**: Map Companion buttons to APS slots

#### 3.3 Enhanced Feedback
- **Real-time Status**: Slide number, total slides, build count
- **Media Feedback**: Video duration, current position, playback state
- **Connection Status**: APS connection health monitoring

### Phase 4: UI Updates

#### 4.1 Update Sidebar Component
- **Location**: `src/lib/components/sidebar.svelte`
- **Changes**:
  - Add APS connection status indicator
  - Update PPT status with slide information
  - Add media control status for PowerPoint videos

#### 4.2 Update PPT Folder Settings
- **Location**: `src/lib/components/ppt-folder-settings.svelte`
- **Changes**:
  - Replace custom folder scanning with APS folder integration
  - Add APS slot management UI
  - Update file filtering to work with APS

#### 4.3 Add APS Settings Panel
- **New Component**: APS server configuration
- **Features**: Connection settings, status display, test functionality

## Technical Implementation Details

### APS API Client (TypeScript)
```typescript
// src/lib/utils/aps-api-client.ts
export class APSAPIClient {
  private socket: net.Socket | null = null;
  private host: string;
  private port: number;
  
  async connect(host: string, port: number = 31600): Promise<void>
  async sendCommand(command: APSCommand): Promise<void>
  async openPresentation(filePath: string, slideNr: number = 1, isFullscreen: boolean = true): Promise<void>
  async nextSlide(): Promise<void>
  async previousSlide(): Promise<void>
  async goToSlide(slideNr: number): Promise<void>
  onFeedback(callback: (feedback: APSFeedback) => void): void
}
```

### Rust Integration
```rust
// src-tauri/src/discovery_server.rs
// Replace current PPT opening with APS client calls
pub async fn open_presentation_aps(
    file_path: String,
    start_presenter: bool,
    slide_number: Option<u32>
) -> Result<(), Box<dyn std::error::Error>>
```

### Companion Module Updates
```typescript
// packages/companion-module-sermon-helper/src/instance.ts
// Add new actions for slide control and feedback
async function handleSlideNavigation(direction: 'next' | 'previous'): Promise<void>
async function handleGoToSlide(slideNumber: number): Promise<void>
```

## API Endpoints

### Updated Endpoints
- `POST /api/v1/ppt/open` - Open via APS
- `POST /api/v1/ppt/next` - Next slide
- `POST /api/v1/ppt/previous` - Previous slide  
- `POST /api/v1/ppt/goto-slide` - Go to specific slide
- `GET /api/v1/aps/status` - APS connection status
- `GET /api/v1/aps/presentation-info` - Current presentation info

### New WebSocket Events
- `aps_connection_status` - Connection state changes
- `slide_changed` - Slide navigation updates
- `media_state_changed` - PowerPoint media updates

## Configuration Schema

### APS Settings
```typescript
interface APSSettings {
  host: string;
  port: number;
  autoConnect: boolean;
  timeout: number;
}
```

### Storage Updates
- Add APS settings to Tauri store
- Migrate existing PPT folder settings to APS format
- Maintain backward compatibility during transition

## Testing Strategy

### Unit Tests
- APS API client functionality
- Command serialization/deserialization
- Connection management

### Integration Tests  
- APS server communication
- Companion module actions
- End-to-end presentation control

### Manual Testing
- PowerPoint presentation control
- Media playback functionality
- Connection failure handling
- Cross-platform compatibility

## Migration Plan

### Step 1: Parallel Implementation
- Keep existing PPT opening as fallback
- Add APS integration alongside current system
- Test APS functionality thoroughly

### Step 2: Gradual Migration
- Update UI to use APS as primary option
- Add configuration option to choose between systems
- Migrate existing PPT folders to APS slots

### Step 3: Complete Replacement
- Remove old direct PowerPoint opening code
- Make APS the only presentation control method
- Clean up unused code and dependencies

## Benefits of APS Integration

### Enhanced Control
- Professional slide navigation
- PowerPoint media control
- Presentation switching
- Real-time feedback

### Cross-Platform Support
- Windows and macOS compatibility
- Consistent behavior across platforms
- No platform-specific workarounds

### Professional Features
- Media playback control
- Build progression tracking
- Presentation slot management
- Folder watching capabilities

### Future Extensibility
- Support for PDF presentations via APS
- Keynote integration on macOS
- Web browser presentation control
- Media player integration

## Risks and Mitigations

### Risk: APS Dependency
- **Mitigation**: Keep fallback mechanism during transition
- **Monitoring**: APS connection status in UI

### Risk: Network Latency
- **Mitigation**: Local APS installation recommended
- **Optimization**: Connection pooling and command queuing

### Risk: Compatibility Issues
- **Mitigation**: Thorough testing with different PowerPoint versions
- **Fallback**: Graceful degradation to basic file opening

## Success Criteria

1. ✅ APS successfully opens and controls PowerPoint presentations
2. ✅ Slide navigation works via Companion buttons
3. ✅ Real-time feedback displays in UI
4. ✅ Media control functions for embedded videos
5. ✅ Cross-platform compatibility maintained
6. ✅ Seamless migration from current system
7. ✅ Professional presentation control achieved

## Timeline Estimate

- **Phase 1**: 2-3 days (Foundation + Configuration)
- **Phase 2**: 2-3 days (Core Control + Companion Updates)  
- **Phase 3**: 2 days (Advanced Features + Feedback)
- **Phase 4**: 1-2 days (UI Updates + Polish)
- **Testing & Migration**: 2 days

**Total Estimated Time**: 9-12 days

---

*This plan provides a comprehensive roadmap for replacing the current PowerPoint handling with professional APS integration, enabling advanced presentation control through Companion.*