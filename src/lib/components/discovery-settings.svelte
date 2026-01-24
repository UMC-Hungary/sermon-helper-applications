<script lang="ts">
	import Card from '$lib/components/ui/card.svelte';
	import Button from '$lib/components/ui/button.svelte';
	import Input from '$lib/components/ui/input.svelte';
	import Label from '$lib/components/ui/label.svelte';
	import { toast } from '$lib/utils/toast';
	import { appSettingsStore, appSettings } from '$lib/utils/app-settings-store';
	import {
		discoveryServerStatus,
		discoveryServerLoading,
		discoveryServerError,
		discoveryServerManager
	} from '$lib/stores/discovery-server-store';
	import type { DiscoverySettings } from '$lib/types/discovery';
	import { DEFAULT_DISCOVERY_SETTINGS } from '$lib/types/discovery';
	import {
		Radio,
		Play,
		Square,
		RefreshCw,
		Copy,
		Check,
		Wifi,
		WifiOff,
		Users,
		Shield
	} from 'lucide-svelte';
	import { onMount } from 'svelte';

	let settings: DiscoverySettings = { ...DEFAULT_DISCOVERY_SETTINGS };
	let isLoading = true;
	let isSaving = false;
	let tokenCopied = false;
	let localAddresses: string[] = [];

	onMount(async () => {
		try {
			// Load settings
			const appSettingsValue = await appSettingsStore.get('discoverySettings');
			if (appSettingsValue) {
				settings = { ...DEFAULT_DISCOVERY_SETTINGS, ...appSettingsValue };
			}

			// Initialize the discovery server manager
			await discoveryServerManager.init();

			// Get local addresses
			localAddresses = await discoveryServerManager.getLocalAddresses();
		} catch (error) {
			console.error('Failed to load discovery settings:', error);
		} finally {
			isLoading = false;
		}
	});

	async function handleStartServer() {
		try {
			const info = await discoveryServerManager.start(settings);
			toast({
				title: 'Discovery Server Started',
				description: `Server running on port ${info.port}`,
				variant: 'success'
			});
		} catch (error) {
			const message = error instanceof Error ? error.message : String(error);
			toast({
				title: 'Failed to Start Server',
				description: message,
				variant: 'error'
			});
		}
	}

	async function handleStopServer() {
		try {
			await discoveryServerManager.stop();
			toast({
				title: 'Discovery Server Stopped',
				description: 'The discovery server has been stopped',
				variant: 'success'
			});
		} catch (error) {
			const message = error instanceof Error ? error.message : String(error);
			toast({
				title: 'Failed to Stop Server',
				description: message,
				variant: 'error'
			});
		}
	}

	async function handleSaveSettings() {
		isSaving = true;
		try {
			await appSettingsStore.set('discoverySettings', settings);
			toast({
				title: 'Settings Saved',
				description: 'Discovery settings have been saved',
				variant: 'success'
			});
		} catch (error) {
			console.error('Failed to save discovery settings:', error);
			toast({
				title: 'Failed to Save Settings',
				description: 'Could not save discovery settings',
				variant: 'error'
			});
		} finally {
			isSaving = false;
		}
	}

	async function handleGenerateToken() {
		try {
			const token = await discoveryServerManager.generateAuthToken();
			settings.authToken = token;
			await handleSaveSettings();
			toast({
				title: 'Token Generated',
				description: 'A new authentication token has been generated',
				variant: 'success'
			});
		} catch (error) {
			toast({
				title: 'Failed to Generate Token',
				description: 'Could not generate a new token',
				variant: 'error'
			});
		}
	}

	async function handleCopyToken() {
		if (settings.authToken) {
			await navigator.clipboard.writeText(settings.authToken);
			tokenCopied = true;
			setTimeout(() => {
				tokenCopied = false;
			}, 2000);
		}
	}

	async function refreshAddresses() {
		try {
			localAddresses = await discoveryServerManager.getLocalAddresses();
		} catch (error) {
			console.error('Failed to refresh addresses:', error);
		}
	}
</script>

<Card>
	<svelte:fragment slot="title">
		<Radio class="h-5 w-5" />
		Mobile Companion Discovery
	</svelte:fragment>

	<svelte:fragment slot="description">
		Allow mobile devices to discover and connect to this app on the local network
	</svelte:fragment>

	<svelte:fragment slot="content">
		<div class="space-y-6">
			<!-- Server Status -->
			<div class="rounded-lg bg-muted/50 p-4 space-y-3">
				<div class="flex items-center justify-between">
					<h4 class="font-medium text-sm">Server Status</h4>
					<div class="flex items-center gap-2">
						{#if $discoveryServerStatus.running}
							<Wifi class="h-4 w-4 text-green-600" />
							<span class="text-sm text-green-600 font-medium">Running</span>
						{:else}
							<WifiOff class="h-4 w-4 text-muted-foreground" />
							<span class="text-sm text-muted-foreground font-medium">Stopped</span>
						{/if}
					</div>
				</div>

				{#if $discoveryServerStatus.running}
					<div class="grid grid-cols-2 gap-4 text-sm">
						<div>
							<span class="text-muted-foreground">Port:</span>
							<span class="ml-2 font-mono">{$discoveryServerStatus.port}</span>
						</div>
						<div class="flex items-center gap-1">
							<Users class="h-4 w-4 text-muted-foreground" />
							<span class="text-muted-foreground">Clients:</span>
							<span class="ml-1">{$discoveryServerStatus.connectedClients}</span>
						</div>
					</div>

					{#if $discoveryServerStatus.addresses.length > 0}
						<div class="text-sm">
							<span class="text-muted-foreground">Addresses:</span>
							<div class="mt-1 space-y-1">
								{#each $discoveryServerStatus.addresses as address}
									<code class="block text-xs bg-background px-2 py-1 rounded">
										http://{address}:{$discoveryServerStatus.port}
									</code>
								{/each}
							</div>
						</div>
					{/if}

					{#if $discoveryServerStatus.mdnsRegistered}
						<div class="flex items-center gap-2 text-sm text-green-600">
							<Check class="h-4 w-4" />
							<span>mDNS service registered (discoverable)</span>
						</div>
					{/if}
				{/if}

				{#if $discoveryServerError}
					<p class="text-xs text-red-600">Error: {$discoveryServerError}</p>
				{/if}
			</div>

			<!-- Start/Stop Button -->
			<div class="flex gap-3">
				{#if $discoveryServerStatus.running}
					<Button
						buttonVariant="destructive"
						onclick={handleStopServer}
						disabled={$discoveryServerLoading}
						className="flex-1"
					>
						<Square class="mr-2 h-4 w-4" />
						{$discoveryServerLoading ? 'Stopping...' : 'Stop Server'}
					</Button>
				{:else}
					<Button
						onclick={handleStartServer}
						disabled={$discoveryServerLoading || isLoading}
						className="flex-1"
					>
						<Play class="mr-2 h-4 w-4" />
						{$discoveryServerLoading ? 'Starting...' : 'Start Server'}
					</Button>
				{/if}
			</div>

			<!-- Port Setting -->
			<div class="space-y-2">
				<Label for="discovery-port">Port</Label>
				<Input
					id="discovery-port"
					type="number"
					bind:value={settings.port}
					placeholder="8765"
					disabled={isLoading || $discoveryServerStatus.running}
					min={1024}
					max={65535}
				/>
				<p class="text-xs text-muted-foreground">
					Default is 8765. Change only if there's a conflict.
				</p>
			</div>

			<!-- Instance Name -->
			<div class="space-y-2">
				<Label for="instance-name">Instance Name</Label>
				<Input
					id="instance-name"
					type="text"
					bind:value={settings.instanceName}
					placeholder="Sermon Helper"
					disabled={isLoading || $discoveryServerStatus.running}
				/>
				<p class="text-xs text-muted-foreground">
					The name shown when discovering this device on the network.
				</p>
			</div>

			<!-- Authentication -->
			<div class="rounded-lg border p-4 space-y-4">
				<div class="flex items-center gap-2">
					<Shield class="h-4 w-4" />
					<h4 class="font-medium text-sm">Authentication</h4>
				</div>

				<div class="flex items-center gap-3">
					<input
						type="checkbox"
						id="auth-required"
						bind:checked={settings.authRequired}
						disabled={isLoading || $discoveryServerStatus.running}
						class="h-4 w-4 rounded border-gray-300"
					/>
					<Label for="auth-required" class="text-sm font-normal">
						Require authentication token
					</Label>
				</div>

				{#if settings.authRequired}
					<div class="space-y-2">
						<Label for="auth-token">Auth Token</Label>
						<div class="flex gap-2">
							<Input
								id="auth-token"
								type="text"
								value={settings.authToken || ''}
								placeholder="No token set"
								disabled={true}
								className="font-mono text-sm flex-1"
							/>
							<Button
								buttonVariant="outline"
								onclick={handleCopyToken}
								disabled={!settings.authToken}
								className="px-3"
							>
								{#if tokenCopied}
									<Check class="h-4 w-4" />
								{:else}
									<Copy class="h-4 w-4" />
								{/if}
							</Button>
						</div>
						<Button
							buttonVariant="outline"
							onclick={handleGenerateToken}
							disabled={isLoading || $discoveryServerStatus.running}
							className="w-full"
						>
							<RefreshCw class="mr-2 h-4 w-4" />
							Generate New Token
						</Button>
						<p class="text-xs text-muted-foreground">
							Share this token with your mobile app to allow connections.
						</p>
					</div>
				{/if}
			</div>

			<!-- Local Addresses -->
			<div class="rounded-lg bg-muted p-4 space-y-2">
				<div class="flex items-center justify-between">
					<h4 class="font-medium text-sm">Local Network Addresses</h4>
					<Button buttonVariant="ghost" onclick={refreshAddresses} className="h-8 w-8 p-0">
						<RefreshCw class="h-4 w-4" />
					</Button>
				</div>
				{#if localAddresses.length > 0}
					<div class="space-y-1">
						{#each localAddresses as address}
							<code class="block text-xs bg-background px-2 py-1 rounded">{address}</code>
						{/each}
					</div>
				{:else}
					<p class="text-sm text-muted-foreground">No local addresses found</p>
				{/if}
			</div>

			<!-- Save Button -->
			<Button onclick={handleSaveSettings} disabled={isLoading || isSaving} className="w-full">
				{isSaving ? 'Saving...' : 'Save Settings'}
			</Button>
		</div>
	</svelte:fragment>
</Card>
