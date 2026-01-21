// Multi-platform upload configuration types

// Supported upload platforms
export type UploadPlatform = 'youtube' | 'facebook' | 'custom';

// Base configuration for all platforms
export interface UploadPlatformConfig {
	platform: UploadPlatform;
	enabled: boolean;
	autoUpload: boolean; // Upload automatically when event ends
}

// YouTube-specific config
export interface YouTubeUploadConfig extends UploadPlatformConfig {
	platform: 'youtube';
	useEventPrivacy: boolean; // Use event's youtubeUploadPrivacyStatus
	defaultPrivacy: 'public' | 'unlisted' | 'private';
	publishAfterUpload: boolean; // Auto-publish when processing complete
}

// Facebook-specific config (for future)
export interface FacebookUploadConfig extends UploadPlatformConfig {
	platform: 'facebook';
	pageId?: string; // Facebook Page to upload to
	defaultPrivacy: 'EVERYONE' | 'ALL_FRIENDS' | 'SELF';
	crosspostToTimeline: boolean;
}

// Custom webhook config (for future extensibility)
export interface CustomUploadConfig extends UploadPlatformConfig {
	platform: 'custom';
	webhookUrl: string;
	headers?: Record<string, string>;
}

// Union type for all platform configs
export type PlatformUploadConfig = YouTubeUploadConfig | FacebookUploadConfig | CustomUploadConfig;

// Main upload settings
export interface UploadSettings {
	platforms: PlatformUploadConfig[];
	minDurationMinutes: number; // Minimum duration to trigger auto-upload (default: 45)
	shortVideoThresholdMinutes: number; // Videos shorter than this are "short" (default: 10)
	retryAttempts: number; // Number of retry attempts (default: 3)
	chunkSizeMB: number; // Upload chunk size (default: 10)
}

// Default settings
export const DEFAULT_UPLOAD_SETTINGS: UploadSettings = {
	platforms: [
		{
			platform: 'youtube',
			enabled: true,
			autoUpload: true,
			useEventPrivacy: true,
			defaultPrivacy: 'public',
			publishAfterUpload: true
		}
	],
	minDurationMinutes: 45,
	shortVideoThresholdMinutes: 10, // Videos <= 10min are considered "short"
	retryAttempts: 3,
	chunkSizeMB: 10
};

// Helper to get platform config by type
export function getPlatformConfig<T extends PlatformUploadConfig>(
	settings: UploadSettings,
	platform: UploadPlatform
): T | undefined {
	return settings.platforms.find((p) => p.platform === platform) as T | undefined;
}

// Helper to check if a platform is enabled
export function isPlatformEnabled(settings: UploadSettings, platform: UploadPlatform): boolean {
	const config = settings.platforms.find((p) => p.platform === platform);
	return config?.enabled ?? false;
}
