import { get } from 'svelte/store';
import { youtubeAuthStore, youtubeTokens, youtubeOAuthConfig } from '$lib/stores/youtube-store';
import {
	YOUTUBE_AUTH_URL,
	YOUTUBE_TOKEN_URL,
	YOUTUBE_API_BASE,
	YOUTUBE_OAUTH_SCOPES,
	type YouTubeTokens,
	type YouTubeBroadcastRequest,
	type YouTubeBroadcastResponse
} from '$lib/types/youtube';

// Check if running in Tauri
function isTauri(): boolean {
	return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

// OAuth callback result from the local server
interface OAuthCallbackResult {
	code: string | null;
	state: string | null;
	error: string | null;
	error_description: string | null;
}

// Current OAuth port (set when server starts)
let currentOAuthPort: number | null = null;

// Get the appropriate redirect URI based on environment
function getRedirectUri(port?: number): string {
	if (isTauri() && port) {
		// Use loopback address for desktop OAuth (Google requirement)
		return `http://127.0.0.1:${port}/callback`;
	} else if (!isTauri()) {
		// Web mode - use current origin
		return `${window.location.origin}/auth/callback`;
	}
	// Fallback (should not happen)
	return `${window.location.origin}/auth/callback`;
}

// Open URL in browser (works in both Tauri and web)
async function openBrowserUrl(url: string): Promise<void> {
	if (isTauri()) {
		try {
			console.log('Opening URL in Tauri:', url);
			const { openUrl } = await import('@tauri-apps/plugin-opener');
			await openUrl(url);
			console.log('URL opened successfully');
		} catch (err) {
			console.error('Failed to open URL with Tauri opener:', err);
			// Fallback: try using window.open
			console.log('Trying fallback window.open...');
			const opened = window.open(url, '_blank');
			if (!opened) {
				throw new Error(
					'Failed to open browser. If running in WSL, try copying this URL manually: ' + url
				);
			}
		}
	} else {
		// Web fallback - open in same window for OAuth flow
		window.location.href = url;
	}
}

class YouTubeApiService {
	// Generate OAuth authorization URL (with optional port for loopback)
	getAuthUrl(port?: number): string {
		const config = get(youtubeOAuthConfig);
		if (!config) throw new Error('YouTube OAuth config not set');

		// Use environment-appropriate redirect URI
		const redirectUri = getRedirectUri(port);

		const params = new URLSearchParams({
			client_id: config.clientId,
			redirect_uri: redirectUri,
			response_type: 'code',
			scope: YOUTUBE_OAUTH_SCOPES,
			access_type: 'offline',
			prompt: 'consent',
			state: crypto.randomUUID() // CSRF protection
		});

		return `${YOUTUBE_AUTH_URL}?${params.toString()}`;
	}

	// Start OAuth flow - different behavior for Tauri vs web
	// In Tauri: starts local server, opens browser, waits for callback, returns tokens
	// In Web: redirects to Google (callback handled by /auth/callback page)
	async startOAuthFlow(): Promise<YouTubeTokens | void> {
		if (isTauri()) {
			// In Tauri, use the full OAuth flow with loopback server
			return this.completeOAuthFlow();
		} else {
			// In web mode, just redirect to Google
			const authUrl = this.getAuthUrl();
			await openBrowserUrl(authUrl);
			// Returns void - the callback page will handle token exchange
		}
	}

	// Complete OAuth flow in Tauri - starts server, opens browser, waits for callback
	async completeOAuthFlow(): Promise<YouTubeTokens> {
		if (!isTauri()) {
			throw new Error('This method is only for Tauri mode');
		}

		const { invoke } = await import('@tauri-apps/api/core');
		const { listen } = await import('@tauri-apps/api/event');

		console.log('Starting complete OAuth flow...');

		// Start server first to get the port
		const port = await invoke<number>('start_oauth_callback_server');
		currentOAuthPort = port;
		this._oauthPort = port;
		console.log('OAuth server started on port:', port);

		// Generate auth URL with the correct redirect URI
		const authUrl = this.getAuthUrl(port);
		console.log('Opening OAuth URL:', authUrl);

		// Set up event listener before opening browser
		const resultPromise = new Promise<YouTubeTokens>((resolve, reject) => {
			this._oauthResolve = resolve;
			this._oauthReject = reject;

			// Listen for the OAuth callback event from Rust
			listen<OAuthCallbackResult>('oauth-callback', async (event) => {
				console.log('Received OAuth callback event:', event.payload);
				try {
					const tokens = await this.handleOAuthResult(event.payload);
					resolve(tokens);
				} catch (err) {
					reject(err);
				}
			}).then((unlisten) => {
				// Store unlisten function to clean up later if needed
				this._oauthUnlisten = unlisten;
			});

			// Set a timeout
			setTimeout(() => {
				reject(new Error('OAuth timeout - no callback received within 5 minutes'));
			}, 5 * 60 * 1000);
		});

		// Open browser
		await openBrowserUrl(authUrl);

		return resultPromise;
	}

	// Internal storage for OAuth promise resolvers
	private _oauthResolve?: (tokens: YouTubeTokens) => void;
	private _oauthReject?: (error: Error) => void;
	private _oauthPort?: number;
	private _oauthUnlisten?: () => void;

	// Get current OAuth port (for redirect URI display)
	getCurrentOAuthPort(): number | null {
		return this._oauthPort || currentOAuthPort;
	}

	// Handle OAuth callback result (called from event handler)
	async handleOAuthResult(result: OAuthCallbackResult): Promise<YouTubeTokens> {
		// Clean up event listener
		this._oauthUnlisten?.();

		if (result.error) {
			const error = new Error(result.error_description || result.error);
			throw error;
		}

		if (!result.code) {
			const error = new Error('No authorization code received');
			throw error;
		}

		// Exchange code for tokens using the correct redirect URI
		const tokens = await this.exchangeCodeForTokens(result.code, this._oauthPort);
		return tokens;
	}

	// Exchange authorization code for tokens
	async exchangeCodeForTokens(code: string, port?: number): Promise<YouTubeTokens> {
		const config = get(youtubeOAuthConfig);
		if (!config) throw new Error('YouTube OAuth config not set');

		// Use environment-appropriate redirect URI (must match what was used in auth request)
		const redirectUri = getRedirectUri(port || this._oauthPort);

		console.log('Exchanging code for tokens with redirect URI:', redirectUri);

		let response: Response;
		try {
			response = await fetch(YOUTUBE_TOKEN_URL, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/x-www-form-urlencoded'
				},
				body: new URLSearchParams({
					client_id: config.clientId,
					client_secret: config.clientSecret,
					code,
					grant_type: 'authorization_code',
					redirect_uri: redirectUri
				})
			});
		} catch (err) {
			// This usually means CORS blocked the request
			console.error('Token exchange fetch failed (likely CORS):', err);
			throw new Error(
				'Token exchange failed - this may be a CORS issue. In web dev mode, OAuth token exchange may not work. Please use Tauri dev mode (pnpm tauri dev) for full OAuth functionality.'
			);
		}

		if (!response.ok) {
			const errorText = await response.text();
			console.error('Token exchange error response:', errorText);
			let errorMessage = 'Failed to exchange code';
			try {
				const errorJson = JSON.parse(errorText);
				errorMessage = errorJson.error_description || errorJson.error || errorMessage;
			} catch {
				errorMessage = errorText || errorMessage;
			}
			throw new Error(errorMessage);
		}

		const data = await response.json();

		const tokens: YouTubeTokens = {
			accessToken: data.access_token,
			refreshToken: data.refresh_token,
			expiresAt: Date.now() + data.expires_in * 1000,
			scope: data.scope
		};

		await youtubeAuthStore.setTokens(tokens);
		return tokens;
	}

	// Refresh access token using refresh token
	async refreshAccessToken(): Promise<YouTubeTokens> {
		const config = get(youtubeOAuthConfig);
		const tokens = get(youtubeTokens);

		if (!config) throw new Error('YouTube OAuth config not set');
		if (!tokens?.refreshToken) throw new Error('No refresh token available');

		const response = await fetch(YOUTUBE_TOKEN_URL, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/x-www-form-urlencoded'
			},
			body: new URLSearchParams({
				client_id: config.clientId,
				client_secret: config.clientSecret,
				refresh_token: tokens.refreshToken,
				grant_type: 'refresh_token'
			})
		});

		if (!response.ok) {
			const error = await response.json();
			// If refresh fails, clear tokens and require re-auth
			await youtubeAuthStore.clearTokens();
			throw new Error(error.error_description || 'Failed to refresh token');
		}

		const data = await response.json();

		const newTokens: YouTubeTokens = {
			accessToken: data.access_token,
			refreshToken: tokens.refreshToken, // Keep existing refresh token
			expiresAt: Date.now() + data.expires_in * 1000,
			scope: data.scope || tokens.scope
		};

		await youtubeAuthStore.setTokens(newTokens);
		return newTokens;
	}

	// Get valid access token (refreshes if needed)
	async getValidAccessToken(): Promise<string> {
		let tokens = get(youtubeTokens);
		if (!tokens) throw new Error('Not logged in to YouTube');

		const bufferMs = 5 * 60 * 1000; // 5 minutes buffer
		if (tokens.expiresAt <= Date.now() + bufferMs) {
			tokens = await this.refreshAccessToken();
		}

		return tokens.accessToken;
	}

	// Create a live broadcast
	async createBroadcast(request: YouTubeBroadcastRequest): Promise<YouTubeBroadcastResponse> {
		const accessToken = await this.getValidAccessToken();

		const response = await fetch(
			`${YOUTUBE_API_BASE}/liveBroadcasts?part=snippet,status,contentDetails`,
			{
				method: 'POST',
				headers: {
					Authorization: `Bearer ${accessToken}`,
					'Content-Type': 'application/json'
				},
				body: JSON.stringify({
					snippet: {
						title: request.title.substring(0, 100), // Max 100 chars
						description: request.description.substring(0, 5000), // Max 5000 chars
						scheduledStartTime: request.scheduledStartTime
					},
					status: {
						privacyStatus: request.privacyStatus,
						selfDeclaredMadeForKids: false
					},
					contentDetails: {
						enableAutoStart: request.enableAutoStart ?? false,
						enableAutoStop: request.enableAutoStop ?? true,
						enableDvr: request.enableDvr ?? true,
						enableEmbed: request.enableEmbed ?? true,
						recordFromStart: true
					}
				})
			}
		);

		if (!response.ok) {
			const error = await response.json();
			throw new Error(error.error?.message || 'Failed to create broadcast');
		}

		return response.json();
	}

	// Get broadcast details
	async getBroadcast(broadcastId: string): Promise<YouTubeBroadcastResponse | null> {
		const accessToken = await this.getValidAccessToken();

		const response = await fetch(
			`${YOUTUBE_API_BASE}/liveBroadcasts?part=snippet,status,contentDetails&id=${broadcastId}`,
			{
				headers: {
					Authorization: `Bearer ${accessToken}`
				}
			}
		);

		if (!response.ok) {
			const error = await response.json();
			throw new Error(error.error?.message || 'Failed to get broadcast');
		}

		const data = await response.json();
		return data.items?.[0] ?? null;
	}

	// Update an existing broadcast
	async updateBroadcast(
		broadcastId: string,
		request: YouTubeBroadcastRequest
	): Promise<YouTubeBroadcastResponse> {
		const accessToken = await this.getValidAccessToken();

		// First, fetch the existing broadcast to get current contentDetails
		const existing = await this.getBroadcast(broadcastId);
		if (!existing) {
			throw new Error('Broadcast not found');
		}

		const response = await fetch(
			`${YOUTUBE_API_BASE}/liveBroadcasts?part=snippet,status,contentDetails`,
			{
				method: 'PUT',
				headers: {
					Authorization: `Bearer ${accessToken}`,
					'Content-Type': 'application/json'
				},
				body: JSON.stringify({
					id: broadcastId,
					snippet: {
						title: request.title.substring(0, 100),
						description: request.description.substring(0, 5000),
						scheduledStartTime: request.scheduledStartTime
					},
					status: {
						privacyStatus: request.privacyStatus,
						selfDeclaredMadeForKids: false
					},
					// Preserve existing contentDetails and merge with updates
					contentDetails: {
						...existing.contentDetails,
						enableDvr: request.enableDvr ?? existing.contentDetails.enableDvr,
						enableEmbed: request.enableEmbed ?? existing.contentDetails.enableEmbed
					}
				})
			}
		);

		if (!response.ok) {
			const error = await response.json();
			throw new Error(error.error?.message || 'Failed to update broadcast');
		}

		return response.json();
	}

	// Get YouTube Studio URL for a broadcast
	getYoutubeStudioUrl(broadcastId: string): string {
		return `https://studio.youtube.com/video/${broadcastId}/livestreaming`;
	}

	// Logout - clear tokens
	async logout(): Promise<void> {
		await youtubeAuthStore.clearTokens();
	}
}

export const youtubeApi = new YouTubeApiService();
