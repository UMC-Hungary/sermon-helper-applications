import { invoke } from '@tauri-apps/api/core';

export interface BadgeInstallResult {
	shaderfilter_installed: boolean;
	shader_installed: boolean;
	sources_created: boolean;
}

export interface ObsScene {
	name: string;
}

export async function installBadge(): Promise<BadgeInstallResult> {
	return invoke<BadgeInstallResult>('install_badge');
}

export async function getObsScenes(): Promise<ObsScene[]> {
	return invoke<ObsScene[]>('get_obs_scenes');
}

export async function createBadgeSources(sceneName: string): Promise<void> {
	return invoke('create_badge_sources', { sceneName });
}
