import { derived, get } from 'svelte/store';
import { appSettings, appSettingsStore } from '$lib/utils/app-settings-store';
import { updateYoutubeLogin } from '$lib/stores/system-store';
import { youtubeAuthStatusStore } from '$lib/stores/youtube-auth-status-store';
import type { YouTubeTokens, YouTubeOAuthConfig } from '$lib/types/youtube';

// Derived store for YouTube tokens
export const youtubeTokens = derived(
	appSettings,
	($settings) => $settings?.youtubeTokens ?? null
);

// Derived store for OAuth config
export const youtubeOAuthConfig = derived(
	appSettings,
	($settings) => $settings?.youtubeOAuthConfig ?? null
);

// Check if YouTube tokens are present (persisted, survives refresh)
export const isYouTubeConnected = derived(
	youtubeTokens,
	($tokens) => $tokens !== null && !!$tokens.accessToken
);

// Check if tokens are valid (not expired, with 5 min buffer)
export const isTokenValid = derived(youtubeTokens, ($tokens) => {
	if (!$tokens) return false;
	const bufferMs = 5 * 60 * 1000; // 5 minutes
	return $tokens.expiresAt > Date.now() + bufferMs;
});

// YouTube auth store operations
export const youtubeAuthStore = {
	async setOAuthConfig(config: YouTubeOAuthConfig): Promise<void> {
		await appSettingsStore.set('youtubeOAuthConfig', config);
	},

	async setTokens(tokens: YouTubeTokens): Promise<void> {
		await appSettingsStore.set('youtubeTokens', tokens);
		updateYoutubeLogin(true);
		youtubeAuthStatusStore.setLoggedIn();
	},

	async clearTokens(): Promise<void> {
		await appSettingsStore.set('youtubeTokens', null);
		updateYoutubeLogin(false);
		// Note: Don't call setLoggedOut here - let the calling code handle
		// reauth_required vs logged_out distinction
	},

	getTokens(): YouTubeTokens | null {
		return get(youtubeTokens);
	},

	getConfig(): YouTubeOAuthConfig | null {
		return get(youtubeOAuthConfig);
	},

	isLoggedIn(): boolean {
		return get(isTokenValid);
	}
};
