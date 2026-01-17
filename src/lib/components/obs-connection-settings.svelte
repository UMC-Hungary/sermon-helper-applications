<script lang="ts">
	import Card from "$lib/components/ui/card.svelte";
	import Button from "$lib/components/ui/button.svelte";
	import Input from "$lib/components/ui/input.svelte";
	import Label from "$lib/components/ui/label.svelte";
	import { toast } from "$lib/utils/toast";
	import { obsWebSocket } from "$lib/utils/obs-websocket";
	import { obsStatus } from "$lib/stores/system-store";
	import { obsSettingsStore } from "$lib/utils/obs-store";
	import { Settings, Save, TestTube, Wifi, WifiOff } from "lucide-svelte";
	import { onMount } from "svelte";
	import { _ } from 'svelte-i18n';

	let websocketUrl: string = "ws://localhost:4455";
	let websocketPassword: string = "";
	let isTesting: boolean = false;
	let isLoading: boolean = true;

	onMount(async () => {
		try {
			const settings = await obsSettingsStore.getSettings();
			websocketUrl = settings.websocketUrl;
			websocketPassword = settings.websocketPassword;
		} catch (error) {
			console.error("Failed to load OBS settings:", error);
			toast({
				title: $_('toasts.error.title'),
				description: $_('toasts.error.loadSettings'),
				variant: "error",
				duration: 100000
			});
		} finally {
			isLoading = false;
		}
	});

	// Event handlers
	const handleTestConnection = async () => {
		isTesting = true;

		try {
			const result = await obsWebSocket.connect(websocketUrl, websocketPassword).then();

			toast({
				title: $_('toasts.connectionTest.title'),
				description: $_('toasts.connectionTest.success'),
				variant: "success",
				duration: 100000
			});
		} catch (error) {
			console.error("Connection test failed:", error);
			toast({
				title: $_('toasts.connectionTest.title'),
				description: $_('toasts.error.connectionFailed'),
				variant: "error",
				duration: 100000
			});
		} finally {
			isTesting = false;
		}
	};

	const handleSaveSettings = async () => {
		try {
			await obsWebSocket.disconnect();
			await obsSettingsStore.saveSettings({
				websocketUrl,
				websocketPassword,
			});

			toast({
				title: $_('toasts.settingsSaved.title'),
				description: $_('toasts.settingsSaved.description'),
				variant: "success",
				duration: 100000
			});

			setTimeout(async () => {
				const reconnectResult = await obsWebSocket.connect(websocketUrl, websocketPassword);
				if (!reconnectResult.connected) {
					toast({
						title: $_('toasts.error.title'),
						description: reconnectResult.error || $_('toasts.error.reconnectFailed'),
						variant: "error",
						duration: 100000
					});
				} else {
					toast({
						title: $_('toasts.reconnected.title'),
						description: $_('toasts.reconnected.description'),
						variant: "success",
						duration: 100000
					});
				}
			}, 1000); // Wait 1 second before reconnecting

		} catch (error) {
			console.error("Failed to save OBS settings:", error);
			toast({
				title: $_('toasts.error.title'),
				description: $_('toasts.error.saveSettings'),
				variant: "error",
				duration: 100000
			});
		}
	};
</script>

<!-- OBS Settings Card -->
<Card>
	<svelte:fragment slot="title">
		<Settings class="h-5 w-5" />
		{$_('obsSettings.card.title')}
	</svelte:fragment>

	<svelte:fragment slot="description">
		{$_('obsSettings.card.description')}
	</svelte:fragment>

	<svelte:fragment slot="content">
		<div class="space-y-6">
			<!-- WebSocket URL Input -->
			<div class="space-y-2">
				<Label for="websocket-url">{$_('obsSettings.form.websocketUrl')}</Label>
				<Input
					id="websocket-url"
					type="text"
					bind:value={websocketUrl}
					placeholder="ws://localhost:4455"
					disabled={isLoading}
				/>
				<p class="text-xs text-muted-foreground">
					{$_('obsSettings.form.websocketUrlHint')}
				</p>
			</div>

			<!-- WebSocket Password Input -->
			<div class="space-y-2">
				<Label for="websocket-password">{$_('obsSettings.form.websocketPassword')}</Label>
				<Input
					id="websocket-password"
					type="password"
					bind:value={websocketPassword}
					placeholder={$_('obsSettings.form.websocketPasswordPlaceholder')}
					disabled={isLoading}
				/>
				<p class="text-xs text-muted-foreground">{$_('obsSettings.form.websocketPasswordHint')}</p>
			</div>

			<!-- Action Buttons -->
			<div class="flex gap-3 pt-2">
				<Button
					buttonVariant="outline"
					onclick={handleTestConnection}
					disabled={$obsStatus.connected || isTesting || isLoading}
					className="flex-1 bg-transparent"
				>
					<TestTube class="mr-2 h-4 w-4" />
					{isTesting ? $_('obsSettings.form.testing') : $_('obsSettings.form.testConnection')}
				</Button>

				<Button
					onclick={handleSaveSettings}
					disabled={isLoading}
					className="flex-1"
				>
					<Save class="mr-2 h-4 w-4" />
					{isLoading ? $_('obsSettings.form.loading') : $_('obsSettings.form.saveSettings')}
				</Button>
			</div>

			<!-- Connection Status -->
			<div class="rounded-lg bg-muted/50 p-4 space-y-3">
				<div class="flex items-center justify-between">
					<h4 class="font-medium text-sm">{$_('obsSettings.status.title')}</h4>
					<div class="flex items-center gap-2">
						{#if $obsStatus.connected}
							<Wifi class="h-4 w-4 text-green-600" />
							<span class="text-sm text-green-600 font-medium">{$_('obsSettings.status.connected')}</span>
						{:else}
							<WifiOff class="h-4 w-4 text-red-600" />
							<span class="text-sm text-red-600 font-medium">{$_('obsSettings.status.disconnected')}</span>
						{/if}
					</div>
				</div>

				{#if $obsStatus.lastConnected}
					<p class="text-xs text-muted-foreground">
						{$_('obsSettings.status.lastConnected')}: {new Date($obsStatus.lastConnected).toLocaleString()}
					</p>
				{/if}

				{#if $obsStatus.error}
					<p class="text-xs text-red-600">
						{$_('obsSettings.status.error')}: {$obsStatus.error}
					</p>
				{/if}
			</div>

			<!-- Setup Instructions -->
			<div class="rounded-lg bg-muted p-4 space-y-2">
				<h4 class="font-medium text-sm">{$_('obsSettings.instructions.title')}</h4>
				<ol class="text-sm text-muted-foreground space-y-1 list-decimal list-inside">
					<li>{$_('obsSettings.instructions.step1')}</li>
					<li>{$_('obsSettings.instructions.step2')}</li>
					<li>{$_('obsSettings.instructions.step3')}</li>
					<li>{$_('obsSettings.instructions.step4')}</li>
					<li>{$_('obsSettings.instructions.step5')}</li>
					<li>{$_('obsSettings.instructions.step6')}</li>
					<li>{$_('obsSettings.instructions.step7')}</li>
				</ol>
			</div>
		</div>
	</svelte:fragment>
</Card>
