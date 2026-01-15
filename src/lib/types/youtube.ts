export interface YouTubeTokens {
	accessToken: string;
	refreshToken: string;
	expiresAt: number; // Unix timestamp in milliseconds
	scope: string;
}

export interface YouTubeOAuthConfig {
	clientId: string;
	clientSecret: string;
	redirectUri: string; // sermon-helper://oauth/callback
}

export interface YouTubeBroadcastRequest {
	title: string;
	description: string;
	scheduledStartTime: string; // ISO 8601 format
	privacyStatus: 'public' | 'private' | 'unlisted';
	enableAutoStart?: boolean;
	enableAutoStop?: boolean;
	enableDvr?: boolean;
	enableEmbed?: boolean;
}

export interface YouTubeBroadcastResponse {
	id: string;
	snippet: {
		title: string;
		description: string;
		scheduledStartTime: string;
		actualStartTime?: string;
		actualEndTime?: string;
		liveChatId?: string;
	};
	status: {
		lifeCycleStatus: string;
		privacyStatus: string;
		recordingStatus: string;
	};
	contentDetails: {
		boundStreamId?: string;
		enableDvr: boolean;
		enableEmbed: boolean;
	};
}

export const YOUTUBE_OAUTH_SCOPES = [
	'https://www.googleapis.com/auth/youtube',
	'https://www.googleapis.com/auth/youtube.force-ssl'
].join(' ');

export const YOUTUBE_AUTH_URL = 'https://accounts.google.com/o/oauth2/v2/auth';
export const YOUTUBE_TOKEN_URL = 'https://oauth2.googleapis.com/token';
export const YOUTUBE_API_BASE = 'https://www.googleapis.com/youtube/v3';
