# companion-module-sermon-helper

Bitfocus Companion module for controlling Broadlink IR/RF devices through the Sermon Helper application.

## Installation

### For Development

1. Create a `companion-module-dev` folder on your system
2. Clone or copy this module into that folder
3. In Companion settings, set the developer modules path to your `companion-module-dev` folder
4. Run `npm install` and `npm run build` in this module's directory
5. Restart Companion

### Building

```bash
npm install
npm run build
```

### Development Mode

```bash
npm run dev
```

This will watch for changes and recompile automatically.

## Configuration

- **Host**: IP address or hostname of the Sermon Helper server (default: `localhost`)
- **Port**: Discovery Server API port (default: `8765`)
- **Auth Token**: Optional authentication token
- **Use Auto-Discovery**: Enable mDNS discovery for automatic server detection
- **Poll Interval**: How often to refresh command list (default: 30 seconds)

## Actions

| Action | Description |
|--------|-------------|
| Execute RF/IR Command | Execute a saved command by slug |
| Execute Command by Category | Execute a command with category filter |
| Refresh Command List | Manually refresh commands from server |

## Requirements

- Sermon Helper desktop application running with Discovery Server enabled
- Broadlink devices configured in Sermon Helper
- RF/IR commands saved in Sermon Helper

## API Endpoints Used

- `GET /api/v1/health` - Connection health check
- `GET /api/v1/rfir/commands` - List all commands
- `POST /api/v1/rfir/commands/:slug/execute` - Execute a command
- `WS /ws` - Real-time status updates

## License

MIT
