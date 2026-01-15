<script lang="ts">
	import { _ } from 'svelte-i18n';
	import Button from '$lib/components/ui/button.svelte';
	import Input from '$lib/components/ui/input.svelte';
	import Label from '$lib/components/ui/label.svelte';
	import Alert from '$lib/components/ui/alert.svelte';
	import AlertTitle from '$lib/components/ui/alert-title.svelte';
	import AlertDescription from '$lib/components/ui/alert-description.svelte';
	import { youtubeApi } from '$lib/utils/youtube-api';
	import { youtubeAuthStore, youtubeOAuthConfig } from '$lib/stores/youtube-store';
	import { onOAuthComplete } from '$lib/utils/oauth-handler';
	import { toast } from '$lib/utils/toast';
	import { LogIn, Loader2, AlertCircle, Settings, Copy } from 'lucide-svelte';
	import { onMount, onDestroy } from 'svelte';

	// Check if running in Tauri
	function isTauri(): boolean {
		return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
	}

	// Get redirect URI for display
	// For desktop apps, Google uses loopback addresses which don't need to be pre-registered
	function getDisplayRedirectUri(): string {
		if (typeof window === 'undefined') return '';
		if (isTauri()) {
			// Desktop apps use loopback - no need to configure in Google Console
			return 'http://127.0.0.1 (auto-configured)';
		}
		return `${window.location.origin}/auth/callback`;
	}

	export let open: boolean = false;
	export let onClose: () => void = () => {};
	export let onSuccess: () => void = () => {};

	let dialogElement: HTMLDialogElement;
	let isLoading = false;
	let showConfigForm = false;
	let manualAuthUrl = ''; // For manual copy if browser doesn't open

	// Config form state
	let clientId = '';
	let clientSecret = '';

	// Load existing config when mounted
	onMount(() => {
		const config = youtubeAuthStore.getConfig();
		if (config) {
			clientId = config.clientId;
			clientSecret = config.clientSecret;
		}
	});

	// Subscribe to config changes
	$: if ($youtubeOAuthConfig) {
		clientId = $youtubeOAuthConfig.clientId;
		clientSecret = $youtubeOAuthConfig.clientSecret;
	}

	// Dialog open/close effect
	$: if (open && dialogElement) {
		dialogElement.showModal();
	} else if (!open && dialogElement) {
		dialogElement.close();
	}

	// Subscribe to OAuth completion
	let unsubscribe: (() => void) | null = null;

	onMount(() => {
		unsubscribe = onOAuthComplete((result) => {
			isLoading = false;
			if (result.success) {
				onSuccess();
				onClose();
			}
		});
	});

	onDestroy(() => {
		unsubscribe?.();
	});

	// Save OAuth config
	async function handleSaveConfig() {
		if (!clientId.trim() || !clientSecret.trim()) {
			toast({
				title: $_('youtube.modal.error'),
				description: $_('youtube.modal.enterBoth'),
				variant: 'warning'
			});
			return;
		}

		await youtubeAuthStore.setOAuthConfig({
			clientId: clientId.trim(),
			clientSecret: clientSecret.trim(),
			redirectUri: 'sermon-helper://oauth/callback'
		});

		showConfigForm = false;
		toast({
			title: $_('youtube.modal.configSaved'),
			description: $_('youtube.modal.configSavedDescription'),
			variant: 'success'
		});
	}

	// Start OAuth login
	async function handleLogin() {
		const config = youtubeAuthStore.getConfig();
		if (!config) {
			showConfigForm = true;
			return;
		}

		manualAuthUrl = ''; // Reset manual URL
		isLoading = true;
		try {
			// In Tauri mode, this will complete the full OAuth flow and return tokens
			// In web mode, this redirects to Google and returns void
			const result = await youtubeApi.startOAuthFlow();

			if (result) {
				// Tauri mode - we got tokens back, login complete
				isLoading = false;
				toast({
					title: $_('youtube.modal.success') || 'Success',
					description: $_('youtube.modal.loginSuccess') || 'Successfully logged in to YouTube',
					variant: 'success'
				});
				onSuccess();
				onClose();
			}
			// In web mode, the callback page handles the rest
		} catch (error) {
			isLoading = false;
			// If browser failed to open, show the URL for manual copy
			const port = youtubeApi.getCurrentOAuthPort();
			try {
				manualAuthUrl = youtubeApi.getAuthUrl(port || undefined);
			} catch {
				// Ignore if we can't generate URL
			}
			toast({
				title: $_('youtube.modal.error'),
				description: error instanceof Error ? error.message : $_('youtube.modal.loginFailed'),
				variant: 'error'
			});
		}
	}

	// Copy URL to clipboard
	async function copyAuthUrl() {
		try {
			await navigator.clipboard.writeText(manualAuthUrl);
			toast({
				title: 'Copied',
				description: 'URL copied to clipboard',
				variant: 'success'
			});
		} catch {
			toast({
				title: 'Copy failed',
				description: 'Please select and copy the URL manually',
				variant: 'warning'
			});
		}
	}

	function handleDialogClose() {
		onClose();
	}
</script>

<dialog
	bind:this={dialogElement}
	class="fixed z-50 bg-background border rounded-lg shadow-lg max-w-md w-full backdrop:bg-black/50 backdrop:backdrop-blur-sm p-0"
	onclose={handleDialogClose}
>
	<div class="p-6 space-y-4">
		<div class="space-y-2">
			<h2 class="text-lg font-semibold">{$_('youtube.modal.title')}</h2>
			<p class="text-sm text-muted-foreground">
				{$_('youtube.modal.description')}
			</p>
		</div>

		{#if showConfigForm}
			<!-- API Configuration Form -->
			<div class="space-y-4">
				<Alert>
					<AlertCircle class="h-4 w-4" />
					<AlertTitle>{$_('youtube.modal.configRequired')}</AlertTitle>
					<AlertDescription>
						{$_('youtube.modal.configDescription')}
					</AlertDescription>
				</Alert>

				<div class="space-y-2">
					<Label htmlFor="client-id">{$_('youtube.modal.clientId')}</Label>
					<Input
						id="client-id"
						bind:value={clientId}
						placeholder={$_('youtube.modal.clientIdPlaceholder')}
					/>
				</div>

				<div class="space-y-2">
					<Label htmlFor="client-secret">{$_('youtube.modal.clientSecret')}</Label>
					<Input
						id="client-secret"
						type="password"
						bind:value={clientSecret}
						placeholder={$_('youtube.modal.clientSecretPlaceholder')}
					/>
				</div>

				<p class="text-xs text-muted-foreground">
					{$_('youtube.modal.redirectUri')}: <code class="bg-muted px-1 rounded">{getDisplayRedirectUri()}</code>
				</p>

				<div class="flex gap-2">
					<Button buttonVariant="outline" onclick={() => (showConfigForm = false)} className="flex-1">
						{$_('common.cancel')}
					</Button>
					<Button onclick={handleSaveConfig} className="flex-1">
						<Settings class="mr-2 h-4 w-4" />
						{$_('youtube.modal.saveConfig')}
					</Button>
				</div>
			</div>
		{:else}
			<!-- Login Button -->
			<div class="space-y-4">
				<Button onclick={handleLogin} className="w-full" disabled={isLoading}>
					{#if isLoading}
						<Loader2 class="mr-2 h-4 w-4 animate-spin" />
						{$_('youtube.modal.waitingForLogin')}
					{:else}
						<LogIn class="mr-2 h-4 w-4" />
						{$_('youtube.modal.loginButton')}
					{/if}
				</Button>

				{#if manualAuthUrl}
					<!-- Manual URL fallback when browser doesn't open -->
					<Alert>
						<AlertCircle class="h-4 w-4" />
						<AlertTitle>Browser didn't open?</AlertTitle>
						<AlertDescription>
							<p class="mb-2">Copy this URL and paste it in your browser:</p>
							<div class="flex gap-2">
								<code class="flex-1 text-xs bg-muted p-2 rounded break-all max-h-20 overflow-y-auto">
									{manualAuthUrl}
								</code>
								<Button buttonVariant="outline" buttonSize="sm" onclick={copyAuthUrl}>
									<Copy class="h-4 w-4" />
								</Button>
							</div>
						</AlertDescription>
					</Alert>
				{/if}

				<Button buttonVariant="outline" onclick={() => (showConfigForm = true)} className="w-full">
					<Settings class="mr-2 h-4 w-4" />
					{$_('youtube.modal.configureCredentials')}
				</Button>
			</div>
		{/if}

		<div class="flex justify-end pt-4 border-t">
			<Button buttonVariant="ghost" onclick={onClose}>
				{$_('common.close')}
			</Button>
		</div>
	</div>
</dialog>
