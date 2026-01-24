import { writable, derived, get } from 'svelte/store';

export type YouTubeAuthStatus = 'logged_out' | 'logged_in' | 'reauth_required';

export interface YouTubeAuthState {
	status: YouTubeAuthStatus;
	lastError?: string;
	pausedUploadId?: string;
}

const initialState: YouTubeAuthState = {
	status: 'logged_out'
};

const _authState = writable<YouTubeAuthState>(initialState);

export const youtubeAuthStatusStore = {
	subscribe: _authState.subscribe,

	setLoggedIn(): void {
		_authState.set({ status: 'logged_in' });
	},

	setLoggedOut(): void {
		_authState.set({ status: 'logged_out' });
	},

	setReauthRequired(error?: string, pausedUploadId?: string): void {
		_authState.set({
			status: 'reauth_required',
			lastError: error,
			pausedUploadId
		});
	},

	clearReauthRequired(): void {
		_authState.update((state) => {
			if (state.status === 'reauth_required') {
				return { status: 'logged_out' };
			}
			return state;
		});
	},

	getPausedUploadId(): string | undefined {
		return get(_authState).pausedUploadId;
	},

	getStatus(): YouTubeAuthStatus {
		return get(_authState).status;
	}
};

// Derived stores for easy access
export const youtubeAuthStatus = derived(_authState, ($state) => $state.status);
export const youtubeReauthRequired = derived(_authState, ($state) => $state.status === 'reauth_required');
export const youtubeReauthError = derived(_authState, ($state) => $state.lastError);
export const youtubePausedUploadId = derived(_authState, ($state) => $state.pausedUploadId);
