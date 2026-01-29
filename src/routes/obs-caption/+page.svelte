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
			case 'white': return '#ffffff';
			default: return '#000000';
		}
	}

	// Get text color based on background
	function getTextColor(color: string): string {
		return color === 'white' ? '#000000' : '#ffffff';
	}

	// Get accent color (for service info)
	function getAccentColor(color: string): string {
		if (color === 'white' || color === 'black') return '#dc2626'; // red
		return '#ffffff'; // white on colored backgrounds
	}

	// Get dimensions for current settings
	$: exportDimensions = getExportDimensions(settings.resolution, settings.aspectRatio);
	$: captionHeight = getCaptionHeight(settings.type, settings.resolution);
	$: previewWidth = exportDimensions.width;
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

	// Export to PNG using canvas
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
			const paddingX = width * 0.1; // 10% horizontal padding
			const paddingY = height * 0.05; // 5% vertical padding

			// For full-screen service announcement style
			if (settings.type === 'full') {
				let currentY = paddingY;

				// Draw title (large name)
				if (settings.title) {
					const titleSize = Math.min(width * 0.1, height * 0.15); // Responsive size
					ctx.font = `700 ${titleSize}px -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif`;
					ctx.fillStyle = getTextColor(settings.color);
					ctx.textBaseline = 'top';
					ctx.fillText(settings.title, paddingX, currentY);
					currentY += titleSize * 1.2;
				}

				// Draw service info (bold • light format)
				if (settings.boldText || settings.lightText) {
					const infoSize = Math.min(width * 0.03, height * 0.05);
					ctx.font = `700 ${infoSize}px -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif`;
					ctx.fillStyle = getAccentColor(settings.color);
					ctx.textBaseline = 'top';

					let infoText = '';
					if (settings.boldText) infoText += settings.boldText.toUpperCase();
					if (settings.boldText && settings.lightText) infoText += '  •  ';
					if (settings.lightText) infoText += settings.lightText.toUpperCase();

					ctx.fillText(infoText, paddingX, currentY + (infoSize * 0.5));
				}

				// Draw logo at bottom
				if (settings.showLogo && settings.svgLogo) {
					const logoMaxWidth = width * 0.25;
					const logoMaxHeight = height * 0.12;

					const svgBlob = new Blob([settings.svgLogo], { type: 'image/svg+xml' });
					const svgUrl = URL.createObjectURL(svgBlob);
					const logoImg = new window.Image();

					await new Promise<void>((resolve, reject) => {
						logoImg.onload = () => resolve();
						logoImg.onerror = reject;
						logoImg.src = svgUrl;
					});

					const logoAspect = logoImg.width / logoImg.height;
					let drawWidth = logoMaxHeight * logoAspect;
					let drawHeight = logoMaxHeight;

					if (drawWidth > logoMaxWidth) {
						drawWidth = logoMaxWidth;
						drawHeight = logoMaxWidth / logoAspect;
					}

					const logoY = height - paddingY - drawHeight;
					ctx.drawImage(logoImg, paddingX, logoY, drawWidth, drawHeight);
					URL.revokeObjectURL(svgUrl);
				}
			} else {
				// Caption bar style (original)
				const gap = 30 * scale;
				let contentX = paddingX;

				// Draw logo if present
				if (settings.showLogo && settings.svgLogo) {
					const logoHeight = height * 0.55;
					const logoMaxWidth = 120 * scale;

					const svgBlob = new Blob([settings.svgLogo], { type: 'image/svg+xml' });
					const svgUrl = URL.createObjectURL(svgBlob);
					const logoImg = new window.Image();

					await new Promise<void>((resolve, reject) => {
						logoImg.onload = () => resolve();
						logoImg.onerror = reject;
						logoImg.src = svgUrl;
					});

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
				ctx.fillStyle = getTextColor(settings.color);
				const titleSize = 36 * scale;
				const textSize = 28 * scale;
				const contentGap = 8 * scale;

				let totalTextHeight = 0;
				if (settings.title) totalTextHeight += titleSize;
				if (settings.boldText || settings.lightText) {
					if (settings.title) totalTextHeight += contentGap;
					totalTextHeight += textSize;
				}

				let textY = (height - totalTextHeight) / 2;

				if (settings.title) {
					ctx.font = `700 ${titleSize}px -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif`;
					ctx.textBaseline = 'top';
					ctx.fillText(settings.title.toUpperCase(), contentX, textY);
					textY += titleSize + contentGap;
				}

				if (settings.boldText || settings.lightText) {
					let textX = contentX;

					if (settings.boldText) {
						ctx.font = `600 ${textSize}px -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif`;
						ctx.textBaseline = 'top';
						ctx.fillText(settings.boldText, textX, textY);
						textX += ctx.measureText(settings.boldText).width + ctx.measureText(' ').width;
					}

					if (settings.lightText) {
						ctx.font = `300 ${textSize}px -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif`;
						ctx.globalAlpha = 0.9;
						ctx.textBaseline = 'top';
						ctx.fillText(settings.lightText, textX, textY);
						ctx.globalAlpha = 1;
					}
				}
			}

			// Convert to blob and download
			const blob = await new Promise<Blob>((resolve, reject) => {
				canvas.toBlob((b) => {
					if (b) resolve(b);
					else reject(new Error('Failed to create blob'));
				}, 'image/png');
			});

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
				description: `Image exported at ${width}x${settings.type === 'full' ? exportDimensions.height : captionHeight}`,
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

					<!-- Title / Name -->
					<div class="space-y-2">
						<Label for="caption-title">{settings.type === 'full' ? 'Name' : 'Title'}</Label>
						<Input
							id="caption-title"
							type="text"
							bind:value={settings.title}
							placeholder={settings.type === 'full' ? 'e.g., Pásztor Balázs' : 'e.g., SUNDAY SERVICE'}
							disabled={isLoading}
						/>
						<p class="text-xs text-muted-foreground">
							{settings.type === 'full' ? 'Large display name (speaker name)' : 'Main heading displayed in uppercase'}
						</p>
					</div>

					<!-- Service Info / Bold Text -->
					<div class="space-y-2">
						<Label for="caption-bold">{settings.type === 'full' ? 'Service Type' : 'Bold Text'}</Label>
						<Input
							id="caption-bold"
							type="text"
							bind:value={settings.boldText}
							placeholder={settings.type === 'full' ? 'e.g., VASÁRNAPI ISTENTISZTELET' : 'e.g., Pastor John Smith'}
							disabled={isLoading}
						/>
					</div>

					<!-- Secondary Info / Light Text -->
					<div class="space-y-2">
						<Label for="caption-light">{settings.type === 'full' ? 'Event Type' : 'Light Text'}</Label>
						<Input
							id="caption-light"
							type="text"
							bind:value={settings.lightText}
							placeholder={settings.type === 'full' ? 'e.g., IGEHIRDETÉS' : 'e.g., Sermon Title'}
							disabled={isLoading}
						/>
						{#if settings.type === 'full'}
							<p class="text-xs text-muted-foreground">
								Displayed as: {settings.boldText || 'SERVICE TYPE'} • {settings.lightText || 'EVENT TYPE'}
							</p>
						{/if}
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
								<option value="white">White</option>
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
						<p class="text-xs text-muted-foreground">
							{settings.type === 'full' ? 'Logo displayed at bottom left' : 'Logo displayed on the left side'}
						</p>
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
					<div class="rounded-lg overflow-hidden border shadow-lg">
						{#if settings.type === 'full'}
							<!-- Full Screen Service Announcement Style -->
							<div
								class="w-full relative"
								style="aspect-ratio: 16 / 9; background-color: {getBgColor(settings.color)};"
							>
								<div
									class="absolute inset-0 flex flex-col justify-between"
									style="padding: 5% 10%;"
								>
									<!-- Content Area -->
									<div class="flex flex-col">
										<!-- Large Name -->
										{#if settings.title}
											<h1
												class="font-bold leading-none m-0"
												style="font-size: clamp(1.5rem, 8vw, 4rem); color: {getTextColor(settings.color)};"
											>
												{settings.title}
											</h1>
										{/if}

										<!-- Service Info with Dot Separator -->
										{#if settings.boldText || settings.lightText}
											<div
												class="font-bold flex items-center gap-2 mt-2"
												style="font-size: clamp(0.6rem, 2.5vw, 1.2rem); color: {getAccentColor(settings.color)};"
											>
												{#if settings.boldText}
													<span>{settings.boldText.toUpperCase()}</span>
												{/if}
												{#if settings.boldText && settings.lightText}
													<span
														class="inline-block rounded-full"
														style="width: 0.4em; height: 0.4em; background-color: {getAccentColor(settings.color)};"
													></span>
												{/if}
												{#if settings.lightText}
													<span>{settings.lightText.toUpperCase()}</span>
												{/if}
											</div>
										{/if}
									</div>

									<!-- Logo at Bottom -->
									{#if settings.showLogo && settings.svgLogo}
										<div class="mt-auto w-full max-w-[30%]">
											<div class="[&>svg]:w-full [&>svg]:h-auto">
												{@html settings.svgLogo}
											</div>
										</div>
									{/if}
								</div>
							</div>
						{:else}
							<!-- Caption Bar Style -->
							<div
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
									<div
										class="flex-1 flex flex-col justify-center gap-[2%]"
										style="color: {getTextColor(settings.color)};"
									>
										{#if settings.title}
											<div
												class="font-bold uppercase tracking-wide leading-tight"
												style="font-size: clamp(8px, 2.5vw, 18px);"
											>
												{settings.title}
											</div>
										{/if}
										{#if settings.boldText || settings.lightText}
											<div
												class="leading-tight"
												style="font-size: clamp(6px, 2vw, 14px);"
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
						{/if}
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
