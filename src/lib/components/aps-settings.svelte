<script lang="ts">
	import Card from "$lib/components/ui/card.svelte";
	import Button from "$lib/components/ui/button.svelte";
	import Input from "$lib/components/ui/input.svelte";
	import Label from "$lib/components/ui/label.svelte";
	import { toast } from "$lib/utils/toast";
	import { apsClient, connectAPS, disconnectAPS, type APSSettings } from "$lib/utils/aps-api-client";
	import { apsSettingsStore } from "$lib/utils/aps-store";
	import { Settings, Save, TestTube, Wifi, WifiOff, Monitor, Play, Pause, Square } from "lucide-svelte";
	import { onMount, onDestroy } from "svelte";
	import { _ } from 'svelte-i18n';

	let host: string = "127.0.0.1";
	let port: number = 31600;
	let autoConnect: boolean = true;
	let isTesting: boolean = false;
	let isLoading: boolean = true;
	let slideNumber: number = 1;

	// Subscribe to APS status
	let apsStatus = apsClient.apsStatus;
	let presentationInfo = apsClient.currentPresentation;

	let unsubscribeFeedback: (() => void) | null = null;

	onMount(async () => {
		try {
			const settings = await apsSettingsStore.getSettings();
			host = settings.host;
			port = settings.port;
			autoConnect = settings.autoConnect;

			// Subscribe to APS feedback
			unsubscribeFeedback = apsClient.onFeedback((feedback) => {
				console.log('APS Feedback received:', feedback);
			});

			// Auto-connect if enabled
			if (autoConnect) {
				await testConnection();
			}
		} catch (error) {
			console.error("Failed to load APS settings:", error);
			toast({
				title: "Error",
				description: "Failed to load APS settings",
				variant: "error"
			});
		} finally {
			isLoading = false;
		}
	});

	onDestroy(() => {
		if (unsubscribeFeedback) {
			unsubscribeFeedback();
		}
	});

	const testConnection = async (): Promise<boolean> => {
		isTesting = true;
		try {
			const settings: APSSettings = { host, port, autoConnect, timeout: 5000 };
			const result = await connectAPS(settings);
			
			if (result.connected) {
				toast({
					title: "Connection Test",
					description: "Successfully connected to APS",
					variant: "success"
				});
				return true;
			} else {
				toast({
					title: "Connection Test",
					description: result.error || "Failed to connect to APS",
					variant: "error"
				});
				return false;
			}
		} catch (error) {
			console.error("Connection test failed:", error);
			toast({
				title: "Connection Test",
				description: "Connection test failed",
				variant: "error"
			});
			return false;
		} finally {
			isTesting = false;
		}
	};

	const handleTestConnection = async () => {
		await testConnection();
	};

	const handleSaveSettings = async () => {
		try {
			await apsSettingsStore.saveSettings({
				host,
				port,
				autoConnect,
				timeout: 5000
			});

			toast({
				title: "Settings Saved",
				description: "APS settings have been saved successfully",
				variant: "success"
			});
		} catch (error) {
			console.error("Failed to save APS settings:", error);
			toast({
				title: "Error",
				description: "Failed to save APS settings",
				variant: "error"
			});
		}
	};

	const handleDisconnect = async () => {
		disconnectAPS();
		toast({
			title: "Disconnected",
			description: "Disconnected from APS",
			variant: "info"
		});
	};

	// Slide control handlers
	const handleNextSlide = () => {
		apsClient.nextSlide();
	};

	const handlePreviousSlide = () => {
		apsClient.previousSlide();
	};

	const handleGoToSlide = () => {
		if (slideNumber > 0) {
			apsClient.goToSlide(slideNumber);
		}
	};

	const handleClosePresentation = () => {
		apsClient.closePresentation();
	};
</script>

<!-- APS Settings Card -->
<Card>
	<svelte:fragment slot="title">
		<Monitor class="h-5 w-5" />
		Auto Presentation Switcher (APS)
	</svelte:fragment>

	<svelte:fragment slot="description">
		Control PowerPoint, Keynote, and PDF presentations professionally via APS integration.
	</svelte:fragment>

	<svelte:fragment slot="content">
		<div class="space-y-6">
			<!-- Host Input -->
			<div class="space-y-2">
				<Label for="aps-host">Host</Label>
				<Input
					id="aps-host"
					type="text"
					bind:value={host}
					placeholder="127.0.0.1"
					disabled={isLoading}
				/>
				<p class="text-xs text-muted-foreground">
					The hostname or IP address where APS is running (usually localhost)
				</p>
			</div>

			<!-- Port Input -->
			<div class="space-y-2">
				<Label for="aps-port">Port</Label>
				<Input
					id="aps-port"
					type="number"
					bind:value={port}
					placeholder="31600"
					disabled={isLoading}
				/>
				<p class="text-xs text-muted-foreground">
					The TCP port APS is listening on (default: 31600)
				</p>
			</div>

			<!-- Action Buttons -->
			<div class="flex gap-3 pt-2">
				<Button
					buttonVariant="outline"
					onclick={handleTestConnection}
					disabled={$apsStatus.connected || isTesting || isLoading}
					className="flex-1 bg-transparent"
				>
					<TestTube class="mr-2 h-4 w-4" />
					{isTesting ? "Testing..." : "Test Connection"}
				</Button>

				{#if $apsStatus.connected}
					<Button
						buttonVariant="outline"
						onclick={handleDisconnect}
						className="flex-1"
					>
						<Square class="mr-2 h-4 w-4" />
						Disconnect
					</Button>
				{/if}

				<Button
					onclick={handleSaveSettings}
					disabled={isLoading}
					className="flex-1"
				>
					<Save class="mr-2 h-4 w-4" />
					{isLoading ? "Loading..." : "Save Settings"}
				</Button>
			</div>

			<!-- Connection Status -->
			<div class="rounded-lg bg-muted/50 p-4 space-y-3">
				<div class="flex items-center justify-between">
					<h4 class="font-medium text-sm">Connection Status</h4>
					<div class="flex items-center gap-2">
						{#if $apsStatus.connected}
							<Wifi class="h-4 w-4 text-green-600" />
							<span class="text-sm text-green-600 font-medium">Connected</span>
						{:else}
							<WifiOff class="h-4 w-4 text-red-600" />
							<span class="text-sm text-red-600 font-medium">Disconnected</span>
						{/if}
					</div>
				</div>

				{#if $apsStatus.apiVersion}
					<p class="text-xs text-muted-foreground">
						API Version: {$apsStatus.apiVersion}
					</p>
				{/if}

				{#if $apsStatus.lastConnected}
					<p class="text-xs text-muted-foreground">
						Last Connected: {new Date($apsStatus.lastConnected).toLocaleString()}
					</p>
				{/if}

				{#if $apsStatus.error}
					<p class="text-xs text-red-600">
						Error: {$apsStatus.error}
					</p>
				{/if}
			</div>

			<!-- Presentation Info -->
			{#if $apsStatus.connected && ($presentationInfo.currentFile || $presentationInfo.slideNumber)}
				<div class="rounded-lg bg-blue-50 dark:bg-blue-950 p-4 space-y-3">
					<h4 class="font-medium text-sm">Current Presentation</h4>
					
					{#if $presentationInfo.currentFile}
						<p class="text-sm">
							<strong>File:</strong> {$presentationInfo.currentFile}
						</p>
					{/if}
					
					{#if $presentationInfo.slideNumber}
						<p class="text-sm">
							<strong>Slide:</strong> {$presentationInfo.slideNumber} / {$presentationInfo.slidesCount || '?'}
						</p>
					{/if}

					{#if $presentationInfo.powerPointMediaState}
						<p class="text-sm">
							<strong>Media:</strong> {$presentationInfo.powerPointMediaState}
							{#if $presentationInfo.powerPointMediaTimeLeft}
								({$presentationInfo.powerPointMediaTimeLeft} remaining)
							{/if}
						</p>
					{/if}
				</div>
			{/if}

			<!-- Slide Controls -->
			{#if $apsStatus.connected}
				<div class="rounded-lg bg-muted/50 p-4 space-y-4">
					<h4 class="font-medium text-sm">Slide Controls</h4>
					
					<div class="flex gap-2">
						<Button
							buttonVariant="outline"
							onclick={handlePreviousSlide}
							className="flex-1"
						>
							Previous
						</Button>
						<Button
							onclick={handleNextSlide}
							className="flex-1"
						>
							Next
						</Button>
					</div>

					<div class="flex gap-2 items-center">
						<Input
							type="number"
							bind:value={slideNumber}
							placeholder="Slide #"
							className="w-24"
							min="1"
						/>
						<Button
							onclick={handleGoToSlide}
							className="flex-1"
						>
							Go to Slide
						</Button>
					</div>

					<Button
						buttonVariant="outline"
						onclick={handleClosePresentation}
						className="w-full"
					>
						<Square class="mr-2 h-4 w-4" />
						Close Presentation
					</Button>
				</div>
			{/if}

			<!-- Setup Instructions -->
			<div class="rounded-lg bg-muted p-4 space-y-2">
				<h4 class="font-medium text-sm">Setup Instructions</h4>
				<ol class="text-sm text-muted-foreground space-y-1 list-decimal list-inside">
					<li>Download and install APS from presentationtools.com</li>
					<li>Start APS on your presentation computer</li>
					<li>Enable API access in APS settings (port 31600)</li>
					<li>Configure PowerPoint/Keynote file paths</li>
					<li>Test the connection here</li>
					<li>Use Companion module to control presentations remotely</li>
				</ol>
			</div>
		</div>
	</svelte:fragment>
</Card>
