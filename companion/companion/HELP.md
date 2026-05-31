# Metocast Bridge - Companion Module

Control Broadlink IR/RF devices and presentations through the Metocast desktop application via WebSocket.

## Prerequisites

- Metocast desktop application running
- Broadlink devices and/or Keynote configured in the Metocast app
- Auth token from Metocast settings

## Configuration

### Connection Settings

- **Host**: IP address or hostname of the computer running Metocast (default: `127.0.0.1`)
- **Port**: Metocast WebSocket port (default: `3737`)
- **Auth Token**: Authentication token from Metocast settings

## Actions

### RF/IR Commands
- **Execute RF/IR Command** — trigger a saved Broadlink command by slug
- **Execute Command by Category** — filter by category then trigger
- **Refresh Command List** — manually re-fetch the command list

### PPT Selector
- **PPT: Type Digit** — append a digit to the PPT file filter
- **PPT: Backspace** — remove the last digit from the filter
- **PPT: Clear Filter** — clear the filter entirely
- **PPT: Select File** — open the file at a display slot in Keynote
- **PPT: Select Folder** — switch to a different PPT folder
- **PPT: Refresh Files** — refresh the file list

### Presentation Control (Keynote)
- **Presentation: Open File** — open a presentation by path
- **Presentation: Start / Stop Slideshow**
- **Presentation: Close All / Close Latest**
- **Presentation: Next / Previous / First / Last Slide**
- **Presentation: Go to Slide** — jump to a specific slide number
- **Presentation: Toggle Blank Screen**

## Feedbacks

| Feedback | Description |
|----------|-------------|
| Connection Status | Green when connected to Metocast |
| Command Available | Blue when a specific command is loaded |
| PPT: Slot Has File | Green when a PPT slot contains a file |
| PPT: Filter Active | Orange when a digit filter is applied |
| Presentation: Slideshow Active | Green when a slideshow is running |
| Presentation: Screen Blanked | Black when the screen is blanked |

## Variables

| Variable | Description |
|----------|-------------|
| `$(metocast-bridge:connection_status)` | Connected / Disconnected |
| `$(metocast-bridge:last_command)` | Name of the last executed command |
| `$(metocast-bridge:command_count)` | Total available commands |
| `$(metocast-bridge:ppt_filter)` | Current digit filter |
| `$(metocast-bridge:ppt_match_count)` | Number of matching files |
| `$(metocast-bridge:ppt_folder_name)` | Selected folder name |
| `$(metocast-bridge:ppt_slot_1_name)` … `ppt_slot_5_name` | File names in slots 1–5 |
| `$(metocast-bridge:ppt_current_slide)` | Current slide number |
| `$(metocast-bridge:ppt_total_slides)` | Total slides |
| `$(metocast-bridge:ppt_slideshow_active)` | ON / OFF |
| `$(metocast-bridge:ppt_app)` | Presentation app name |
| `$(metocast-bridge:ppt_blanked)` | YES / NO |

## Troubleshooting

### Module shows "Disconnected"
1. Verify Metocast is running
2. Check host and port settings
3. Verify the auth token matches Metocast settings
4. Check firewall allows connections on port 3737

### Commands not appearing
1. Ensure Broadlink commands are saved in Metocast
2. Click the **Refresh Command List** action button
