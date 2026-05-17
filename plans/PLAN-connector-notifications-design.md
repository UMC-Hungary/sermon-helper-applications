# Design Prompt — Connector System Notifications

**Context:** Church livestream control desktop app (Tauri + SvelteKit). Operators run it during services.
Connectors bridge the app to external hardware and online platforms. When something breaks mid-service,
the operator needs to notice immediately, understand what broke, and recover — without leaving the screen.

---

## The connectors

| ID | Display name | Type | Purpose |
|----|-------------|------|---------|
| `obs` | OBS Studio | Local software | Video capture, streaming, recording |
| `atem` | Blackmagic ATEM | Hardware switcher | Multi-camera switching (network UDP) |
| `youtube` | YouTube | OAuth platform | Live broadcast, RTMP destination |
| `discord` | Discord | Webhook platform | Automated service announcements |
| `device:{id}` | *(device label)* | OBS source listener | Monitors a physical audio/video device inside OBS |

---

## Status model — 4 states, same for every connector

```
disconnected  →  connecting  →  connected
                     ↓               ↓
                   error  ←────── (runtime fault)
```

| State | Colour | Dot | Meaning |
|-------|--------|-----|---------|
| `connected` | Green | Solid | Fully operational |
| `connecting` | Amber | Pulsing | Attempting handshake |
| `disconnected` | Grey | Dim | Not enabled or cleanly stopped |
| `error` | Red | Solid | Connection failed or lost |

State changes arrive as real-time WebSocket messages. There is no polling.

---

## Where the status is shown today

1. **Navigation bar badges** — compact pill per enabled connector, always visible:
   `● OBS  Connected`  /  `◐ ATEM  Connecting`  /  `✕ YouTube  Error`

2. **Dashboard widgets** — same badge, larger; OBS additionally shows
   `Streaming` / `Not Streaming` and `Recording` / `Not Recording` flags.

3. **Errors page (`/errors`)** — one card per active error:
   - Connector name + generic message `"{name} connection error"`
   - Action buttons: **Recheck**, **Fix** (opens settings), **Info** (expands step-by-step guide)
   - No runtime error detail from the server — only the static troubleshooting markdown is shown

4. **Settings page** — inline per-connector error text for save/auth failures (OBS, YouTube, Facebook).

5. **Re-login modal** — appears automatically when YouTube or Facebook reaches `error`; prompts
   re-authentication without leaving the current page.

---

## What each connector error means to the operator

### OBS Studio
- **Common causes:** WebSocket server disabled, wrong port/password, OBS crashed
- **Recovery:** Re-enable WebSocket in OBS → Tools → WebSocket Server Settings; verify port (4455) and password
- **Severity:** Critical — no OBS = no recording, no streaming

### Blackmagic ATEM
- **Common causes:** Network cable unplugged, wrong IP, switcher powered off
- **Recovery:** Check Ethernet, verify IP address in ATEM Software Control
- **Severity:** High — losing multi-cam switching during a service
- **Note:** Integration is not yet fully implemented; settings can be saved for future use

### YouTube
- **Common causes:** OAuth token expired, API quota exhausted, account not live-streaming-enabled
- **Recovery:** Re-authenticate via the **Re-login** modal (triggered automatically on error)
- **Severity:** Medium — affects upload destination and live stream URL fetch
- **Special UX:** Shows a blocking re-login modal overlay instead of just an error card

### Discord
- **Common causes:** Webhook URL deleted or revoked, wrong URL pasted, channel permissions changed
- **Recovery:** Recreate webhook in Discord → Server Settings → Integrations → Webhooks
- **Severity:** Low — affects automated announcements only; no live content impact
- **Note:** Integration is not yet fully implemented

### OBS Device Listeners ("missing system devices")
- **What they are:** Named listeners that watch a specific audio or video source inside OBS
  (e.g. "Rode Interface", "Stage Camera"). Each has a user-assigned friendly name.
- **Common causes:** USB device unplugged, Bluetooth device disconnected, OBS source renamed/deleted
- **Error signal:** Server sends availability status per listener; frontend generates the message
  `"Device unavailable"` locally — no description of *why* it is unavailable
- **Recovery:** Replug device, rename source in OBS to match the listener, or delete the listener
- **Severity:** Medium — operator may not notice a missing audio source until it is too late

---

## Notification delivery channels (current implementation)

| Channel | Trigger | Content |
|---------|---------|---------|
| Nav badge (persistent) | Any status change | Colour + status label |
| Error card (`/errors`) | Status reaches `error` | `"{name} connection error"` + static troubleshooting guide |
| Re-login modal | YouTube or Facebook reaches `error` | Inline auth form |
| Toast (info) | Generic server `notification` level=info | Raw server string |
| Toast (warning) | Generic server `notification` level=warn | Raw server string |
| Toast (error) | `error` WS message *or* level=error notification | Raw server string |
| Inline settings text | Save / auth API failure | Per-operation error string |

**Key gap:** Status transitions to `connecting` or `error` do **not** produce a toast.
The operator must notice the badge change themselves.

---

## Design brief — what to solve

Design a notification system for these five connector types that satisfies three operators:

- **Camera operator:** Watches the presenter screen, glances at the control app occasionally.
  Needs a hard-to-miss alert when OBS or ATEM loses connection mid-service.

- **Streaming operator:** Monitors the dashboard during the service.
  Needs to know immediately if YouTube auth expires before or during a live stream.

- **Tech director:** Troubleshoots issues between services.
  Needs a clear, actionable guide for each error type without opening a browser.

### Requirements

**R1 — Urgency tiers**

Not all errors are equal. The design must distinguish:

| Tier | Examples | Delivery |
|------|---------|---------|
| Critical | OBS error during recording; ATEM error during live service | Full-screen or persistent banner + sound optional |
| High | YouTube re-auth needed; OBS device missing | Persistent toast + error badge |
| Medium | Discord webhook dead; ATEM error outside service | Error card only (no toast) |
| Low | Informational state changes (connecting → connected) | Badge update only, no notification |

**R2 — Recovery CTAs in the notification**

Every error notification must contain at least one recovery action:

- `Recheck` — re-tests the connection immediately (shows spinner while pending)
- `Fix` — opens the connector's settings form inline or in a side drawer
- `Re-login` — for OAuth connectors (YouTube, Facebook); initiates auth flow inline
- `Dismiss` — removes the notification without fixing (available on all tiers ≤ High)

**R3 — Missing device: name the device**

When an OBS device listener goes unavailable, the notification must name the device
by its friendly label (e.g. "Rode Interface unavailable") — not show a generic message.
Recovery hint: "Check USB/Bluetooth connection or update the source name in OBS."

**R4 — Connecting state feedback**

When a connector enters `connecting`, show a non-blocking indicator (animated badge or
brief inline toast: "Reconnecting to OBS…") so the operator knows the system is trying,
not frozen.

**R5 — Error detail is static, not dynamic**

The server does not send error reason text. Each connector has a static troubleshooting
guide (markdown steps). The notification must:
- Show a short summary (one line) in the notification itself
- Offer an expandable "Why?" section that renders the full troubleshooting steps
- Not overwhelm the operator with a wall of text at first glance

**R6 — Non-intrusive during normal operation**

When all connectors are `connected`, the notification surface must be invisible or
reduced to a single quiet status indicator. Alerts only appear when something is wrong.

**R7 — Grouping**

If multiple connectors error simultaneously (e.g. network outage), group them into
one notification rather than stacking five separate toasts.

---

## Visual constraints

- Dark-background desktop UI (Tauri app, windowed)
- Existing status colours: green `#10B981`, amber `#F59E0B`, red `#EF4444`, grey `#6B7280`
- Existing badge anatomy: `[coloured dot] [connector name] [status label]`
- Toast position: top-right (svelte-sonner, `richColors`, close button visible)
- The operator may be in a darkened room; contrast ratio WCAG AA minimum

---

## Connector-specific troubleshooting copy (for expandable "Why?" sections)

### OBS Studio
1. Open OBS Studio on this machine.
2. Go to **Tools → WebSocket Server Settings** and enable the WebSocket server.
3. Note the port (default: **4455**) and set a secure password.
4. Click **Apply** and **OK**.
5. In Settings, enter the host, port, and password, then click **Save & Reconnect**.

### Blackmagic ATEM
*(Integration not yet fully implemented — save settings for future use)*
1. Ensure the ATEM switcher is on and connected to the same network.
2. Find the IP address in **ATEM Software Control → Preferences**.
3. In Settings, enter the host and port (**9910**), then save.

### YouTube
1. Go to **Google Cloud Console** and create an OAuth 2.0 Client ID.
2. Enable the **YouTube Data API v3** for your project.
3. In Settings, enter your Client ID and Client Secret, then save.
4. Click **Login with YouTube** and authenticate with your church's Google account.
5. Grant the required YouTube permissions when prompted.

### Discord
*(Integration not yet fully implemented — save settings for future use)*
1. In your Discord server, go to **Server Settings → Integrations → Webhooks**.
2. Create a new webhook for the announcements channel.
3. Copy the **Webhook URL** and paste it into Settings.

### OBS Device Listener (missing device)
1. Check that the physical device (USB mic, camera) is plugged in and powered.
2. Open OBS and confirm the source named **"{device label}"** is visible and active.
3. If the source was renamed, update the listener name in **Settings → OBS Devices**.
4. If the device was replaced, delete this listener and add a new one.

---

## Deliverables requested

1. **Notification anatomy** — annotated component showing all states (idle / connecting /
   error / grouped errors) with copy, icon, colour, and CTA placement.

2. **Urgency tier examples** — one mockup per tier showing how the same interface
   looks for Critical (OBS mid-service), High (YouTube re-auth), and Low (badge-only).

3. **Missing-device card** — specific mockup for the OBS device listener error, naming
   the device and showing the expandable guide.

4. **Re-login flow** — YouTube/Facebook inline auth modal triggered from the notification
   CTA, without leaving the current page.

5. **Grouped error state** — what the operator sees if OBS + ATEM + one device all fail
   at once.

6. **Connecting transition** — animated badge or toast for the reconnecting state.
