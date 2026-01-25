<script lang="ts">
	import Card from '$lib/components/ui/card.svelte';
	import Button from '$lib/components/ui/button.svelte';
	import Input from '$lib/components/ui/input.svelte';
	import Label from '$lib/components/ui/label.svelte';
	import Textarea from '$lib/components/ui/textarea.svelte';
	import { toast } from '$lib/utils/toast';
	import {
		captionSettingsStore,
		type CaptionSettings,
		type Resolution,
		type AspectRatio,
		RESOLUTION_DIMENSIONS,
		getExportDimensions,
		getCaptionHeight
	} from '$lib/utils/caption-store';
	import { discoveryServerStatus } from '$lib/stores/discovery-server-store';
	import {
		Subtitles,
		Copy,
		Check,
		Save,
		RotateCcw,
		Eye,
		EyeOff,
		Download,
		Monitor,
		Image,
		Loader2
	} from 'lucide-svelte';
	import { onMount } from 'svelte';

	let settings: CaptionSettings = captionSettingsStore.getDefaultSettings();
	let isLoading = true;
	let isSaving = false;
	let urlCopied = false;
	let isExporting = false;
	let previewRef: HTMLDivElement;

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

	// Get background color
	function getBgColor(color: string): string {
		switch (color) {
			case 'red': return '#8B0000';
			case 'blue': return '#1a365d';
			case 'green': return '#1a4d1a';
			default: return '#000000';
		}
	}

	// Get dimensions for current settings
	$: exportDimensions = getExportDimensions(settings.resolution, settings.aspectRatio);
	$: captionHeight = getCaptionHeight(settings.type, settings.resolution);
	$: previewWidth = settings.type === 'full' ? exportDimensions.width : exportDimensions.width;
	$: previewHeight = settings.type === 'full' ? exportDimensions.height : captionHeight;

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
		params.set('resolution', settings.resolution);
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

	// Export to PNG
	async function handleExport() {
		isExporting = true;
		try {
			const { width, height } = settings.type === 'full'
				? exportDimensions
				: { width: exportDimensions.width, height: captionHeight };

			// Create canvas
			const canvas = document.createElement('canvas');
			canvas.width = width;
			canvas.height = height;
			const ctx = canvas.getContext('2d');

			if (!ctx) {
				throw new Error('Could not get canvas context');
			}

			// Draw background
			ctx.fillStyle = getBgColor(settings.color);
			ctx.fillRect(0, 0, width, height);

			// Scale factor for 4K
			const scale = settings.resolution === '4k' ? 2 : 1;
			const padding = 40 * scale;
			const gap = 30 * scale;

			let contentX = padding;

			// Draw logo if present
			if (settings.showLogo && settings.svgLogo) {
				const logoHeight = 80 * scale;
				const logoMaxWidth = 120 * scale;

				// Convert SVG to image
				const svgBlob = new Blob([settings.svgLogo], { type: 'image/svg+xml' });
				const svgUrl = URL.createObjectURL(svgBlob);
				const logoImg = new window.Image();

				await new Promise<void>((resolve, reject) => {
					logoImg.onload = () => resolve();
					logoImg.onerror = reject;
					logoImg.src = svgUrl;
				});

				// Calculate logo dimensions maintaining aspect ratio
				const logoAspect = logoImg.width / logoImg.height;
				let drawWidth = logoHeight * logoAspect;
				let drawHeight = logoHeight;

				if (drawWidth > logoMaxWidth) {
					drawWidth = logoMaxWidth;
					drawHeight = logoMaxWidth / logoAspect;
				}

				const logoY = (height - drawHeight) / 2;
				ctx.drawImage(logoImg, contentX, logoY, drawWidth, drawHeight);

				URL.revokeObjectURL(svgUrl);
				contentX += drawWidth + gap;
			}

			// Draw text
			ctx.fillStyle = 'white';
			const titleSize = 36 * scale;
			const textSize = 28 * scale;
			const contentGap = 8 * scale;

			let textY = height / 2;

			// Calculate total text height
			let totalTextHeight = 0;
			if (settings.title) totalTextHeight += titleSize;
			if (settings.boldText || settings.lightText) {
				if (settings.title) totalTextHeight += contentGap;
				totalTextHeight += textSize;
			}

			textY = (height - totalTextHeight) / 2;

			// Draw title
			if (settings.title) {
				ctx.font = `700 ${titleSize}px -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif`;
				ctx.textBaseline = 'top';
				ctx.fillText(settings.title.toUpperCase(), contentX, textY);
				textY += titleSize + contentGap;
			}

			// Draw bold and light text
			if (settings.boldText || settings.lightText) {
				let textX = contentX;

				if (settings.boldText) {
					ctx.font = `600 ${textSize}px -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif`;
					ctx.textBaseline = 'top';
					ctx.fillText(settings.boldText, textX, textY);
					textX += ctx.measureText(settings.boldText).width;
				}

				if (settings.boldText && settings.lightText) {
					textX += ctx.measureText(' ').width;
				}

				if (settings.lightText) {
					ctx.font = `300 ${textSize}px -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif`;
					ctx.globalAlpha = 0.9;
					ctx.textBaseline = 'top';
					ctx.fillText(settings.lightText, textX, textY);
					ctx.globalAlpha = 1;
				}
			}

			// Convert to blob and download
			const blob = await new Promise<Blob>((resolve, reject) => {
				canvas.toBlob((b) => {
					if (b) resolve(b);
					else reject(new Error('Failed to create blob'));
				}, 'image/png');
			});

			// Create download link
			const url = URL.createObjectURL(blob);
			const a = document.createElement('a');
			a.href = url;
			a.download = `caption-${settings.resolution}-${settings.aspectRatio.replace(':', 'x')}.png`;
			document.body.appendChild(a);
			a.click();
			document.body.removeChild(a);
			URL.revokeObjectURL(url);

			toast({
				title: 'Export Complete',
				description: `Image exported at ${width}x${height}`,
				variant: 'success'
			});
		} catch (error) {
			console.error('Export failed:', error);
			toast({
				title: 'Export Failed',
				description: error instanceof Error ? error.message : 'Failed to export image',
				variant: 'error'
			});
		} finally {
			isExporting = false;
		}
	}

	function getOBSDimensions(): { width: number; height: number } {
		const base = RESOLUTION_DIMENSIONS[settings.resolution];
		if (settings.type === 'full') {
			return base;
		}
		return { width: base.width, height: captionHeight };
	}
</script>

<div class="mt-12 lg:mt-0">
	<h2 class="text-3xl font-bold tracking-tight">OBS Caption</h2>
	<p class="text-muted-foreground">Create embeddable captions for OBS browser sources</p>
</div>

<div class="grid gap-6 lg:grid-cols-2 mt-6">
	<!-- Left Column: Settings -->
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
					<!-- Resolution & Type Row -->
					<div class="grid grid-cols-2 gap-4">
						<div class="space-y-2">
							<Label for="caption-resolution">Resolution</Label>
							<select
								id="caption-resolution"
								bind:value={settings.resolution}
								disabled={isLoading}
								class="flex h-10 w-full items-center justify-between rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
							>
								<option value="1080p">1080p (1920×1080)</option>
								<option value="4k">4K (3840×2160)</option>
							</select>
						</div>

						<div class="space-y-2">
							<Label for="caption-type">Caption Type</Label>
							<select
								id="caption-type"
								bind:value={settings.type}
								disabled={isLoading}
								class="flex h-10 w-full items-center justify-between rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
							>
								<option value="caption">Caption Bar</option>
								<option value="full">Full Screen</option>
							</select>
						</div>
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
					</div>

					<!-- Color & Logo Row -->
					<div class="grid grid-cols-2 gap-4">
						<div class="space-y-2">
							<Label for="caption-color">Background</Label>
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

						<div class="space-y-2">
							<Label>Logo</Label>
							<div class="flex items-center h-10 gap-2">
								<input
									type="checkbox"
									id="show-logo"
									bind:checked={settings.showLogo}
									disabled={isLoading}
									class="h-4 w-4 rounded border-gray-300"
								/>
								<Label for="show-logo" className="text-sm font-normal flex items-center gap-2 cursor-pointer">
									{#if settings.showLogo}
										<Eye class="h-4 w-4" />
										Visible
									{:else}
										<EyeOff class="h-4 w-4" />
										Hidden
									{/if}
								</Label>
							</div>
						</div>
					</div>

					<!-- SVG Logo Input -->
					<div class="space-y-2">
						<Label for="svg-logo">SVG Logo Code</Label>
						<Textarea
							id="svg-logo"
							bind:value={settings.svgLogo}
							placeholder={'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">...</svg>'}
							disabled={isLoading}
							rows={4}
							className="font-mono text-xs"
						/>
					</div>

					<!-- Action Buttons -->
					<div class="flex gap-3">
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

	<!-- Right Column: Preview & Export -->
	<div class="space-y-6">
		<!-- Live Preview -->
		<Card>
			<svelte:fragment slot="title">
				<Monitor class="h-5 w-5" />
				Live Preview
			</svelte:fragment>

			<svelte:fragment slot="description">
				{previewWidth} × {previewHeight} pixels ({settings.resolution})
			</svelte:fragment>

			<svelte:fragment slot="content">
				<div class="space-y-4">
					<!-- Preview Container -->
					<div class="rounded-lg overflow-hidden border bg-muted/20">
						<div
							bind:this={previewRef}
							class="w-full relative"
							style="aspect-ratio: {previewWidth} / {previewHeight}; background-color: {getBgColor(settings.color)};"
						>
							<div class="absolute inset-0 flex items-center px-[2%] gap-[1.5%]">
								<!-- Logo -->
								{#if settings.showLogo && settings.svgLogo}
									<div class="flex-shrink-0 flex items-center justify-center h-[55%]">
										<div class="h-full w-auto [&>svg]:h-full [&>svg]:w-auto [&>svg]:max-w-[80px]">
											{@html settings.svgLogo}
										</div>
									</div>
								{/if}

								<!-- Content -->
								<div class="flex-1 flex flex-col justify-center gap-[2%] text-white">
									{#if settings.title}
										<div
											class="font-bold uppercase tracking-wide leading-tight"
											style="font-size: clamp(8px, 2.5vw, {settings.type === 'full' ? '36px' : '18px'});"
										>
											{settings.title}
										</div>
									{/if}
									{#if settings.boldText || settings.lightText}
										<div
											class="leading-tight"
											style="font-size: clamp(6px, 2vw, {settings.type === 'full' ? '28px' : '14px'});"
										>
											{#if settings.boldText}
												<span class="font-semibold">{settings.boldText}</span>
											{/if}
											{#if settings.boldText && settings.lightText}
												{' '}
											{/if}
											{#if settings.lightText}
												<span class="font-light opacity-90">{settings.lightText}</span>
											{/if}
										</div>
									{/if}
								</div>
							</div>
						</div>
					</div>

					<!-- Export Options -->
					<div class="rounded-lg bg-muted p-4 space-y-4">
						<div class="flex items-center gap-2">
							<Image class="h-4 w-4" />
							<h4 class="font-medium text-sm">Export Image</h4>
						</div>

						<div class="space-y-2">
							<Label for="aspect-ratio">Aspect Ratio</Label>
							<select
								id="aspect-ratio"
								bind:value={settings.aspectRatio}
								class="flex h-10 w-full items-center justify-between rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2"
							>
								<option value="16:9">16:9 (Widescreen)</option>
								<option value="4:3">4:3 (Standard)</option>
								<option value="1:1">1:1 (Square)</option>
								<option value="9:16">9:16 (Portrait)</option>
							</select>
						</div>

						<div class="text-xs text-muted-foreground">
							Export size: {exportDimensions.width} × {settings.type === 'full' ? exportDimensions.height : captionHeight} pixels
						</div>

						<Button
							onclick={handleExport}
							disabled={isExporting}
							className="w-full"
						>
							{#if isExporting}
								<Loader2 class="mr-2 h-4 w-4 animate-spin" />
								Exporting...
							{:else}
								<Download class="mr-2 h-4 w-4" />
								Export PNG
							{/if}
						</Button>
					</div>
				</div>
			</svelte:fragment>
		</Card>

		<!-- OBS URL -->
		<Card>
			<svelte:fragment slot="title">
				OBS Browser Source
			</svelte:fragment>

			<svelte:fragment slot="description">
				Use this URL in OBS Studio
			</svelte:fragment>

			<svelte:fragment slot="content">
				<div class="space-y-4">
					{#if !$discoveryServerStatus.running}
						<div class="rounded-lg bg-destructive/10 border border-destructive/20 p-4">
							<p class="text-sm text-destructive">
								Discovery server is not running. Start it from Settings.
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
					<div class="text-sm text-muted-foreground space-y-1">
						<p><strong>OBS Settings:</strong></p>
						<p>Width: <code class="bg-muted px-1 rounded">{getOBSDimensions().width}</code></p>
						<p>Height: <code class="bg-muted px-1 rounded">{getOBSDimensions().height}</code></p>
					</div>
				</div>
			</svelte:fragment>
		</Card>
	</div>
</div>
