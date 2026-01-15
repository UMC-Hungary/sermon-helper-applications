<script lang="ts">
	import { CheckCircle2, XCircle, Menu, X, Home, Youtube, Calendar, Settings, Loader2, Globe, CalendarDays, Sun, Moon, Monitor, Edit, LogIn } from 'lucide-svelte';
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
	function openYoutubeStudio() {
		if (nextEvent?.youtubeScheduledId) {
			const url = youtubeApi.getYoutubeStudioUrl(nextEvent.youtubeScheduledId);
			window.open(url, '_blank');
		}
	}

	const navItems = [
		{ id: '/', labelKey: 'sidebar.nav.dashboard', icon: Home },
		{ id: '/events', labelKey: 'sidebar.nav.events', icon: CalendarDays },
		{ id: '/youtube-schedule', labelKey: 'sidebar.nav.scheduleEvent', icon: Calendar },
		{ id: '/youtube-events', labelKey: 'sidebar.nav.youtubeEvents', icon: Youtube },
		{ id: '/obs-settings', labelKey: 'sidebar.nav.obsSettings', icon: Settings },
	];

	function handleSystemRecheck() {
		console.log('[v0] Rechecking system status...');
	}
</script>

<!-- Mobile menu button -->
<Button
	buttonVariant="outline"
	buttonSize="icon"
	className="fixed top-4 left-4 z-40 lg:hidden bg-background shadow-md"
	onclick={onMobileMenuToggle}
>
	<Menu class="h-5 w-5" />
</Button>

<!-- Mobile sidebar overlay -->
{#if isMobileMenuOpen}
	<div class="fixed inset-0 z-50 bg-black/50 lg:hidden" onclick={onMobileMenuToggle} onkeydown={(e) => e.key === 'Enter' && onMobileMenuToggle()} role="button" tabindex="0"></div>
{/if}

<!-- Sidebar -->
<aside
	class={cn(
		"fixed inset-y-0 left-0 z-[60] w-72 transform transition-transform duration-200 lg:relative lg:translate-x-0".split(' '),
		isMobileMenuOpen ? "translate-x-0" : "-translate-x-full",
	)}
>
	<div class="flex h-full flex-col bg-sidebar">
		<div class="flex items-center justify-between border-b border-sidebar-border p-4">
			<h1 class="text-lg font-semibold text-sidebar-foreground">{$_('sidebar.appTitle')}</h1>
			<Button buttonVariant="ghost" buttonSize="icon" className="lg:hidden" onclick={onMobileMenuToggle}>
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
						<span class="text-sm text-muted-foreground">{$_('sidebar.systemStatus.rodeInterface')}</span>
						{#if $systemStore.rodeInterface}
							<CheckCircle2 class="h-4 w-4 text-green-600" />
						{:else}
							<XCircle class="h-4 w-4 text-red-600" />
						{/if}
					</div>

					<div class="flex items-center justify-between py-2">
						<span class="text-sm text-muted-foreground">{$_('sidebar.systemStatus.mainDisplay')}</span>
						{#if $systemStore.mainDisplay}
							<CheckCircle2 class="h-4 w-4 text-green-600" />
						{:else}
							<XCircle class="h-4 w-4 text-red-600" />
						{/if}
					</div>

					<div class="flex items-center justify-between py-2">
						<span class="text-sm text-muted-foreground">{$_('sidebar.systemStatus.secondaryDisplay')}</span>
						{#if $systemStore.secondaryDisplay}
							<CheckCircle2 class="h-4 w-4 text-green-600" />
						{:else}
							<XCircle class="h-4 w-4 text-red-600" />
						{/if}
					</div>

					<div class="flex items-center justify-between py-2">
						<span class="text-sm text-muted-foreground">{$_('sidebar.systemStatus.airplayDisplay')}</span>
						{#if $systemStore.airplayDisplay}
							<CheckCircle2 class="h-4 w-4 text-green-600" />
						{:else}
							<XCircle class="h-4 w-4 text-red-600" />
						{/if}
					</div>

					<div class="flex items-center justify-between py-2">
						<span class="text-sm text-muted-foreground">{$_('sidebar.systemStatus.displayAlignment')}</span>
						{#if $systemStore.displayAlignment}
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
				</div>
			</Card>

			<Card className="p-4">
				<div class="flex items-center justify-between mb-3">
					<h3 class="font-medium text-sm text-card-foreground">{$_('sidebar.upcomingEvent.title')}</h3>
					{#if nextEvent && isToday}
						<Badge variant="success" className="text-xs">{$_('sidebar.upcomingEvent.today')}</Badge>
					{/if}
				</div>

				{#if nextEvent}
					<div class="space-y-3">
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

						<!-- YouTube Schedule Status -->
						<div class="pt-2 border-t border-border">
							{#if isScheduledOnYoutube}
								<!-- Scheduled - show status (clickable to open YouTube Studio) -->
								<button
									type="button"
									onclick={openYoutubeStudio}
									class="flex items-center gap-2 w-full text-left hover:bg-muted/50 rounded p-1 -m-1 transition-colors"
								>
									<CheckCircle2 class="h-4 w-4 text-green-600 flex-shrink-0" />
									<span class="text-sm text-green-600">{$_('sidebar.upcomingEvent.youtube.scheduled')}</span>
								</button>
							{:else if $systemStore.youtubeLoggedIn}
								<!-- Not scheduled but logged in - show schedule button -->
								<div class="space-y-1">
									<div class="flex items-center gap-2">
										<XCircle class="h-4 w-4 text-muted-foreground flex-shrink-0" />
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
										<XCircle class="h-4 w-4 text-muted-foreground flex-shrink-0" />
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