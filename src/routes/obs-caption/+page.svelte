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
	import { appSettings } from '$lib/utils/app-settings-store';
	import { appSettingsStore } from '$lib/utils/app-settings-store';
	import { saveFile, pickOutputFolder, openFolder } from '$lib/utils/file-saver';
	import { isTauriApp } from '$lib/utils/storage-helpers';
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
		Loader2,
		FolderOpen
	} from 'lucide-svelte';
	import { onMount } from 'svelte';

	const isTauri = isTauriApp();

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

	// Get text color based on color setting (matching reference: color changes text, bg is always white)
	function getTextColor(color: string): string {
		return color === 'red' ? '#EA0029' : '#000000';
	}

	// Get dimensions for current settings
	$: exportDimensions = getExportDimensions(settings.resolution, settings.aspectRatio);
	$: captionHeight = getCaptionHeight(settings.type, settings.resolution);
	$: previewWidth = exportDimensions.width;
	$: previewHeight = settings.type === 'preview' ? exportDimensions.height : captionHeight;

	// Reactive caption URL — depends on all settings fields so iframe updates live
	$: captionUrl = $discoveryServerStatus.running
		? (() => {
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
				params.set('showLogo', settings.showLogo ? 'true' : 'false');
				return `${baseUrl}/caption?${params.toString()}`;
			})()
		: '';


	async function handleCopyUrl() {
		if (!captionUrl) {
			toast({
				title: 'Server Not Running',
				description: 'Start the discovery server first',
				variant: 'error'
			});
			return;
		}

		await navigator.clipboard.writeText(captionUrl);
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

	// Export to PNG using canvas (Oswald font, white bg, matching reference)
	async function handleExport() {
		isExporting = true;
		try {
			const { width, height } = settings.type === 'preview'
				? exportDimensions
				: { width: exportDimensions.width, height: captionHeight };

			const canvas = document.createElement('canvas');
			canvas.width = width;
			canvas.height = height;
			const ctx = canvas.getContext('2d');

			if (!ctx) {
				throw new Error('Could not get canvas context');
			}

			// White background (always, matching reference)
			ctx.fillStyle = '#ffffff';
			ctx.fillRect(0, 0, width, height);

			const textColor = getTextColor(settings.color);
			const scale = settings.resolution === '4k' ? 2 : 1;

			if (settings.type === 'preview') {
				// Preview/full-screen style
				const paddingX = width * 0.08;
				const paddingBottom = height * 0.15;

				// Draw title
				if (settings.title) {
					const titleSize = 200 * scale;
					ctx.font = `600 ${titleSize}px Oswald, sans-serif`;
					ctx.fillStyle = textColor;
					ctx.textBaseline = 'top';
					ctx.fillText(settings.title, paddingX, height * 0.05);
				}

				// Draw caption text (bold + dot + light)
				if (settings.boldText || settings.lightText) {
					const captionSize = height * 0.267; // 26.667vh
					ctx.textBaseline = 'middle';
					const textY = height * 0.55;
					let textX = paddingX;

					if (settings.boldText) {
						ctx.font = `600 ${captionSize}px Oswald, sans-serif`;
						ctx.fillStyle = textColor;
						ctx.fillText(settings.boldText.toUpperCase(), textX, textY);
						textX += ctx.measureText(settings.boldText.toUpperCase()).width;
					}

					if (settings.boldText && settings.lightText) {
						// Dot separator
						const dotSize = 15 * scale;
						const dotMargin = 16 * scale;
						textX += dotMargin;
						ctx.beginPath();
						ctx.arc(textX + dotSize / 2, textY, dotSize / 2, 0, Math.PI * 2);
						ctx.fillStyle = textColor;
						ctx.fill();
						textX += dotSize + dotMargin;
					}

					if (settings.lightText) {
						ctx.font = `300 ${captionSize}px Oswald, sans-serif`;
						ctx.fillStyle = textColor;
						ctx.fillText(settings.lightText.toUpperCase(), textX, textY);
					}
				}

				// Draw logo at bottom
				if (settings.showLogo && settings.svgLogo) {
					const logoMaxWidth = 300 * scale;
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

					const logoY = height - paddingBottom;
					ctx.drawImage(logoImg, paddingX, logoY, drawWidth, drawHeight);
					URL.revokeObjectURL(svgUrl);
				}
			} else {
				// Caption bar style (matching reference exactly)
				const padding = 48 * scale; // 3rem
				const paddingY = 32 * scale; // 2rem
				let contentX = padding;

				// Draw logo if present
				if (settings.showLogo && settings.svgLogo) {
					// Logo takes flex: 0 0 133.3334vh equivalent
					const logoWidth = height * 1.333334;

					const svgBlob = new Blob([settings.svgLogo], { type: 'image/svg+xml' });
					const svgUrl = URL.createObjectURL(svgBlob);
					const logoImg = new window.Image();

					await new Promise<void>((resolve, reject) => {
						logoImg.onload = () => resolve();
						logoImg.onerror = reject;
						logoImg.src = svgUrl;
					});

					const logoAspect = logoImg.width / logoImg.height;
					const drawHeight = height - paddingY * 2;
					let drawWidth = drawHeight * logoAspect;
					if (drawWidth > logoWidth) {
						drawWidth = logoWidth;
					}

					const logoY = (height - drawHeight) / 2;
					ctx.drawImage(logoImg, contentX, logoY, drawWidth, drawHeight);
					contentX += drawWidth;

					// Draw vertical divider
					const dividerMargin = height * 0.213334; // 21.3334vh
					const dividerHeight = height * 0.5; // 50vh
					const dividerX = contentX + dividerMargin;
					const dividerY = (height - dividerHeight) / 2;
					ctx.strokeStyle = textColor;
					ctx.lineWidth = 5 * scale;
					ctx.beginPath();
					ctx.moveTo(dividerX, dividerY);
					ctx.lineTo(dividerX, dividerY + dividerHeight);
					ctx.stroke();
					contentX = dividerX + dividerMargin;
				}

				// Draw caption text
				const captionSize = height * 0.26667; // 26.667vh
				ctx.textBaseline = 'middle';
				const textY = height / 2;
				let textX = contentX;

				if (settings.boldText) {
					ctx.font = `600 ${captionSize}px Oswald, sans-serif`;
					ctx.fillStyle = textColor;
					ctx.fillText(settings.boldText.toUpperCase(), textX, textY);
					textX += ctx.measureText(settings.boldText.toUpperCase()).width;
				}

				if (settings.boldText && settings.lightText) {
					// Dot separator
					const dotSize = 15 * scale;
					const dotMargin = 16 * scale;
					textX += dotMargin;
					ctx.beginPath();
					ctx.arc(textX + dotSize / 2, textY, dotSize / 2, 0, Math.PI * 2);
					ctx.fillStyle = textColor;
					ctx.fill();
					textX += dotSize + dotMargin;
				}

				if (settings.lightText) {
					ctx.font = `300 ${captionSize}px Oswald, sans-serif`;
					ctx.fillStyle = textColor;
					ctx.fillText(settings.lightText.toUpperCase(), textX, textY);
				}
			}

			// Convert to blob and save
			const blob = await new Promise<Blob>((resolve, reject) => {
				canvas.toBlob((b) => {
					if (b) resolve(b);
					else reject(new Error('Failed to create blob'));
				}, 'image/png');
			});

			const filename = `caption-${settings.resolution}-${settings.aspectRatio.replace(':', 'x')}.png`;

			// If Tauri and no output path configured, prompt for folder
			if (isTauri && !$appSettings.captionOutputPath) {
				const selectedPath = await pickOutputFolder('Select Caption Export Folder');
				if (!selectedPath) {
					toast({
						title: 'Export Cancelled',
						description: 'No output folder selected',
						variant: 'error'
					});
					return;
				}
				await appSettingsStore.set('captionOutputPath', selectedPath);
			}

			const result = await saveFile(blob, filename, $appSettings.captionOutputPath);

			if (result.success) {
				toast({
					title: 'Export Complete',
					description: result.path
						? `Saved to ${result.path}`
						: `Image exported at ${width}x${settings.type === 'preview' ? exportDimensions.height : captionHeight}`,
					variant: 'success'
				});
			} else {
				toast({
					title: 'Export Failed',
					description: result.error || 'Failed to export image',
					variant: 'error'
				});
			}
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

	async function handleChangeFolder() {
		const selectedPath = await pickOutputFolder('Select Caption Export Folder');
		if (selectedPath) {
			await appSettingsStore.set('captionOutputPath', selectedPath);
		}
	}

	async function handleOpenFolder() {
		if ($appSettings.captionOutputPath) {
			await openFolder($appSettings.captionOutputPath);
		}
	}

	function getOBSDimensions(): { width: number; height: number } {
		const base = RESOLUTION_DIMENSIONS[settings.resolution];
		if (settings.type === 'preview') {
			return base;
		}
		return { width: base.width, height: captionHeight };
	}
</script>

<svelte:head>
	<link href="https://fonts.googleapis.com/css2?family=Oswald:wght@300;600&display=swap" rel="stylesheet" />
</svelte:head>

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
								<option value="preview">Preview (Full Screen)</option>
							</select>
						</div>
					</div>

					<!-- Title (only shown in preview mode, hidden in caption per reference) -->
					{#if settings.type === 'preview'}
						<div class="space-y-2">
							<Label for="caption-title">Title</Label>
							<Input
								id="caption-title"
								type="text"
								bind:value={settings.title}
								placeholder="e.g., Pásztor Balázs"
								disabled={isLoading}
							/>
							<p class="text-xs text-muted-foreground">
								Large display name (shown at top)
							</p>
						</div>
					{/if}

					<!-- Bold Text -->
					<div class="space-y-2">
						<Label for="caption-bold">Bold text</Label>
						<Input
							id="caption-bold"
							type="text"
							bind:value={settings.boldText}
							placeholder="e.g., VASÁRNAPI ISTENTISZTELET"
							disabled={isLoading}
						/>
					</div>

					<!-- Light Text -->
					<div class="space-y-2">
						<Label for="caption-light">Light text</Label>
						<Input
							id="caption-light"
							type="text"
							bind:value={settings.lightText}
							placeholder="e.g., IGEHIRDETÉS"
							disabled={isLoading}
						/>
						<p class="text-xs text-muted-foreground">
							Displayed as: {settings.boldText || 'BOLD'} <span class="inline-block w-1.5 h-1.5 rounded-full bg-current align-middle mx-1"></span> {settings.lightText || 'LIGHT'} (uppercase)
						</p>
					</div>

					<!-- Color & Logo Row -->
					<div class="grid grid-cols-2 gap-4">
						<div class="space-y-2">
							<Label for="caption-color">Text Color</Label>
							<select
								id="caption-color"
								bind:value={settings.color}
								disabled={isLoading}
								class="flex h-10 w-full items-center justify-between rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
							>
								<option value="black">Black</option>
								<option value="red">Red</option>
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
							{settings.type === 'preview' ? 'Logo displayed at bottom left' : 'Logo displayed on the left side with vertical divider'}
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
					<!-- Preview Container (iframe showing actual served caption) -->
					{#if captionUrl}
						{@const obsDims = getOBSDimensions()}
						{@const previewScale = Math.min(1, 500 / obsDims.width)}
						{@const scaledHeight = obsDims.height * previewScale}
						<div
							class="rounded-lg overflow-hidden border shadow-lg"
							style="width: 100%; height: {scaledHeight}px;"
						>
							{#key captionUrl}
								<iframe
									title="Caption Preview"
									src={captionUrl}
									width={obsDims.width}
									height={obsDims.height}
									style="transform: scale({previewScale}); transform-origin: 0 0; border: none;"
								></iframe>
							{/key}
						</div>
					{:else}
						<div class="rounded-lg border p-8 text-center text-muted-foreground">
							Start the discovery server to see a live preview
						</div>
					{/if}

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
							Export size: {exportDimensions.width} × {settings.type === 'preview' ? exportDimensions.height : captionHeight} pixels
						</div>

						<!-- Output folder (Tauri only) -->
						{#if isTauri}
							<div class="space-y-2">
								<Label>Output Folder</Label>
								{#if $appSettings.captionOutputPath}
									<div class="flex items-center gap-2">
										<code class="flex-1 text-xs bg-background rounded px-2 py-1.5 truncate border">{$appSettings.captionOutputPath}</code>
										<button
											type="button"
											on:click={handleOpenFolder}
											class="shrink-0 p-1.5 rounded hover:bg-background transition-colors"
											title="Open folder"
										>
											<FolderOpen class="h-4 w-4" />
										</button>
										<button
											type="button"
											on:click={handleChangeFolder}
											class="shrink-0 text-xs text-muted-foreground hover:text-foreground underline"
										>
											Change
										</button>
									</div>
								{:else}
									<Button
										buttonVariant="outline"
										onclick={handleChangeFolder}
										className="w-full bg-transparent"
									>
										<FolderOpen class="mr-2 h-4 w-4" />
										Select Output Folder
									</Button>
								{/if}
							</div>
						{/if}

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
							<code class="text-xs break-all">{captionUrl}</code>
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
