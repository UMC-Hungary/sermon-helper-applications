<script lang="ts">
	import Card from "$lib/components/ui/card.svelte";
	import Button from "$lib/components/ui/button.svelte";
	import Input from "$lib/components/ui/input.svelte";
	import Label from "$lib/components/ui/label.svelte";
	import Checkbox from "$lib/components/ui/checkbox.svelte";
	import { toast } from "$lib/utils/toast";
	import { obsWebSocket } from "$lib/utils/obs-websocket";
	import { obsStatus } from "$lib/stores/system-store";
	import {
		Plus,
		Trash2,
		RefreshCw,
		Monitor,
		Volume2,
		Globe,
		Save,
		AlertCircle,
		CheckCircle2
	} from "lucide-svelte";
	import { onMount } from "svelte";
	import { _ } from "svelte-i18n";
	import {
		deviceConfigs,
		browserSourceConfigs,
		obsDevicesStore
	} from "$lib/stores/obs-devices-store";
	import {
		obsDeviceStatuses,
		obsBrowserStatuses,
		isCheckingDevices
	} from "$lib/stores/obs-device-status-store";
	import { checkAllObsDevices } from "$lib/utils/obs-device-checker";
	import type {
		ObsDevice,
		ObsInputInfo,
		ObsDeviceConfig,
		ObsBrowserSourceConfig,
		ObsDeviceType
	} from "$lib/types/obs-devices";

	// Available devices and sources from OBS
	let availableDisplays: ObsDevice[] = [];
	let availableAudioDevices: ObsDevice[] = [];
	let availableSources: ObsInputInfo[] = [];
	let availableBrowserSources: ObsInputInfo[] = [];

	// Loading states
	let isLoadingObs = false;
	let isSaving = false;

	// New device/source forms
	let newDisplayConfig: Partial<ObsDeviceConfig> = {
		type: "display",
		name: "",
		required: false,
		targetSourceName: "",
		expectedValue: ""
	};

	let newAudioConfig: Partial<ObsDeviceConfig> = {
		type: "audio",
		name: "",
		required: false,
		targetSourceName: "",
		expectedValue: ""
	};

	let newBrowserConfig: Partial<ObsBrowserSourceConfig> = {
		name: "",
		targetSourceName: "",
		urlTemplate: ""
	};

	// Show add forms
	let showAddDisplay = false;
	let showAddAudio = false;
	let showAddBrowser = false;

	onMount(async () => {
		if ($obsStatus.connected) {
			await loadObsData();
		}
	});

	// Load available devices and sources from OBS
	async function loadObsData() {
		if (!$obsStatus.connected) {
			toast({
				title: $_("toasts.error.title"),
				description: $_("obsDevices.errors.obsNotConnected"),
				variant: "error"
			});
			return;
		}

		isLoadingObs = true;
		try {
			// Get all inputs
			availableSources = await obsWebSocket.getInputList();

			// Filter for screen capture sources
			const screenCaptureSources = availableSources.filter(
				(s) => s.inputKind === "screen_capture" || s.inputKind === "monitor_capture"
			);

			// Filter for audio sources
			const audioSources = availableSources.filter(
				(s) =>
					s.inputKind === "coreaudio_input_capture" ||
					s.inputKind === "coreaudio_output_capture" ||
					s.inputKind === "wasapi_input_capture" ||
					s.inputKind === "wasapi_output_capture" ||
					s.inputKind === "pulse_input_capture" ||
					s.inputKind === "pulse_output_capture" ||
					s.inputKind === "alsa_input_capture"
			);

			// Filter for browser sources
			availableBrowserSources = availableSources.filter(
				(s) => s.inputKind === "browser_source"
			);

			// Get available displays from first screen capture source (if any)
			if (screenCaptureSources.length > 0) {
				try {
					availableDisplays = await obsWebSocket.getInputPropertyItems(
						screenCaptureSources[0].inputName,
						"display_uuid"
					);
				} catch {
					availableDisplays = [];
				}
			}

			// Get available audio devices from first audio source (if any)
			if (audioSources.length > 0) {
				try {
					availableAudioDevices = await obsWebSocket.getInputPropertyItems(
						audioSources[0].inputName,
						"device_id"
					);
				} catch {
					availableAudioDevices = [];
				}
			}

			toast({
				title: $_("obsDevices.toasts.dataLoaded"),
				description: $_("obsDevices.toasts.dataLoadedDescription"),
				variant: "success"
			});
		} catch (error) {
			console.error("Failed to load OBS data:", error);
			toast({
				title: $_("toasts.error.title"),
				description: error instanceof Error ? error.message : "Failed to load OBS data",
				variant: "error"
			});
		} finally {
			isLoadingObs = false;
		}
	}

	// Add a new display configuration
	async function addDisplayConfig() {
		if (!newDisplayConfig.name || !newDisplayConfig.targetSourceName || !newDisplayConfig.expectedValue) {
			toast({
				title: $_("toasts.error.title"),
				description: $_("obsDevices.errors.fillAllFields"),
				variant: "error"
			});
			return;
		}

		isSaving = true;
		try {
			await obsDevicesStore.addDevice(
				"display",
				newDisplayConfig.name,
				newDisplayConfig.targetSourceName,
				newDisplayConfig.expectedValue,
				newDisplayConfig.required
			);

			newDisplayConfig = {
				type: "display",
				name: "",
				required: false,
				targetSourceName: "",
				expectedValue: ""
			};
			showAddDisplay = false;

			toast({
				title: $_("obsDevices.toasts.deviceAdded"),
				variant: "success"
			});

			// Trigger a device check
			await checkAllObsDevices();
		} catch (error) {
			toast({
				title: $_("toasts.error.title"),
				description: error instanceof Error ? error.message : "Failed to add device",
				variant: "error"
			});
		} finally {
			isSaving = false;
		}
	}

	// Add a new audio configuration
	async function addAudioConfig() {
		if (!newAudioConfig.name || !newAudioConfig.targetSourceName || !newAudioConfig.expectedValue) {
			toast({
				title: $_("toasts.error.title"),
				description: $_("obsDevices.errors.fillAllFields"),
				variant: "error"
			});
			return;
		}

		isSaving = true;
		try {
			await obsDevicesStore.addDevice(
				"audio",
				newAudioConfig.name,
				newAudioConfig.targetSourceName,
				newAudioConfig.expectedValue,
				newAudioConfig.required
			);

			newAudioConfig = {
				type: "audio",
				name: "",
				required: false,
				targetSourceName: "",
				expectedValue: ""
			};
			showAddAudio = false;

			toast({
				title: $_("obsDevices.toasts.deviceAdded"),
				variant: "success"
			});

			await checkAllObsDevices();
		} catch (error) {
			toast({
				title: $_("toasts.error.title"),
				description: error instanceof Error ? error.message : "Failed to add device",
				variant: "error"
			});
		} finally {
			isSaving = false;
		}
	}

	// Add a new browser source configuration
	async function addBrowserConfig() {
		if (!newBrowserConfig.name || !newBrowserConfig.targetSourceName || !newBrowserConfig.urlTemplate) {
			toast({
				title: $_("toasts.error.title"),
				description: $_("obsDevices.errors.fillAllFields"),
				variant: "error"
			});
			return;
		}

		isSaving = true;
		try {
			await obsDevicesStore.addBrowserSource(
				newBrowserConfig.name,
				newBrowserConfig.targetSourceName,
				newBrowserConfig.urlTemplate
			);

			newBrowserConfig = {
				name: "",
				targetSourceName: "",
				urlTemplate: ""
			};
			showAddBrowser = false;

			toast({
				title: $_("obsDevices.toasts.browserAdded"),
				variant: "success"
			});

			await checkAllObsDevices();
		} catch (error) {
			toast({
				title: $_("toasts.error.title"),
				description: error instanceof Error ? error.message : "Failed to add browser source",
				variant: "error"
			});
		} finally {
			isSaving = false;
		}
	}

	// Remove a device configuration
	async function removeDevice(id: string) {
		try {
			await obsDevicesStore.removeDevice(id);
			toast({
				title: $_("obsDevices.toasts.deviceRemoved"),
				variant: "success"
			});
		} catch (error) {
			toast({
				title: $_("toasts.error.title"),
				description: error instanceof Error ? error.message : "Failed to remove device",
				variant: "error"
			});
		}
	}

	// Remove a browser source configuration
	async function removeBrowserSource(id: string) {
		try {
			await obsDevicesStore.removeBrowserSource(id);
			toast({
				title: $_("obsDevices.toasts.browserRemoved"),
				variant: "success"
			});
		} catch (error) {
			toast({
				title: $_("toasts.error.title"),
				description: error instanceof Error ? error.message : "Failed to remove browser source",
				variant: "error"
			});
		}
	}

	// Toggle required status
	async function toggleRequired(device: ObsDeviceConfig) {
		try {
			await obsDevicesStore.updateDevice(device.id, { required: !device.required });
		} catch (error) {
			toast({
				title: $_("toasts.error.title"),
				description: error instanceof Error ? error.message : "Failed to update device",
				variant: "error"
			});
		}
	}

	// Get display sources
	$: displaySources = availableSources.filter(
		(s) => s.inputKind === "screen_capture" || s.inputKind === "monitor_capture"
	);

	// Get audio sources
	$: audioSources = availableSources.filter(
		(s) =>
			s.inputKind === "coreaudio_input_capture" ||
			s.inputKind === "coreaudio_output_capture" ||
			s.inputKind === "wasapi_input_capture" ||
			s.inputKind === "wasapi_output_capture" ||
			s.inputKind === "pulse_input_capture" ||
			s.inputKind === "pulse_output_capture" ||
			s.inputKind === "alsa_input_capture"
	);
</script>

<div class="mt-12 lg:mt-0">
	<h2 class="text-3xl font-bold tracking-tight">{$_("obsDevices.title")}</h2>
	<p class="text-muted-foreground">{$_("obsDevices.subtitle")}</p>
</div>

<!-- Refresh Data Button -->
<div class="flex gap-3 mb-6">
	<Button
		buttonVariant="outline"
		onclick={loadObsData}
		disabled={!$obsStatus.connected || isLoadingObs}
	>
		<RefreshCw class="mr-2 h-4 w-4 {isLoadingObs ? 'animate-spin' : ''}" />
		{isLoadingObs ? $_("obsDevices.loadingObs") : $_("obsDevices.refreshFromObs")}
	</Button>

	<Button
		buttonVariant="outline"
		onclick={() => checkAllObsDevices()}
		disabled={!$obsStatus.connected || $isCheckingDevices}
	>
		<RefreshCw class="mr-2 h-4 w-4 {$isCheckingDevices ? 'animate-spin' : ''}" />
		{$isCheckingDevices ? $_("obsDevices.checking") : $_("obsDevices.recheckAll")}
	</Button>
</div>

{#if !$obsStatus.connected}
	<Card className="max-w-2xl">
		<svelte:fragment slot="content">
			<div class="flex items-center gap-3 text-amber-600">
				<AlertCircle class="h-5 w-5" />
				<p>{$_("obsDevices.errors.obsNotConnected")}</p>
			</div>
			<Button href="/obs-settings" className="mt-4">
				{$_("obsDevices.goToObsSettings")}
			</Button>
		</svelte:fragment>
	</Card>
{:else}
	<!-- Displays Section -->
	<Card className="max-w-3xl mb-6">
		<svelte:fragment slot="title">
			<Monitor class="h-5 w-5" />
			{$_("obsDevices.displays.title")}
		</svelte:fragment>

		<svelte:fragment slot="description">
			{$_("obsDevices.displays.description")}
		</svelte:fragment>

		<svelte:fragment slot="content">
			<div class="space-y-4">
				<!-- Existing display configs -->
				{#each $deviceConfigs.filter((d) => d.type === "display") as device (device.id)}
					{@const status = $obsDeviceStatuses.get(device.id)}
					<div class="flex items-center justify-between p-3 rounded-lg bg-muted/50">
						<div class="flex items-center gap-3">
							<div class="flex items-center gap-2">
								<Checkbox checked={device.required} onchange={() => toggleRequired(device)} />
								<span class="text-sm text-muted-foreground">{$_("obsDevices.required")}</span>
							</div>
							<div>
								<p class="font-medium">{device.name}</p>
								<p class="text-xs text-muted-foreground">
									{$_("obsDevices.source")}: {device.targetSourceName}
								</p>
							</div>
						</div>
						<div class="flex items-center gap-2">
							{#if status?.available && status?.assigned}
								<CheckCircle2 class="h-4 w-4 text-green-600" />
							{:else if status}
								<AlertCircle class="h-4 w-4 text-red-600" />
							{/if}
							<Button buttonVariant="ghost" buttonSize="icon" onclick={() => removeDevice(device.id)}>
								<Trash2 class="h-4 w-4" />
							</Button>
						</div>
					</div>
				{/each}

				<!-- Add new display form -->
				{#if showAddDisplay}
					<div class="p-4 rounded-lg border space-y-4">
						<div class="grid grid-cols-2 gap-4">
							<div class="space-y-2">
								<Label>{$_("obsDevices.form.name")}</Label>
								<Input bind:value={newDisplayConfig.name} placeholder={$_("obsDevices.form.namePlaceholder")} />
							</div>
							<div class="space-y-2">
								<Label>{$_("obsDevices.form.obsSource")}</Label>
								<select
									bind:value={newDisplayConfig.targetSourceName}
									class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
								>
									<option value="">{$_("obsDevices.form.selectSource")}</option>
									{#each displaySources as source}
										<option value={source.inputName}>{source.inputName}</option>
									{/each}
								</select>
							</div>
						</div>
						<div class="space-y-2">
							<Label>{$_("obsDevices.form.device")}</Label>
							<select
								bind:value={newDisplayConfig.expectedValue}
								class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
							>
								<option value="">{$_("obsDevices.form.selectDevice")}</option>
								{#each availableDisplays as display}
									<option value={display.itemValue}>{display.itemName}</option>
								{/each}
							</select>
						</div>
						<div class="flex items-center gap-2">
							<Checkbox bind:checked={newDisplayConfig.required} />
							<span class="text-sm">{$_("obsDevices.form.markRequired")}</span>
						</div>
						<div class="flex gap-2">
							<Button onclick={addDisplayConfig} disabled={isSaving}>
								<Save class="mr-2 h-4 w-4" />
								{$_("obsDevices.form.save")}
							</Button>
							<Button buttonVariant="outline" onclick={() => (showAddDisplay = false)}>
								{$_("obsDevices.form.cancel")}
							</Button>
						</div>
					</div>
				{:else}
					<Button buttonVariant="outline" onclick={() => (showAddDisplay = true)}>
						<Plus class="mr-2 h-4 w-4" />
						{$_("obsDevices.displays.add")}
					</Button>
				{/if}
			</div>
		</svelte:fragment>
	</Card>

	<!-- Audio Devices Section -->
	<Card className="max-w-3xl mb-6">
		<svelte:fragment slot="title">
			<Volume2 class="h-5 w-5" />
			{$_("obsDevices.audio.title")}
		</svelte:fragment>

		<svelte:fragment slot="description">
			{$_("obsDevices.audio.description")}
		</svelte:fragment>

		<svelte:fragment slot="content">
			<div class="space-y-4">
				<!-- Existing audio configs -->
				{#each $deviceConfigs.filter((d) => d.type === "audio") as device (device.id)}
					{@const status = $obsDeviceStatuses.get(device.id)}
					<div class="flex items-center justify-between p-3 rounded-lg bg-muted/50">
						<div class="flex items-center gap-3">
							<div class="flex items-center gap-2">
								<Checkbox checked={device.required} onchange={() => toggleRequired(device)} />
								<span class="text-sm text-muted-foreground">{$_("obsDevices.required")}</span>
							</div>
							<div>
								<p class="font-medium">{device.name}</p>
								<p class="text-xs text-muted-foreground">
									{$_("obsDevices.source")}: {device.targetSourceName}
								</p>
							</div>
						</div>
						<div class="flex items-center gap-2">
							{#if status?.available && status?.assigned}
								<CheckCircle2 class="h-4 w-4 text-green-600" />
							{:else if status}
								<AlertCircle class="h-4 w-4 text-red-600" />
							{/if}
							<Button buttonVariant="ghost" buttonSize="icon" onclick={() => removeDevice(device.id)}>
								<Trash2 class="h-4 w-4" />
							</Button>
						</div>
					</div>
				{/each}

				<!-- Add new audio form -->
				{#if showAddAudio}
					<div class="p-4 rounded-lg border space-y-4">
						<div class="grid grid-cols-2 gap-4">
							<div class="space-y-2">
								<Label>{$_("obsDevices.form.name")}</Label>
								<Input bind:value={newAudioConfig.name} placeholder={$_("obsDevices.form.namePlaceholder")} />
							</div>
							<div class="space-y-2">
								<Label>{$_("obsDevices.form.obsSource")}</Label>
								<select
									bind:value={newAudioConfig.targetSourceName}
									class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
								>
									<option value="">{$_("obsDevices.form.selectSource")}</option>
									{#each audioSources as source}
										<option value={source.inputName}>{source.inputName}</option>
									{/each}
								</select>
							</div>
						</div>
						<div class="space-y-2">
							<Label>{$_("obsDevices.form.device")}</Label>
							<select
								bind:value={newAudioConfig.expectedValue}
								class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
							>
								<option value="">{$_("obsDevices.form.selectDevice")}</option>
								{#each availableAudioDevices as device}
									<option value={device.itemValue}>{device.itemName}</option>
								{/each}
							</select>
						</div>
						<div class="flex items-center gap-2">
							<Checkbox bind:checked={newAudioConfig.required} />
							<span class="text-sm">{$_("obsDevices.form.markRequired")}</span>
						</div>
						<div class="flex gap-2">
							<Button onclick={addAudioConfig} disabled={isSaving}>
								<Save class="mr-2 h-4 w-4" />
								{$_("obsDevices.form.save")}
							</Button>
							<Button buttonVariant="outline" onclick={() => (showAddAudio = false)}>
								{$_("obsDevices.form.cancel")}
							</Button>
						</div>
					</div>
				{:else}
					<Button buttonVariant="outline" onclick={() => (showAddAudio = true)}>
						<Plus class="mr-2 h-4 w-4" />
						{$_("obsDevices.audio.add")}
					</Button>
				{/if}
			</div>
		</svelte:fragment>
	</Card>

	<!-- Browser Sources Section -->
	<Card className="max-w-3xl">
		<svelte:fragment slot="title">
			<Globe class="h-5 w-5" />
			{$_("obsDevices.browser.title")}
		</svelte:fragment>

		<svelte:fragment slot="description">
			{$_("obsDevices.browser.description")}
		</svelte:fragment>

		<svelte:fragment slot="content">
			<div class="space-y-4">
				<!-- Existing browser source configs -->
				{#each $browserSourceConfigs as config (config.id)}
					{@const status = $obsBrowserStatuses.get(config.id)}
					<div class="p-3 rounded-lg bg-muted/50 space-y-2">
						<div class="flex items-center justify-between">
							<div>
								<p class="font-medium">{config.name}</p>
								<p class="text-xs text-muted-foreground">
									{$_("obsDevices.source")}: {config.targetSourceName}
								</p>
							</div>
							<div class="flex items-center gap-2">
								{#if status?.matches}
									<CheckCircle2 class="h-4 w-4 text-green-600" />
								{:else if status}
									<AlertCircle class="h-4 w-4 text-amber-600" />
								{/if}
								<Button
									buttonVariant="ghost"
									buttonSize="icon"
									onclick={() => removeBrowserSource(config.id)}
								>
									<Trash2 class="h-4 w-4" />
								</Button>
							</div>
						</div>
						<div class="text-xs font-mono bg-background p-2 rounded overflow-x-auto">
							{config.urlTemplate}
						</div>
					</div>
				{/each}

				<!-- Add new browser source form -->
				{#if showAddBrowser}
					<div class="p-4 rounded-lg border space-y-4">
						<div class="grid grid-cols-2 gap-4">
							<div class="space-y-2">
								<Label>{$_("obsDevices.form.name")}</Label>
								<Input
									bind:value={newBrowserConfig.name}
									placeholder={$_("obsDevices.form.namePlaceholder")}
								/>
							</div>
							<div class="space-y-2">
								<Label>{$_("obsDevices.form.obsSource")}</Label>
								<select
									bind:value={newBrowserConfig.targetSourceName}
									class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
								>
									<option value="">{$_("obsDevices.form.selectSource")}</option>
									{#each availableBrowserSources as source}
										<option value={source.inputName}>{source.inputName}</option>
									{/each}
								</select>
							</div>
						</div>
						<div class="space-y-2">
							<Label>{$_("obsDevices.form.urlTemplate")}</Label>
							<Input
								bind:value={newBrowserConfig.urlTemplate}
								placeholder={"http://example.com/${textus}?lekcio=${lekcio}"}
							/>
							<p class="text-xs text-muted-foreground">
								{$_("obsDevices.form.urlTemplateHint")}
							</p>
						</div>
						<div class="flex gap-2">
							<Button onclick={addBrowserConfig} disabled={isSaving}>
								<Save class="mr-2 h-4 w-4" />
								{$_("obsDevices.form.save")}
							</Button>
							<Button buttonVariant="outline" onclick={() => (showAddBrowser = false)}>
								{$_("obsDevices.form.cancel")}
							</Button>
						</div>
					</div>
				{:else}
					<Button buttonVariant="outline" onclick={() => (showAddBrowser = true)}>
						<Plus class="mr-2 h-4 w-4" />
						{$_("obsDevices.browser.add")}
					</Button>
				{/if}
			</div>
		</svelte:fragment>
	</Card>
{/if}
