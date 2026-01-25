<script lang="ts">
	import { CheckCircle2, XCircle, Menu, X, Youtube, Settings, Loader2, Globe, CalendarDays, Sun, Moon, Monitor, Edit, LogIn, RefreshCw, Check, FileText, FolderOpen, AlertCircle, Radio } from 'lucide-svelte';
	import { cn } from '$lib/utils.js';
	import Button from '$lib/components/ui/button.svelte';
	import Card from '$lib/components/ui/card.svelte';
	import Badge from '$lib/components/ui/badge.svelte';
	import Separator from '$lib/components/ui/separator.svelte';
	import { systemStore } from '$lib/stores/system-store';
	import { upcomingEvents, eventStore } from '$lib/stores/event-store';
	import { isEventToday, formatEventDate, generateCalculatedTitle } from '$lib/types/event';
	import { page } from '$app/state';
	import { _, locale } from 'svelte-i18n';
	import { setLocale } from '$lib/i18n';
	import { theme, setTheme } from '$lib/stores/theme-store';
	import { youtubeApi } from '$lib/utils/youtube-api';
	import { toast } from '$lib/utils/toast';
	import YouTubeLoginModal from '$lib/components/youtube-login-modal.svelte';
	import SidebarStreamingControls from '$lib/components/sidebar-streaming-controls.svelte';
	import UploadStatusSection from '$lib/components/upload-status-section.svelte';
	import Alert from '$lib/components/ui/alert.svelte';
	import AlertTitle from '$lib/components/ui/alert-title.svelte';
	import AlertDescription from '$lib/components/ui/alert-description.svelte';
	import { youtubeReauthRequired, youtubeReauthError, youtubeAuthStatusStore } from '$lib/stores/youtube-auth-status-store';
	import { requiredDeviceConfigs } from '$lib/stores/obs-devices-store';
	import { obsDeviceStatuses, obsBrowserStatuses, isCheckingDevices } from '$lib/stores/obs-device-status-store';
	import { browserSourceConfigs } from '$lib/stores/obs-devices-store';
	import { manualRefreshBrowserSource } from '$lib/utils/obs-device-checker';
	import { generatePptx } from '$lib/utils/pptx-generator';
	import { savePptxFile, pickOutputFolder, openFolder } from '$lib/utils/file-saver';
	import { appSettings, appSettingsStore } from '$lib/utils/app-settings-store';
	import { isTauriApp } from '$lib/utils/storage-helpers';

	type Status = "active" | "inactive" | "warning";

	export let isMobileMenuOpen = false;
	export let onMobileMenuToggle: () => void = () => {};

	// Reactive: get the soonest upcoming event (first in sorted list)
	$: nextEvent = $upcomingEvents.length > 0 ? $upcomingEvents[0] : null;
	$: isToday = nextEvent ? isEventToday(nextEvent) : false;
	$: isScheduledOnYoutube = nextEvent?.youtubeScheduledId != null;

	// State for YouTube scheduling
	let isScheduling = false;
	let showYoutubeLoginModal = false;

	// State for PPTX generation
	let generatingTextus = false;
	let generatingLeckio = false;
	$: isTauri = isTauriApp();
	$: pptxOutputPath = $appSettings.pptxOutputPath;

	// Handle PPTX generation
	async function handleGeneratePptx(type: 'textus' | 'leckio') {
		if (!nextEvent) return;

		const verses = type === 'textus' ? nextEvent.textusVerses : nextEvent.leckioVerses;
		const reference = type === 'textus' ? nextEvent.textus : nextEvent.leckio;

		if (!verses?.length || !reference) {
			toast({
				title: $_('toasts.error.title'),
				description: $_('sidebar.upcomingEvent.pptx.noVerses'),
				variant: 'error'
			});
			return;
		}

		// Check if output path is configured (Tauri only)
		if (isTauri && !pptxOutputPath) {
			const selectedPath = await pickOutputFolder();
			if (!selectedPath) {
				toast({
					title: $_('toasts.error.title'),
					description: $_('sidebar.upcomingEvent.pptx.noFolder'),
					variant: 'error'
				});
				return;
			}
			await appSettingsStore.set('pptxOutputPath', selectedPath);
		}

		// Set loading state
		if (type === 'textus') {
			generatingTextus = true;
		} else {
			generatingLeckio = true;
		}

		try {
			// Generate PPTX
			const blob = await generatePptx({ reference, verses, type });
			const filename = `${type === 'textus' ? 'textus' : 'lekcio'}.pptx`;

			// Save or download
			const result = await savePptxFile(blob, filename, $appSettings.pptxOutputPath);

			if (result.success) {
				// Update event with generation timestamp
				const timestampField = type === 'textus' ? 'textusGeneratedAt' : 'leckioGeneratedAt';
				await eventStore.updateEvent(nextEvent.id, { [timestampField]: new Date().toISOString() });

				toast({
					title: $_('toasts.success.title'),
					description: $_('sidebar.upcomingEvent.pptx.generated'),
					variant: 'success'
				});
			} else {
				toast({
					title: $_('toasts.error.title'),
					description: result.error || $_('sidebar.upcomingEvent.pptx.failed'),
					variant: 'error'
				});
			}
		} catch (error) {
			toast({
				title: $_('toasts.error.title'),
				description: error instanceof Error ? error.message : $_('sidebar.upcomingEvent.pptx.failed'),
				variant: 'error'
			});
		} finally {
			if (type === 'textus') {
				generatingTextus = false;
			} else {
				generatingLeckio = false;
			}
		}
	}

	// Change output folder
	async function handleChangeFolder() {
		const selectedPath = await pickOutputFolder();
		if (selectedPath) {
			await appSettingsStore.set('pptxOutputPath', selectedPath);
		}
	}

	// Schedule the upcoming event on YouTube
	async function scheduleUpcomingEvent() {
		if (!nextEvent || nextEvent.youtubeScheduledId) return;

		isScheduling = true;
		try {
			// Use the shared title generation function
			const title = generateCalculatedTitle(nextEvent);

			// Build description
			const descParts: string[] = [];
			if (nextEvent.speaker) descParts.push(`${$_('events.form.speaker')}: ${nextEvent.speaker}`);
			if (nextEvent.textus) descParts.push(`Textus: ${nextEvent.textus}`);
			if (nextEvent.leckio) descParts.push(`Lekció: ${nextEvent.leckio}`);
			if (nextEvent.description) {
				descParts.push('');
				descParts.push(nextEvent.description);
			}

			// Build scheduled start time in ISO 8601 format
			const scheduledStartTime = new Date(`${nextEvent.date}T${nextEvent.time || '10:00'}:00`).toISOString();

			const response = await youtubeApi.createBroadcast({
				title,
				description: descParts.join('\n'),
				scheduledStartTime,
				privacyStatus: nextEvent.youtubePrivacyStatus || 'public'
			});

			// Update the event with the YouTube broadcast ID
			await eventStore.updateEvent(nextEvent.id, { youtubeScheduledId: response.id });

			toast({
				title: $_('toasts.eventScheduled.title'),
				description: $_('toasts.eventScheduled.description'),
				variant: 'success'
			});
		} catch (error) {
			toast({
				title: $_('toasts.error.title'),
				description: error instanceof Error ? error.message : 'Failed to schedule event',
				variant: 'error'
			});
		} finally {
			isScheduling = false;
		}
	}

	// Open YouTube Studio for the scheduled event
	async function openYoutubeStudio() {
		if (nextEvent?.youtubeScheduledId) {
			const url = youtubeApi.getYoutubeStudioUrl(nextEvent.youtubeScheduledId);
			if (isTauri) {
				try {
					const { openUrl } = await import('@tauri-apps/plugin-opener');
					await openUrl(url);
				} catch (err) {
					console.error('Failed to open URL with Tauri opener:', err);
					// Fallback to window.open
					window.open(url, '_blank');
				}
			} else {
				window.open(url, '_blank');
			}
		}
	}

	const navItems = [
		{ id: '/events', labelKey: 'sidebar.nav.events', icon: CalendarDays },
		{ id: '/rf-ir', labelKey: 'sidebar.nav.remoteControl', icon: Radio },
		{ id: '/obs-config', labelKey: 'sidebar.nav.settings', icon: Settings },
	];
</script>

 <!-- Mobile menu button -->
 <Button
 	buttonVariant="outline"
 	buttonSize="icon"
 	className="fixed top-4 left-4 z-50 md:hidden bg-background shadow-md hover:bg-accent border-2 active:scale-95 transition-transform"
 	onclick={() => {
 		console.log('[Mobile Menu] Button clicked');
 		onMobileMenuToggle();
 	}}
 	type="button"
 	aria-label="Toggle mobile menu"
 >
 	<Menu class="h-5 w-5" />
 </Button>

 <!-- Mobile sidebar overlay -->
 {#if isMobileMenuOpen}
 	<div class="fixed inset-0 z-40 bg-black/50 md:hidden" onclick={onMobileMenuToggle} onkeydown={(e) => e.key === 'Enter' && onMobileMenuToggle()} role="button" tabindex="0"></div>
 {/if}

 <!-- Sidebar -->
 <aside
 	class={cn(
 		"fixed inset-y-0 left-0 z-70 w-72 transform transition-transform duration-300 ease-in-out md:relative md:translate-x-0".split(' '),
 		isMobileMenuOpen ? "translate-x-0" : "-translate-x-full",
 	)}
 >
	<div class="flex h-full flex-col bg-sidebar">
 		<div class="flex items-center justify-between">
 			<Button buttonVariant="ghost" buttonSize="icon" className="md:hidden" onclick={onMobileMenuToggle}>
 				<X class="h-5 w-5" />
 			</Button>
 		</div>

		<div class="flex-1 overflow-y-auto p-4 space-y-6">
			<nav class="space-y-2">
				{#each navItems as item}
					{@const Icon = item.icon}
					<Button
						buttonVariant={page.url.pathname === item.id ? "secondary" : "ghost"}
						className="w-full justify-start"
						href={item.id}
						onclick={() => {
							if (isMobileMenuOpen) onMobileMenuToggle();
						}}
					>
						<Icon class="mr-2 h-4 w-4" />
						{$_(item.labelKey)}
					</Button>
				{/each}
			</nav>

			<Separator />

			<Card className="p-4">
				<h3 class="font-medium text-sm mb-3 text-card-foreground">{$_('sidebar.systemStatus.title')}</h3>
				<div class="space-y-1">
					<div class="flex items-center justify-between py-2">
						<span class="text-sm text-muted-foreground">{$_('sidebar.systemStatus.obsRunning')}</span>
						{#if $systemStore.obsLoading}
							<Loader2 class="h-4 w-4 text-blue-600 animate-spin" />
						{:else if $systemStore.obs}
							<CheckCircle2 class="h-4 w-4 text-green-600" />
						{:else}
							<XCircle class="h-4 w-4 text-red-600" />
						{/if}
					</div>

					<div class="flex items-center justify-between py-2">
						<span class="text-sm text-muted-foreground">{$_('sidebar.systemStatus.youtubeLoggedIn')}</span>
						{#if $systemStore.youtubeLoggedIn}
							<CheckCircle2 class="h-4 w-4 text-green-600" />
						{:else}
							<XCircle class="h-4 w-4 text-red-600" />
						{/if}
					</div>

					<!-- Dynamic OBS Device Status Rows -->
					{#each $requiredDeviceConfigs as device (device.id)}
						{@const status = $obsDeviceStatuses.get(device.id)}
						<div class="flex items-center justify-between py-2">
							<span class="text-sm text-muted-foreground">{device.name}</span>
							{#if $isCheckingDevices && !status}
								<Loader2 class="h-4 w-4 text-blue-600 animate-spin" />
							{:else if status?.available && status?.assigned}
								<CheckCircle2 class="h-4 w-4 text-green-600" />
							{:else}
								<XCircle class="h-4 w-4 text-red-600" />
							{/if}
						</div>
					{/each}
				</div>

				<!-- YouTube Re-auth Alert -->
				{#if $youtubeReauthRequired}
					<Alert variant="warning" className="mt-3">
						<AlertCircle class="h-4 w-4" />
						<AlertTitle>{$_('upload.reauth.title') || 'Re-login Required'}</AlertTitle>
						<AlertDescription className="space-y-2">
							<p class="text-xs">{$youtubeReauthError || $_('upload.reauth.description') || 'YouTube session expired'}</p>
							<Button
								buttonSize="sm"
								buttonVariant="outline"
								className="w-full"
								onclick={() => {
									youtubeAuthStatusStore.clearReauthRequired();
									showYoutubeLoginModal = true;
								}}
							>
								<LogIn class="h-3 w-3 mr-1" />
								{$_('upload.reauth.action') || 'Re-login to YouTube'}
							</Button>
						</AlertDescription>
					</Alert>
				{/if}
			</Card>

			<!-- Upload Status Section (active session + pending uploads) -->
			{#if isTauri}
				<UploadStatusSection />
			{/if}

			<Card className="p-4">
				<div class="flex items-center justify-between mb-3">
					<h3 class="font-medium text-sm text-card-foreground">{$_('sidebar.upcomingEvent.title')}</h3>
					{#if nextEvent && isToday}
						<Badge variant="success" className="text-xs">{$_('sidebar.upcomingEvent.today')}</Badge>
					{/if}
				</div>

				{#if nextEvent}
					<div class="space-y-3">
						<!-- Streaming Controls (only show for today's event) -->
						{#if isToday}
							<SidebarStreamingControls event={nextEvent} />
						{/if}

						<!-- Event Title -->
						<p class="text-sm font-medium text-card-foreground line-clamp-2">{nextEvent.title}</p>

						<!-- Scheduled Date -->
						<div>
							<span class="text-xs text-muted-foreground">{$_('sidebar.upcomingEvent.scheduledDate')}</span>
							<p class="text-sm text-card-foreground">{formatEventDate(nextEvent.date)}</p>
						</div>

						<!-- Textus (if present) -->
						{#if nextEvent.textus}
							<div>
								<span class="text-xs text-muted-foreground">{$_('sidebar.upcomingEvent.textus')}</span>
								<p class="text-sm font-medium text-card-foreground">{nextEvent.textus}</p>
							</div>
						{/if}

						<!-- Leckio (if present) -->
						{#if nextEvent.leckio}
							<div>
								<span class="text-xs text-muted-foreground">{$_('sidebar.upcomingEvent.leckio')}</span>
								<p class="text-sm font-medium text-card-foreground">{nextEvent.leckio}</p>
							</div>
						{/if}

						<!-- PPTX Generation Buttons -->
						{#if nextEvent.textus || nextEvent.leckio}
							<div class="pt-2 border-t border-border">
								<span class="text-xs text-muted-foreground mb-2 block">{$_('sidebar.upcomingEvent.pptx.title')}</span>
								<div class="flex gap-2">
									{#if nextEvent.textus}
										<Button
											buttonVariant="outline"
											buttonSize="sm"
											className="flex-1"
											onclick={() => handleGeneratePptx('textus')}
											disabled={generatingTextus || !nextEvent.textusVerses?.length}
										>
											{#if generatingTextus}
												<Loader2 class="h-4 w-4 mr-1 animate-spin" />
											{:else if nextEvent.textusGeneratedAt}
												<Check class="h-4 w-4 mr-1 text-green-600" />
											{:else}
												<FileText class="h-4 w-4 mr-1" />
											{/if}
											Textus
										</Button>
									{/if}
									{#if nextEvent.leckio}
										<Button
											buttonVariant="outline"
											buttonSize="sm"
											className="flex-1"
											onclick={() => handleGeneratePptx('leckio')}
											disabled={generatingLeckio || !nextEvent.leckioVerses?.length}
										>
											{#if generatingLeckio}
												<Loader2 class="h-4 w-4 mr-1 animate-spin" />
											{:else if nextEvent.leckioGeneratedAt}
												<Check class="h-4 w-4 mr-1 text-green-600" />
											{:else}
												<FileText class="h-4 w-4 mr-1" />
											{/if}
											Lekció
										</Button>
									{/if}
								</div>
								{#if isTauri && pptxOutputPath}
									<button
										type="button"
										onclick={() => openFolder(pptxOutputPath)}
										class="flex items-center gap-1 mt-2 text-xs text-muted-foreground hover:text-foreground transition-colors cursor-pointer"
									>
										<FolderOpen class="h-3 w-3" />
										<span class="truncate">{pptxOutputPath}</span>
									</button>
								{/if}
								{#if isTauri}
									<Button
										buttonVariant="ghost"
										buttonSize="sm"
										className="w-full mt-1 text-xs"
										onclick={handleChangeFolder}
									>
										{$_('sidebar.upcomingEvent.pptx.changeFolder')}
									</Button>
								{/if}
							</div>
						{/if}

						<!-- YouTube Schedule Status -->
						<div class="pt-2 border-t border-border">
							{#if isScheduledOnYoutube}
								<!-- Scheduled - show status (clickable to open YouTube Studio) -->
								<button
									type="button"
									onclick={openYoutubeStudio}
									class="flex items-center gap-2 w-full text-left hover:bg-muted/50 rounded p-1 -m-1 transition-colors"
								>
									<CheckCircle2 class="h-4 w-4 text-green-600 shrink-0" />
									<span class="text-sm text-green-600">{$_('sidebar.upcomingEvent.youtube.scheduled')}</span>
								</button>
							{:else if $systemStore.youtubeLoggedIn}
								<!-- Not scheduled but logged in - show schedule button -->
								<div class="space-y-1">
									<div class="flex items-center gap-2">
										<XCircle class="h-4 w-4 text-muted-foreground shrink-0" />
										<span class="text-sm text-muted-foreground">{$_('sidebar.upcomingEvent.youtube.notScheduled')}</span>
									</div>
									<Button
										buttonVariant="outline"
										buttonSize="sm"
										className="w-full"
										onclick={scheduleUpcomingEvent}
										disabled={isScheduling}
									>
										{#if isScheduling}
											<Loader2 class="h-4 w-4 mr-2 animate-spin" />
											{$_('sidebar.upcomingEvent.youtube.scheduling')}
										{:else}
											<Youtube class="h-4 w-4 mr-2" />
											{$_('sidebar.upcomingEvent.youtube.schedule')}
										{/if}
									</Button>
								</div>
							{:else}
								<!-- Not logged in - show login button -->
								<div class="space-y-1">
									<div class="flex items-center gap-2">
										<XCircle class="h-4 w-4 text-muted-foreground shrink-0" />
										<span class="text-sm text-muted-foreground">{$_('sidebar.upcomingEvent.youtube.notScheduled')}</span>
									</div>
									<Button
										buttonVariant="outline"
										buttonSize="sm"
										className="w-full"
										onclick={() => showYoutubeLoginModal = true}
									>
										<LogIn class="h-4 w-4 mr-2" />
										{$_('sidebar.upcomingEvent.youtube.loginFirst')}
									</Button>
								</div>
							{/if}
						</div>

						<!-- Browser Source Status -->
						{#if $browserSourceConfigs.length > 0}
							<div class="pt-2 border-t border-border">
								<span class="text-xs text-muted-foreground mb-2 block">{$_('sidebar.upcomingEvent.browserSources')}</span>
								{#each $browserSourceConfigs as config (config.id)}
									{@const status = $obsBrowserStatuses.get(config.id)}
									<div class="flex items-center justify-between py-1">
										<span class="text-sm text-muted-foreground">{config.name}</span>
										{#if status?.refreshPending}
											<Loader2 class="h-4 w-4 text-blue-600 animate-spin" />
										{:else if status?.matches}
											<CheckCircle2 class="h-4 w-4 text-green-600" />
										{:else if status?.refreshSuccess === false}
											<button
												type="button"
												onclick={() => manualRefreshBrowserSource(config.id)}
												class="p-1 hover:bg-muted rounded transition-colors"
												title={$_('sidebar.upcomingEvent.refreshBrowserSource')}
											>
												<RefreshCw class="h-4 w-4 text-amber-600" />
											</button>
										{:else}
											<button
												type="button"
												onclick={() => manualRefreshBrowserSource(config.id)}
												class="p-1 hover:bg-muted rounded transition-colors"
												title={$_('sidebar.upcomingEvent.refreshBrowserSource')}
											>
												<RefreshCw class="h-4 w-4 text-amber-600" />
											</button>
										{/if}
									</div>
								{/each}
							</div>
						{/if}

						<!-- Edit Button -->
						<Button
							buttonVariant="outline"
							buttonSize="sm"
							className="w-full"
							href={`/events/${nextEvent.id}`}
							onclick={() => {
								if (isMobileMenuOpen) onMobileMenuToggle();
							}}
						>
							<Edit class="h-4 w-4 mr-2" />
							{$_('sidebar.upcomingEvent.edit')}
						</Button>
					</div>
				{:else}
					<!-- Empty State -->
					<div class="text-center py-4">
						<p class="text-sm text-muted-foreground">{$_('sidebar.upcomingEvent.noUpcoming.title')}</p>
						<p class="text-xs text-muted-foreground mt-1">{$_('sidebar.upcomingEvent.noUpcoming.description')}</p>
					</div>
				{/if}
			</Card>

			<Separator />

			<!-- Language Switcher -->
			<div class="space-y-2">
				<div class="flex items-center gap-2">
					<Globe class="h-4 w-4 text-muted-foreground" />
					<span class="text-sm text-muted-foreground">{$_('sidebar.language')}</span>
				</div>
				<div class="flex gap-2">
					<Button
						buttonVariant={$locale === 'en' ? 'default' : 'outline'}
						buttonSize="sm"
						className="flex-1"
						onclick={() => setLocale('en')}
					>
						EN
					</Button>
					<Button
						buttonVariant={$locale === 'hu' ? 'default' : 'outline'}
						buttonSize="sm"
						className="flex-1"
						onclick={() => setLocale('hu')}
					>
						HU
					</Button>
				</div>
			</div>

			<!-- Theme Switcher -->
			<div class="space-y-2">
				<div class="flex items-center gap-2">
					<Sun class="h-4 w-4 text-muted-foreground" />
					<span class="text-sm text-muted-foreground">{$_('sidebar.theme')}</span>
				</div>
				<div class="flex gap-2">
					<Button
						buttonVariant={$theme === 'system' ? 'default' : 'outline'}
						buttonSize="sm"
						className="flex-1"
						onclick={() => setTheme('system')}
						title={$_('sidebar.themeSystem')}
					>
						<Monitor class="h-4 w-4" />
					</Button>
					<Button
						buttonVariant={$theme === 'light' ? 'default' : 'outline'}
						buttonSize="sm"
						className="flex-1"
						onclick={() => setTheme('light')}
						title={$_('sidebar.themeLight')}
					>
						<Sun class="h-4 w-4" />
					</Button>
					<Button
						buttonVariant={$theme === 'dark' ? 'default' : 'outline'}
						buttonSize="sm"
						className="flex-1"
						onclick={() => setTheme('dark')}
						title={$_('sidebar.themeDark')}
					>
						<Moon class="h-4 w-4" />
					</Button>
				</div>
			</div>
		</div>
	</div>
</aside>

<!-- YouTube Login Modal -->
<YouTubeLoginModal
	open={showYoutubeLoginModal}
	onClose={() => showYoutubeLoginModal = false}
	onSuccess={() => showYoutubeLoginModal = false}
/>