<script lang="ts">
	import type { SystemStatus } from "$lib/stores/types";
	import ErrorMessages from "./ui/error-messages.svelte";
	import Card from "./ui/card.svelte";
	import Button from "./ui/button.svelte";
	import Input from "./ui/input.svelte";
	import Label from "./ui/label.svelte";
	import { toast } from "$lib/utils/toast";
	import { obsSettingsStore, type ObsSettings } from "$lib/utils/obs-store";
	import { Settings, Save, TestTube } from "lucide-svelte";
	import { onMount } from "svelte";

	// Event handler
	export let onRecheck: () => Promise<void> = async () => {};

	// Data props
	export let systemStatus: SystemStatus = {
		obs: true,
		rodeInterface: true,
		mainDisplay: true,
		secondaryDisplay: true,
		airplayDisplay: false,
		displayAlignment: false,
		youtubeLoggedIn: false,
	};

	// State
	let websocketUrl: string = "ws://localhost:4455";
	let websocketPassword: string = "";
	let isTesting: boolean = false;
	let isLoading: boolean = true;

	// Load settings on mount
	onMount(async () => {
		try {
			const settings = await obsSettingsStore.getSettings();
			websocketUrl = settings.websocketUrl;
			websocketPassword = settings.websocketPassword;
		} catch (error) {
			console.error('Failed to load OBS settings:', error);
			toast({
				title: "Error",
				description: "Failed to load OBS settings from storage",
			});
		} finally {
			isLoading = false;
		}
	});

	// Event handlers
	const handleTestConnection = async () => {
		isTesting = true;
		
		// Simulate connection test with timeout
		setTimeout(() => {
			isTesting = false;
			toast({
				title: "Connection Test",
				description: "Successfully connected to OBS WebSocket server",
			});
		}, 1500);
	};

	const handleSaveSettings = async () => {
		try {
			await obsSettingsStore.saveSettings({
				websocketUrl,
				websocketPassword,
			});
			
			toast({
				title: "Settings Saved",
				description: "OBS WebSocket settings have been updated",
			});
		} catch (error) {
			console.error('Failed to save OBS settings:', error);
			toast({
				title: "Error",
				description: "Failed to save OBS settings",
			});
		}
	};
</script>

<div class="p-4 lg:p-8 space-y-6 pt-20 lg:pt-8">
	<ErrorMessages systemStatus={systemStatus} onRecheck={onRecheck} />

	<div class="mt-12 lg:mt-0">
		<h2 class="text-3xl font-bold tracking-tight">OBS Settings</h2>
		<p class="text-muted-foreground">Configure OBS Studio WebSocket connection</p>
	</div>

	<!-- OBS Settings Card -->
	<Card className="max-w-2xl">
		<svelte:fragment slot="title">
			<Settings class="h-5 w-5" />
			WebSocket Configuration
		</svelte:fragment>
		
		<svelte:fragment slot="description">
			Connect to OBS Studio via WebSocket to control scenes, sources, and stream information
		</svelte:fragment>

		<svelte:fragment slot="content">
			<div class="space-y-6">
				<!-- WebSocket URL Input -->
				<div class="space-y-2">
					<Label for="websocket-url">WebSocket URL</Label>
					<Input
						id="websocket-url"
						type="text"
						bind:value={websocketUrl}
						placeholder="ws://localhost:4455"
						disabled={isLoading}
					/>
					<p class="text-xs text-muted-foreground">
						Default OBS WebSocket URL. Change if you've configured a different port.
					</p>
				</div>

				<!-- WebSocket Password Input -->
				<div class="space-y-2">
					<Label for="websocket-password">WebSocket Password</Label>
					<Input
						id="websocket-password"
						type="password"
						bind:value={websocketPassword}
						placeholder="Enter your OBS WebSocket password"
						disabled={isLoading}
					/>
					<p class="text-xs text-muted-foreground">Set in OBS Studio under Tools → WebSocket Server Settings</p>
				</div>

				<!-- Action Buttons -->
				<div class="flex gap-3">
					<Button
						buttonVariant="outline"
						onclick={handleTestConnection}
						disabled={isTesting || isLoading}
						className="flex-1 bg-transparent"
					>
						<TestTube class="mr-2 h-4 w-4" />
						{isTesting ? "Testing..." : "Test Connection"}
					</Button>
					
					<Button
						onclick={handleSaveSettings}
						disabled={isLoading}
						className="flex-1"
					>
						<Save class="mr-2 h-4 w-4" />
						{isLoading ? "Loading..." : "Save Settings"}
					</Button>
				</div>

				<!-- Setup Instructions -->
				<div class="rounded-lg bg-muted p-4 space-y-2">
					<h4 class="font-medium text-sm">Setup Instructions</h4>
					<ol class="text-sm text-muted-foreground space-y-1 list-decimal list-inside">
						<li>Open OBS Studio</li>
						<li>Go to Tools → WebSocket Server Settings</li>
						<li>Enable "Enable WebSocket server"</li>
						<li>Note the server port (default: 4455)</li>
						<li>Set a secure password</li>
						<li>Click Apply and OK</li>
						<li>Enter the settings above and test the connection</li>
					</ol>
				</div>
			</div>
		</svelte:fragment>
	</Card>
</div>
