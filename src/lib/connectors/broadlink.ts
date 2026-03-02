import type { ConnectorDefinition } from './types.js';

export interface BroadlinkConfig {
	enabled: boolean;
	host: string;
	port: number;
}

export const broadlinkDefinition: ConnectorDefinition<BroadlinkConfig> = {
	id: 'broadlink',
	name: 'BroadLink',
	category: 'software-device',
	capabilities: { streaming: true, recording: true, live: false },
	infoMarkdown: `## BroadLink Connection Troubleshooting

BroadLink integration is not yet fully implemented.

1. Ensure your BroadLink hub is connected to the same network.
2. Note the hub IP address from the BroadLink app or your router.
3. In Settings, enter the host and port (default: 80).
4. Save the configuration for future use when BroadLink support is complete.`,
	isConfigured(config) {
		return config.enabled && config.host.length > 0;
	}
};
