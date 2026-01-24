# Plan: Video Uploader Redesign

## Current State

### Components
1. **`event-session-status.svelte`** - Shows current session state (IDLE, PREPARING, ACTIVE, FINALIZING, etc.)
   - Tied to EventSession state machine
   - Shows upload progress only when FINALIZING
   - Mixed concerns: session tracking + upload status

2. **`pending-uploads-list.svelte`** - Shows pending uploads from events
   - Lists uploads with status: pending, paused, failed
   - Resume/Cancel actions
   - Shows event date

3. **`upload-status-section.svelte`** - Container that shows both above components

### Data Sources
- `EventSession.uploadProgress[]` - Live upload progress during session
- `ServiceEvent.uploadSessions[]` - Persisted upload sessions per event
- `allPendingUploads` - Derived store of all pending uploads across events

## Requested Changes

1. **Rename**: "Session" → Keep for OBS tracking, new "Video Uploader" section
2. **Position**: Video Uploader below current session
3. **List format**: Show ALL uploads (active + pending), not just current session
4. **Title**: Use `generateCalculatedTitle(event)` instead of event.title

## Proposed Design

### New Architecture

```
┌─ Sidebar ─────────────────────────────────────┐
│                                               │
│ [System Status Card]                          │
│                                               │
│ [Session Card] ← OBS streaming/recording only │
│  • State: ACTIVE / PREPARING / etc.           │
│  • Pause reason if paused                     │
│  • Manual trigger button                      │
│                                               │
│ [Video Uploader Card] ← NEW unified component │
│  • List of ALL uploads (active + pending)     │
│  • Each item shows:                           │
│    - Calculated title (YouTube format)        │
│    - Platform icon (YouTube)                  │
│    - Status badge (uploading/paused/failed)   │
│    - Progress bar (if uploading)              │
│    - Resume/Cancel buttons                    │
│    - Link to event page                       │
│  • YouTube not connected warning              │
│  • Resume All button (if multiple)            │
│                                               │
│ [Upcoming Event Card]                         │
│                                               │
└───────────────────────────────────────────────┘
```

### Data Model Changes

Need to unify two data sources:
1. **Active uploads** from `EventSession.uploadProgress[]` (live during session)
2. **Stored uploads** from `ServiceEvent.uploadSessions[]` (persisted)

Create a unified view:
```typescript
interface UnifiedUploadItem {
  id: string;
  eventId: string;
  event: ServiceEvent;
  calculatedTitle: string;      // generateCalculatedTitle(event)
  platform: UploadPlatform;
  status: 'uploading' | 'pending' | 'paused' | 'failed' | 'completed';
  progress?: {                  // Only for active uploads
    bytesUploaded: number;
    totalBytes: number;
    percentage: number;
  };
  error?: string;
  startedAt: number;
  source: 'session' | 'event';  // Where the data comes from
}
```

## Implementation Tasks

### Task 1: Simplify `event-session-status.svelte`
- Remove upload progress display (move to new component)
- Keep only OBS session state tracking
- Keep pause/resume, manual trigger functionality
- Rename i18n keys if needed

### Task 2: Create `video-uploader.svelte`
- New unified component replacing `pending-uploads-list.svelte`
- Merge active uploads (from session) + pending uploads (from events)
- Show `generateCalculatedTitle(event)` for each
- Platform icon, status badge, progress bar
- Resume/Cancel actions per item
- Link to event page
- YouTube not connected warning
- Resume All button

### Task 3: Create unified uploads store
- New derived store that combines:
  - `currentSession.uploadProgress[]` (active)
  - `allPendingUploads` (stored in events)
- Deduplicate if same upload appears in both
- Sort by status (uploading first) then by startedAt

### Task 4: Update `upload-status-section.svelte`
- Rename section title to "Video Uploader"
- Use new `video-uploader.svelte` component
- Remove `pending-uploads-list.svelte` usage

### Task 5: Update i18n
- Add new keys for "Video Uploader" section
- Update existing keys as needed

### Task 6: Cleanup
- Remove or deprecate `pending-uploads-list.svelte`
- Update sidebar layout if needed

## File Changes Summary

| File | Action |
|------|--------|
| `src/lib/components/event-session-status.svelte` | Simplify - remove upload progress |
| `src/lib/components/video-uploader.svelte` | **CREATE** - unified upload list |
| `src/lib/components/pending-uploads-list.svelte` | DELETE or deprecate |
| `src/lib/components/upload-status-section.svelte` | Update to use new component |
| `src/lib/stores/unified-uploads-store.ts` | **CREATE** - combined data source |
| `src/lib/locales/en.json` | Add new i18n keys |
| `src/lib/locales/hu.json` | Add new i18n keys |

## UI Mockup

```
┌─────────────────────────────────────────────────┐
│ VIDEO UPLOADER                                  │
├─────────────────────────────────────────────────┤
│ ⚠️ YouTube Not Connected                        │
│ Connect to YouTube to upload recordings.        │
├─────────────────────────────────────────────────┤
│ 🔴 2026.01.26. Istentisztelet | Textus: Jn 3:16│
│    [Uploading 45%] ████████░░░░░░░░░  ▶️  ✕    │
├─────────────────────────────────────────────────┤
│ ⏸️ 2026.01.19. Istentisztelet | Speaker Name   │
│    [Paused] Token expired              ▶️  ✕    │
├─────────────────────────────────────────────────┤
│ ❌ 2026.01.12. Istentisztelet                   │
│    [Failed] Network error              ▶️  ✕    │
├─────────────────────────────────────────────────┤
│ [↻ Resume All]                                  │
└─────────────────────────────────────────────────┘

Legend:
- 🔴 = Currently uploading (animated)
- ⏸️ = Paused
- ❌ = Failed
- ▶️ = Resume button
- ✕ = Cancel button
```

## Questions/Decisions

1. **Should completed uploads show in the list?**
   - Recommendation: No, only show active/pending/failed
   - Completed uploads can be seen on the event page

2. **How long to keep failed uploads visible?**
   - Recommendation: Until manually cancelled or successfully resumed

3. **Should clicking the title go to event page or expand details?**
   - Recommendation: Go to event page (external link icon on hover)
