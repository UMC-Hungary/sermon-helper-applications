<script lang="ts">
	import Card from '$lib/components/ui/card.svelte';
	import Button from '$lib/components/ui/button.svelte';
	import Input from '$lib/components/ui/input.svelte';
	import Label from '$lib/components/ui/label.svelte';
	import Textarea from '$lib/components/ui/textarea.svelte';
	import { toast } from '$lib/utils/toast';
	import { captionSettingsStore, type CaptionSettings } from '$lib/utils/caption-store';
	import { discoveryServerStatus } from '$lib/stores/discovery-server-store';
	import {
		Subtitles,
		Copy,
		Check,
		Save,
		RotateCcw,
		Eye,
		EyeOff
	} from 'lucide-svelte';
	import { onMount } from 'svelte';

	let settings: CaptionSettings = captionSettingsStore.getDefaultSettings();
	let isLoading = true;
	let isSaving = false;
	let urlCopied = false;

	onMount(async () => {
		try {
			settings = await captionSettingsStore.getSettings();
		} catch (error) {
			console.error('Failed to load caption settings:', error);
			toast({
				title: 'Error',
				description: 'Failed to load caption settings',
				variant: 'error'
			});
		} finally {
			isLoading = false;
		}
	});

	// Generate the caption URL
	function getCaptionUrl(): string {
		if (!$discoveryServerStatus.running) {
			return 'Start discovery server to generate URL';
		}

		const baseUrl = $discoveryServerStatus.addresses.length > 0
			? `http://${$discoveryServerStatus.addresses[0]}:${$discoveryServerStatus.port}`
			: `http://localhost:${$discoveryServerStatus.port}`;

		const params = new URLSearchParams();
		params.set('type', settings.type);
		if (settings.title) params.set('title', settings.title);
		if (settings.boldText) params.set('bold', settings.boldText);
		if (settings.lightText) params.set('light', settings.lightText);
		params.set('color', settings.color);
		params.set('showLogo', settings.showLogo ? 'visible' : 'hidden');
		if (settings.svgLogo) {
			params.set('logo', encodeURIComponent(settings.svgLogo));
		}

		return `${baseUrl}/caption?${params.toString()}`;
	}

	async function handleCopyUrl() {
		const url = getCaptionUrl();
		if (!$discoveryServerStatus.running) {
			toast({
				title: 'Server Not Running',
				description: 'Start the discovery server first',
				variant: 'error'
			});
			return;
		}

		await navigator.clipboard.writeText(url);
		urlCopied = true;
		toast({
			title: 'URL Copied',
			description: 'Caption URL copied to clipboard',
			variant: 'success'
		});
		setTimeout(() => {
			urlCopied = false;
		}, 2000);
	}

	async function handleSave() {
		isSaving = true;
		try {
			await captionSettingsStore.saveSettings(settings);
			toast({
				title: 'Settings Saved',
				description: 'Caption settings have been saved',
				variant: 'success'
			});
		} catch (error) {
			console.error('Failed to save caption settings:', error);
			toast({
				title: 'Error',
				description: 'Failed to save caption settings',
				variant: 'error'
			});
		} finally {
			isSaving = false;
		}
	}

	async function handleReset() {
		settings = captionSettingsStore.getDefaultSettings();
		await handleSave();
	}

	function getHeight(): string {
		return settings.type === 'full' ? '1080' : '150';
	}
</script>

<div class="mt-12 lg:mt-0">
	<h2 class="text-3xl font-bold tracking-tight">OBS Caption</h2>
	<p class="text-muted-foreground">Create embeddable captions for OBS browser sources</p>
</div>

<div class="grid gap-6 lg:grid-cols-2 mt-6">
	<!-- Caption Settings -->
	<div class="space-y-6">
		<Card>
			<svelte:fragment slot="title">
				<Subtitles class="h-5 w-5" />
				Caption Settings
			</svelte:fragment>

			<svelte:fragment slot="description">
				Configure the text and appearance of your caption
			</svelte:fragment>

			<svelte:fragment slot="content">
				<div class="space-y-6">
					<!-- Type -->
					<div class="space-y-2">
						<Label for="caption-type">Caption Type</Label>
						<select
							id="caption-type"
							bind:value={settings.type}
							disabled={isLoading}
							class="flex h-10 w-full items-center justify-between rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
						>
							<option value="caption">Caption (150px height)</option>
							<option value="full">Full Screen (1080px height)</option>
						</select>
					</div>

					<!-- Title -->
					<div class="space-y-2">
						<Label for="caption-title">Title</Label>
						<Input
							id="caption-title"
							type="text"
							bind:value={settings.title}
							placeholder="e.g., SUNDAY SERVICE"
							disabled={isLoading}
						/>
						<p class="text-xs text-muted-foreground">
							Main heading displayed in uppercase
						</p>
					</div>

					<!-- Bold Text -->
					<div class="space-y-2">
						<Label for="caption-bold">Bold Text</Label>
						<Input
							id="caption-bold"
							type="text"
							bind:value={settings.boldText}
							placeholder="e.g., Pastor John Smith"
							disabled={isLoading}
						/>
						<p class="text-xs text-muted-foreground">
							Emphasized text (e.g., speaker name)
						</p>
					</div>

					<!-- Light Text -->
					<div class="space-y-2">
						<Label for="caption-light">Light Text</Label>
						<Input
							id="caption-light"
							type="text"
							bind:value={settings.lightText}
							placeholder="e.g., Sermon Title"
							disabled={isLoading}
						/>
						<p class="text-xs text-muted-foreground">
							Secondary text (e.g., sermon topic)
						</p>
					</div>

					<!-- Color -->
					<div class="space-y-2">
						<Label for="caption-color">Background Color</Label>
						<select
							id="caption-color"
							bind:value={settings.color}
							disabled={isLoading}
							class="flex h-10 w-full items-center justify-between rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
						>
							<option value="black">Black</option>
							<option value="red">Red</option>
							<option value="blue">Blue</option>
							<option value="green">Green</option>
						</select>
					</div>

					<!-- Show Logo Toggle -->
					<div class="flex items-center gap-3">
						<input
							type="checkbox"
							id="show-logo"
							bind:checked={settings.showLogo}
							disabled={isLoading}
							class="h-4 w-4 rounded border-gray-300"
						/>
						<Label for="show-logo" className="text-sm font-normal flex items-center gap-2">
							{#if settings.showLogo}
								<Eye class="h-4 w-4" />
								Show Logo
							{:else}
								<EyeOff class="h-4 w-4" />
								Hide Logo
							{/if}
						</Label>
					</div>

					<!-- Action Buttons -->
					<div class="flex gap-3 pt-2">
						<Button
							buttonVariant="outline"
							onclick={handleReset}
							disabled={isLoading || isSaving}
							className="flex-1 bg-transparent"
						>
							<RotateCcw class="mr-2 h-4 w-4" />
							Reset
						</Button>

						<Button
							onclick={handleSave}
							disabled={isLoading || isSaving}
							className="flex-1"
						>
							<Save class="mr-2 h-4 w-4" />
							{isSaving ? 'Saving...' : 'Save'}
						</Button>
					</div>
				</div>
			</svelte:fragment>
		</Card>
	</div>

	<div class="space-y-6">
		<!-- SVG Logo -->
		<Card>
			<svelte:fragment slot="title">
				SVG Logo
			</svelte:fragment>

			<svelte:fragment slot="description">
				Paste your SVG logo code here
			</svelte:fragment>

			<svelte:fragment slot="content">
				<div class="space-y-4">
					<Textarea
						bind:value={settings.svgLogo}
						placeholder={'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">...</svg>'}
						disabled={isLoading}
						rows={6}
						className="font-mono text-xs"
					/>
					<p class="text-xs text-muted-foreground">
						Paste the complete SVG code. The logo will be displayed on the left side of the caption.
					</p>

					{#if settings.svgLogo}
						<div class="rounded-lg bg-muted p-4">
							<p class="text-xs text-muted-foreground mb-2">Preview:</p>
							<div class="flex items-center justify-center h-20 bg-background rounded">
								{@html settings.svgLogo}
							</div>
						</div>
					{/if}
				</div>
			</svelte:fragment>
		</Card>

		<!-- Generated URL -->
		<Card>
			<svelte:fragment slot="title">
				OBS Browser Source URL
			</svelte:fragment>

			<svelte:fragment slot="description">
				Use this URL in OBS as a Browser Source
			</svelte:fragment>

			<svelte:fragment slot="content">
				<div class="space-y-4">
					{#if !$discoveryServerStatus.running}
						<div class="rounded-lg bg-destructive/10 border border-destructive/20 p-4">
							<p class="text-sm text-destructive">
								Discovery server is not running. Start it from Settings to generate the URL.
							</p>
						</div>
					{:else}
						<div class="rounded-lg bg-muted p-3">
							<code class="text-xs break-all">{getCaptionUrl()}</code>
						</div>
					{/if}

					<Button
						onclick={handleCopyUrl}
						disabled={!$discoveryServerStatus.running}
						className="w-full"
					>
						{#if urlCopied}
							<Check class="mr-2 h-4 w-4" />
							Copied!
						{:else}
							<Copy class="mr-2 h-4 w-4" />
							Copy URL
						{/if}
					</Button>

					<!-- OBS Instructions -->
					<div class="rounded-lg bg-muted p-4 space-y-2">
						<h4 class="font-medium text-sm">OBS Setup Instructions</h4>
						<ol class="text-sm text-muted-foreground space-y-1 list-decimal list-inside">
							<li>In OBS, add a new Browser Source</li>
							<li>Paste the URL above</li>
							<li>Set width to <strong>1920</strong></li>
							<li>Set height to <strong>{getHeight()}</strong></li>
							<li>Check "Refresh browser when scene becomes active"</li>
						</ol>
					</div>
				</div>
			</svelte:fragment>
		</Card>
	</div>
</div>
