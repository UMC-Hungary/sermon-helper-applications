<script lang="ts">
	import Card from '$lib/components/ui/card.svelte';
	import Button from '$lib/components/ui/button.svelte';
	import Input from '$lib/components/ui/input.svelte';
	import Label from '$lib/components/ui/label.svelte';
	import { toast } from '$lib/utils/toast';
	import { appSettingsStore } from '$lib/utils/app-settings-store';
	import {
		rfIrSettings,
		rfIrStore,
		rfIrDevices,
		rfIrCommands,
		commandsByCategory,
		commandCategories
	} from '$lib/stores/rf-ir-store';
	import { broadlinkService } from '$lib/utils/broadlink-service';
	import type { RfIrSettings, BroadlinkDevice, RfIrCommand } from '$lib/types/rf-ir';
	import { COMMAND_CATEGORIES } from '$lib/types/rf-ir';
	import RfIrDeviceList from './rf-ir-device-list.svelte';
	import RfIrCommandList from './rf-ir-command-list.svelte';
	import RfIrLearnDialog from './rf-ir-learn-dialog.svelte';
	import RfIrCodeEntryDialog from './rf-ir-code-entry-dialog.svelte';
	import RfIrImport from './rf-ir-import.svelte';
	import {
		Radio,
		Power,
		RefreshCw,
		Plus,
		FileCode,
		Settings2,
		Loader2,
		Upload
	} from 'lucide-svelte';
	import { onMount } from 'svelte';

	let isLoading = true;
	let isSaving = false;
	let isDiscovering = false;
	let showDebug = false;
	let networkInterfaces: Array<[string, string]> = [];

	// Dialog states
	let showLearnDialog = false;
	let showCodeEntryDialog = false;
	let showImportDialog = false;

	// Settings form values
	let enabled = false;
	let autoDiscovery = true;
	let discoveryTimeout = 5;

	async function loadNetworkInterfaces() {
		try {
			networkInterfaces = await broadlinkService.listNetworkInterfaces();
		} catch (error) {
			console.error('Failed to load network interfaces:', error);
		}
	}

	onMount(async () => {
		try {
			await rfIrStore.load();
			enabled = $rfIrSettings.enabled;
			autoDiscovery = $rfIrSettings.autoDiscovery;
			discoveryTimeout = $rfIrSettings.discoveryTimeout;
		} catch (error) {
			console.error('Failed to load RF/IR settings:', error);
		} finally {
			isLoading = false;
		}
	});

	async function handleSaveSettings() {
		isSaving = true;
		try {
			await rfIrStore.update({
				enabled,
				autoDiscovery,
				discoveryTimeout
			});
			toast({
				title: 'Settings Saved',
				description: 'RF/IR settings have been saved',
				variant: 'success'
			});
		} catch (error) {
			console.error('Failed to save RF/IR settings:', error);
			toast({
				title: 'Failed to Save Settings',
				description: 'Could not save RF/IR settings',
				variant: 'error'
			});
		} finally {
			isSaving = false;
		}
	}

	async function handleDiscoverDevices() {
		isDiscovering = true;
		try {
			// Also load network interfaces for debugging
			loadNetworkInterfaces();

			const result = await broadlinkService.discoverAndAddDevices(discoveryTimeout);
			if (result.found === 0) {
				toast({
					title: 'No Devices Found',
					description: 'Try adding your Broadlink device manually using its IP and MAC address (found in your router admin page)',
					variant: 'warning',
					duration: 8000
				});
				// Auto-expand debug section to help troubleshooting
				showDebug = true;
			} else {
				toast({
					title: 'Discovery Complete',
					description: `Found ${result.found} device(s): ${result.added} added, ${result.updated} updated`,
					variant: 'success'
				});
			}
		} catch (error) {
			const message = error instanceof Error ? error.message : String(error);
			toast({
				title: 'Discovery Failed',
				description: `${message}. Try adding the device manually instead.`,
				variant: 'error',
				duration: 8000
			});
			// Auto-expand debug section to help troubleshooting
			showDebug = true;
			loadNetworkInterfaces();
		} finally {
			isDiscovering = false;
		}
	}

	function handleLearnCommand() {
		if ($rfIrDevices.length === 0) {
			toast({
				title: 'No Devices',
				description: 'Please add a Broadlink device first',
				variant: 'warning'
			});
			return;
		}
		showLearnDialog = true;
	}

	function handleAddManualCode() {
		if ($rfIrDevices.length === 0) {
			toast({
				title: 'No Devices',
				description: 'Please add a Broadlink device first',
				variant: 'warning'
			});
			return;
		}
		showCodeEntryDialog = true;
	}

	async function handleCommandSaved(event: CustomEvent<{ slug: string }>) {
		toast({
			title: 'Command Saved',
			description: `Command saved with slug: ${event.detail.slug}`,
			variant: 'success'
		});
	}
</script>

<Card>
	<svelte:fragment slot="title">
		<Radio class="h-5 w-5" />
		RF/IR Remote Control
	</svelte:fragment>

	<svelte:fragment slot="description">
		Configure Broadlink devices for IR/RF control of projectors, screens, and more
	</svelte:fragment>

	<svelte:fragment slot="content">
		{#if isLoading}
			<div class="flex items-center justify-center py-8">
				<Loader2 class="h-6 w-6 animate-spin text-muted-foreground" />
			</div>
		{:else}
			<div class="space-y-6">
				<!-- Enable/Disable Toggle -->
				<div class="flex items-center gap-3">
					<input
						type="checkbox"
						id="rfir-enabled"
						bind:checked={enabled}
						class="h-4 w-4 rounded border-gray-300"
					/>
					<Label for="rfir-enabled" class="text-sm font-normal">
						Enable RF/IR remote control
					</Label>
				</div>

				{#if enabled}
					<!-- Discovery Settings -->
					<div class="rounded-lg border p-4 space-y-4">
						<div class="flex items-center gap-2">
							<Settings2 class="h-4 w-4" />
							<h4 class="font-medium text-sm">Discovery Settings</h4>
						</div>

						<div class="flex items-center gap-3">
							<input
								type="checkbox"
								id="auto-discovery"
								bind:checked={autoDiscovery}
								class="h-4 w-4 rounded border-gray-300"
							/>
							<Label for="auto-discovery" class="text-sm font-normal">
								Auto-discover devices on startup
							</Label>
						</div>

						<div class="space-y-2">
							<Label for="discovery-timeout">Discovery Timeout (seconds)</Label>
							<Input
								id="discovery-timeout"
								type="number"
								bind:value={discoveryTimeout}
								min={1}
								max={30}
								class="w-24"
							/>
						</div>

						<Button
							buttonVariant="outline"
							onclick={handleDiscoverDevices}
							disabled={isDiscovering}
							className="w-full"
						>
							{#if isDiscovering}
								<Loader2 class="mr-2 h-4 w-4 animate-spin" />
								Discovering...
							{:else}
								<RefreshCw class="mr-2 h-4 w-4" />
								Discover Devices
							{/if}
						</Button>

						<!-- Debug: Network Interfaces -->
						<div class="pt-2 border-t">
							<button
								type="button"
								class="text-xs text-muted-foreground hover:text-foreground"
								onclick={() => { showDebug = !showDebug; if (showDebug) loadNetworkInterfaces(); }}
							>
								{showDebug ? '▼' : '▶'} Debug: Network Interfaces
							</button>
							{#if showDebug}
								<div class="mt-2 p-2 bg-muted rounded text-xs font-mono space-y-1">
									{#if networkInterfaces.length === 0}
										<p class="text-muted-foreground">No interfaces found or loading...</p>
									{:else}
										{#each networkInterfaces as [name, ip]}
											<div class="flex justify-between">
												<span class="text-muted-foreground">{name}:</span>
												<span>{ip}</span>
											</div>
										{/each}
									{/if}
								</div>
							{/if}
						</div>
					</div>

					<!-- Device List -->
					<RfIrDeviceList />

					<!-- Commands Section -->
					<div class="rounded-lg border p-4 space-y-4">
						<div class="flex items-center justify-between">
							<div class="flex items-center gap-2">
								<Power class="h-4 w-4" />
								<h4 class="font-medium text-sm">Commands ({$rfIrCommands.length})</h4>
							</div>
							<div class="flex gap-2">
								<Button
									buttonVariant="outline"
									buttonSize="sm"
									onclick={() => showImportDialog = true}
									disabled={$rfIrDevices.length === 0}
								>
									<Upload class="mr-1 h-3 w-3" />
									Import
								</Button>
								<Button
									buttonVariant="outline"
									buttonSize="sm"
									onclick={handleAddManualCode}
									disabled={$rfIrDevices.length === 0}
								>
									<FileCode class="mr-1 h-3 w-3" />
									Add Code
								</Button>
								<Button
									buttonSize="sm"
									onclick={handleLearnCommand}
									disabled={$rfIrDevices.length === 0}
								>
									<Plus class="mr-1 h-3 w-3" />
									Learn
								</Button>
							</div>
						</div>

						<RfIrCommandList />
					</div>
				{/if}

				<!-- Save Button -->
				<Button onclick={handleSaveSettings} disabled={isLoading || isSaving} className="w-full">
					{isSaving ? 'Saving...' : 'Save Settings'}
				</Button>
			</div>
		{/if}
	</svelte:fragment>
</Card>

<!-- Learn Dialog -->
<RfIrLearnDialog
	bind:open={showLearnDialog}
	on:saved={handleCommandSaved}
/>

<!-- Code Entry Dialog -->
<RfIrCodeEntryDialog
	bind:open={showCodeEntryDialog}
	on:saved={handleCommandSaved}
/>

<!-- Import Dialog -->
<RfIrImport
	bind:open={showImportDialog}
	on:imported={(e) => {
		toast({
			title: 'Import Complete',
			description: `Imported ${e.detail.count} commands`,
			variant: 'success'
		});
	}}
/>
