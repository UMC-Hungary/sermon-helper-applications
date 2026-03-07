import type { ConnectorDefinition } from './types.js';

export interface ObsBadgeConfig {
	enabled: boolean;
	sceneName: string;
}

export const obsBadgeDefinition: ConnectorDefinition<ObsBadgeConfig> = {
	id: 'obs-badge',
	name: 'OBS Badge',
	category: 'software-device',
	capabilities: { streaming: false, recording: false, live: false },
	infoMarkdown: `## OBS Badge Setup

This connector creates a liquid glass badge in OBS with the LucidGlass shader.

### Prerequisites:
1. OBS Studio must be installed
2. OBS WebSocket Server must be enabled (Tools → WebSocket Server Settings)
3. OBS must be connected first (use the OBS connector)

### Setup:
1. Connect to OBS using the OBS connector
2. Select the target scene from the dropdown
3. Click "Install & Create Badge"
4. The badge sources will be created in the selected scene

### Sources Created:
- \`__caption\` - HTML browser source for caption text and logo
- \`__caption-background\` - Same source with LucidGlass shader applied (glass effect)

### Caption URL:
\`http://localhost:3737/caption?type=caption&resolution=4k&bold=Textus:&light=Lekcio:&color=red&showLogo=true\``,
	isConfigured(config) {
		return config.enabled && config.sceneName.length > 0;
	}
};
