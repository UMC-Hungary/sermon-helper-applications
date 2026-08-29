import type { ConnectorDefinition } from './types.js';

export interface AtemConfig {
	enabled: boolean;
	host: string;
	port: number;
}

export const atemDefinition: ConnectorDefinition<AtemConfig> = {
	id: 'atem',
	name: 'Blackmagic ATEM',
	category: 'software-device',
	capabilities: { streaming: true, recording: true, live: false },
	infoMarkdown: `## Blackmagic ATEM Connection Troubleshooting

ATEM integration is not yet fully implemented.

1. Ensure your ATEM switcher is connected to the same network.
2. Note the ATEM IP address from the ATEM Software Control app.
3. In Settings, enter the host and port (default: 9910).
4. Save the configuration for future use when ATEM support is complete.`,
	isConfigured(config) {
		return config.enabled && config.host.length > 0;
	}
};
