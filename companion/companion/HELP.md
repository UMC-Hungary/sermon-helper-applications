# Sermon Helper - Companion Module

Control Broadlink IR/RF devices and presentations through the Sermon Helper desktop application via WebSocket.

## Prerequisites

- Sermon Helper desktop application running
- Broadlink devices and/or Keynote configured in the Sermon Helper app
- Auth token from Sermon Helper settings

## Configuration

### Connection Settings

- **Host**: IP address or hostname of the computer running Sermon Helper (default: `127.0.0.1`)
- **Port**: Sermon Helper WebSocket port (default: `3737`)
- **Auth Token**: Authentication token from Sermon Helper settings

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
| Connection Status | Green when connected to Sermon Helper |
| Command Available | Blue when a specific command is loaded |
| PPT: Slot Has File | Green when a PPT slot contains a file |
| PPT: Filter Active | Orange when a digit filter is applied |
| Presentation: Slideshow Active | Green when a slideshow is running |
| Presentation: Screen Blanked | Black when the screen is blanked |

## Variables

| Variable | Description |
|----------|-------------|
| `$(sermon-helper:connection_status)` | Connected / Disconnected |
| `$(sermon-helper:last_command)` | Name of the last executed command |
| `$(sermon-helper:command_count)` | Total available commands |
| `$(sermon-helper:ppt_filter)` | Current digit filter |
| `$(sermon-helper:ppt_match_count)` | Number of matching files |
| `$(sermon-helper:ppt_folder_name)` | Selected folder name |
| `$(sermon-helper:ppt_slot_1_name)` … `ppt_slot_5_name` | File names in slots 1–5 |
| `$(sermon-helper:ppt_current_slide)` | Current slide number |
| `$(sermon-helper:ppt_total_slides)` | Total slides |
| `$(sermon-helper:ppt_slideshow_active)` | ON / OFF |
| `$(sermon-helper:ppt_app)` | Presentation app name |
| `$(sermon-helper:ppt_blanked)` | YES / NO |

## Troubleshooting

### Module shows "Disconnected"
1. Verify Sermon Helper is running
2. Check host and port settings
3. Verify the auth token matches Sermon Helper settings
4. Check firewall allows connections on port 3737

### Commands not appearing
1. Ensure Broadlink commands are saved in Sermon Helper
2. Click the **Refresh Command List** action button
