# Sermon Helper - Companion Module

Control Broadlink IR/RF devices through the Sermon Helper desktop application.

## Prerequisites

- Sermon Helper desktop application running with the Discovery Server enabled
- Broadlink devices configured in the Sermon Helper app
- RF/IR commands saved in the Sermon Helper app

## Configuration

### Connection Settings

- **Host**: The IP address or hostname of the computer running Sermon Helper (default: `localhost`)
- **Port**: The Discovery Server API port (default: `8765`)
- **Auth Token**: Optional authentication token if configured in Sermon Helper

## Actions

### Execute RF/IR Command

Triggers a saved RF/IR command by its slug identifier.

- **Command**: Select from the list of available commands synced from Sermon Helper

### Execute Command by Category

Filter commands by category before selecting.

- **Category**: Select a category (Projector, Screen, HVAC, Lighting, Audio, Other)
- **Command**: Select from the filtered list of commands

### Refresh Command List

Manually refresh the list of available commands from the server.

## Feedbacks

### Connection Status

Shows whether the module is connected to the Sermon Helper server.

- **Connected**: Green indicator
- **Disconnected**: Red indicator

## Variables

| Variable | Description |
|----------|-------------|
| `$(sermon-helper:connection_status)` | Current connection status |
| `$(sermon-helper:last_command)` | Name of the last executed command |
| `$(sermon-helper:command_count)` | Total number of available commands |

## Presets

The module automatically generates preset buttons for each RF/IR command configured in Sermon Helper. Presets are color-coded by category:

- **Projector**: Rosy Copper (#dd614a)
- **Screen**: Coral Glow (#f48668)
- **HVAC**: Powder Blush (#f4a698)
- **Lighting**: Dry Sage (#c5c392)
- **Audio**: Muted Teal (#73a580)
- **Other**: Gray (#808080)

## Troubleshooting

### Module shows "Disconnected"

1. Verify Sermon Helper is running
2. Check that the Discovery Server is enabled in Sermon Helper settings
3. Verify the host and port are correct
4. Check firewall settings allow connections on port 8765

### Commands not appearing

1. Ensure commands are saved in Sermon Helper
2. Click "Refresh Commands" to manually sync
3. Check the Sermon Helper logs for any errors

