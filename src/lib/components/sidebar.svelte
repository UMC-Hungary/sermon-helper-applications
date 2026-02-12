<script lang="ts">
	import { CheckCircle2, XCircle, Menu, X, Settings, Loader2, Globe, CalendarDays, Sun, Moon, Monitor, LogIn, AlertCircle, Radio, Subtitles, Presentation } from 'lucide-svelte';
	import { cn } from '$lib/utils.js';
	import Button from '$lib/components/ui/button.svelte';
	import Card from '$lib/components/ui/card.svelte';
	import Separator from '$lib/components/ui/separator.svelte';
	import { systemStore } from '$lib/stores/system-store';
	import { page } from '$app/state';
	import { _, locale } from 'svelte-i18n';
	import { setLocale } from '$lib/i18n';
	import { theme, setTheme } from '$lib/stores/theme-store';
	import YouTubeLoginModal from '$lib/components/youtube-login-modal.svelte';
	import SidebarUpcomingEvent from '$lib/components/sidebar-upcoming-event.svelte';
	import Alert from '$lib/components/ui/alert.svelte';
	import AlertTitle from '$lib/components/ui/alert-title.svelte';
	import AlertDescription from '$lib/components/ui/alert-description.svelte';
	import { youtubeReauthRequired, youtubeReauthError, youtubeAuthStatusStore } from '$lib/stores/youtube-auth-status-store';
	import { requiredDeviceConfigs } from '$lib/stores/obs-devices-store';
	import { obsDeviceStatuses, isCheckingDevices } from '$lib/stores/obs-device-status-store';

	export let isMobileMenuOpen = false;
	export let onMobileMenuToggle: () => void = () => {};

	let showYoutubeLoginModal = false;

	const navItems = [
		{ id: '/events', labelKey: 'sidebar.nav.events', icon: CalendarDays },
		{ id: '/rf-ir', labelKey: 'sidebar.nav.remoteControl', icon: Radio },
		{ id: '/obs-caption', labelKey: 'sidebar.nav.obsCaption', icon: Subtitles },
		{ id: '/settings', labelKey: 'sidebar.nav.settings', icon: Settings },
	];

	function isNavActive(itemId: string): boolean {
		const pathname = page.url.pathname;
		if (itemId === '/settings') {
			return pathname.startsWith('/settings');
		}
		return pathname === itemId;
	}
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
						buttonVariant={isNavActive(item.id) ? "secondary" : "ghost"}
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

					<div class="flex items-center justify-between py-2">
						<span class="text-sm text-muted-foreground">
							{$systemStore.presentationApp || 'Presentation'}
						</span>
						{#if $systemStore.presentationConnected}
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

			<SidebarUpcomingEvent
				onMobileMenuClose={() => { if (isMobileMenuOpen) onMobileMenuToggle(); }}
				onLoginRequired={() => showYoutubeLoginModal = true}
			/>

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
