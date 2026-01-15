import { youtubeApi } from '$lib/utils/youtube-api';
import { toast } from '$lib/utils/toast';

// Check if running in Tauri
function isTauri(): boolean {
	return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

export interface OAuthCallbackResult {
	success: boolean;
	error?: string;
}

// Parse OAuth callback URL
function parseOAuthCallback(url: string): { code?: string; error?: string; state?: string } {
	try {
		// URL format: sermon-helper://oauth/callback?code=xxx&state=xxx
		const urlObj = new URL(url);
		return {
			code: urlObj.searchParams.get('code') || undefined,
			error: urlObj.searchParams.get('error') || undefined,
			state: urlObj.searchParams.get('state') || undefined
		};
	} catch {
		return { error: 'Invalid callback URL' };
	}
}

// Handle OAuth callback
async function handleOAuthCallback(url: string): Promise<OAuthCallbackResult> {
	const { code, error } = parseOAuthCallback(url);

	if (error) {
		return { success: false, error };
	}

	if (!code) {
		return { success: false, error: 'No authorization code received' };
	}

	try {
		await youtubeApi.exchangeCodeForTokens(code);
		return { success: true };
	} catch (err) {
		return {
			success: false,
			error: err instanceof Error ? err.message : 'Failed to exchange code'
		};
	}
}

// Subscribers for OAuth completion
type OAuthCompletionCallback = (result: OAuthCallbackResult) => void;
const completionCallbacks: Set<OAuthCompletionCallback> = new Set();

export function onOAuthComplete(callback: OAuthCompletionCallback): () => void {
	completionCallbacks.add(callback);
	return () => completionCallbacks.delete(callback);
}

// Initialize deep link listener
export async function initOAuthHandler(): Promise<void> {
	// Deep links only work in Tauri
	if (!isTauri()) {
		console.log('Not running in Tauri, deep link handler skipped');
		return;
	}

	try {
		const { onOpenUrl, getCurrent } = await import('@tauri-apps/plugin-deep-link');

		// Check if app was launched via deep link
		const startUrls = await getCurrent();
		if (startUrls) {
			for (const url of startUrls) {
				if (url.startsWith('sermon-helper://oauth/callback')) {
					const result = await handleOAuthCallback(url);
					completionCallbacks.forEach((cb) => cb(result));

					// Show toast for startup callback
					if (result.success) {
						toast({
							title: 'Logged In',
							description: 'Successfully logged in to YouTube',
							variant: 'success'
						});
					} else {
						toast({
							title: 'Login Failed',
							description: result.error || 'Unknown error',
							variant: 'error'
						});
					}
				}
			}
		}

		// Listen for deep links while app is running
		await onOpenUrl(async (urls: string[]) => {
			for (const url of urls) {
				if (url.startsWith('sermon-helper://oauth/callback')) {
					const result = await handleOAuthCallback(url);

					// Notify subscribers
					completionCallbacks.forEach((cb) => cb(result));

					// Show toast
					if (result.success) {
						toast({
							title: 'Logged In',
							description: 'Successfully logged in to YouTube',
							variant: 'success'
						});
					} else {
						toast({
							title: 'Login Failed',
							description: result.error || 'Unknown error',
							variant: 'error'
						});
					}
				}
			}
		});
	} catch (error) {
		// Deep link plugin may not be available in dev mode without Tauri
		console.warn('Deep link handler initialization failed:', error);
	}
}
