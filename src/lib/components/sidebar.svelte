<script lang="ts">
	import { CheckCircle2, XCircle, Menu, X, Home, Book, Youtube, Calendar, Settings, Loader2, Globe } from 'lucide-svelte';
	import { cn } from '$lib/utils.js';
	import Button from '$lib/components/ui/button.svelte';
	import Card from '$lib/components/ui/card.svelte';
	import Badge from '$lib/components/ui/badge.svelte';
	import Separator from '$lib/components/ui/separator.svelte';
	import { systemStore } from '$lib/stores/system-store';
	import { page } from '$app/state';
	import { _, locale } from 'svelte-i18n';
	import { setLocale, availableLocales } from '$lib/i18n';

	type Status = "active" | "inactive" | "warning";

	export let isMobileMenuOpen = false;
	export let onMobileMenuToggle: () => void = () => {};
	export let currentSermon = {
		textus: '',
		leckio: '',
		youtubeTitle: '',
		youtubeScheduled: true,
		streamStarted: false,
	};

	const navItems = [
		{ id: '/', labelKey: 'sidebar.nav.dashboard', icon: Home },
		{ id: '/bible', labelKey: 'sidebar.nav.bibleEditor', icon: Book },
		{ id: '/youtube-schedule', labelKey: 'sidebar.nav.scheduleEvent', icon: Calendar },
		{ id: '/youtube-events', labelKey: 'sidebar.nav.youtubeEvents', icon: Youtube },
		{ id: '/obs-settings', labelKey: 'sidebar.nav.obsSettings', icon: Settings },
	];

	function handleSystemRecheck() {
		console.log('[v0] Rechecking system status...');
	}

	function handleLocaleChange(event: Event) {
		const target = event.target as HTMLSelectElement;
		setLocale(target.value);
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
				<h3 class="font-medium text-sm mb-3 text-card-foreground">{$_('sidebar.currentSermon.title')}</h3>
				<div class="space-y-3">
					<div>
						<span class="text-xs text-muted-foreground">{$_('sidebar.currentSermon.textus')}</span>
						<p class="text-sm font-medium text-card-foreground">{currentSermon.textus}</p>
					</div>
					<div>
						<span class="text-xs text-muted-foreground">{$_('sidebar.currentSermon.leckio')}</span>
						<p class="text-sm font-medium text-card-foreground">{currentSermon.leckio}</p>
					</div>
					{#if currentSermon.youtubeScheduled && currentSermon.youtubeTitle}
						<Separator />
						<div>
							<span class="text-xs text-muted-foreground">{$_('sidebar.currentSermon.youtube')}</span>
							<p class="text-sm font-medium text-card-foreground">{currentSermon.youtubeTitle}</p>
						</div>
						<Badge
							variant={currentSermon.streamStarted ? "default" : "secondary"}
							className="w-full justify-center"
						>
							{currentSermon.streamStarted ? $_('sidebar.currentSermon.streamLive') : $_('sidebar.currentSermon.scheduled')}
						</Badge>
					{/if}
					{#if !currentSermon.youtubeScheduled}
						<Badge variant="outline" className="w-full justify-center">
							{$_('sidebar.currentSermon.notScheduled')}
						</Badge>
					{/if}
				</div>
			</Card>

			<Separator />

			<!-- Language Switcher -->
			<div class="flex items-center gap-2">
				<Globe class="h-4 w-4 text-muted-foreground" />
				<select
					class="flex-1 bg-sidebar border border-sidebar-border rounded-md px-3 py-2 text-sm text-sidebar-foreground focus:outline-none focus:ring-2 focus:ring-sidebar-ring"
					value={$locale}
					onchange={handleLocaleChange}
				>
					{#each availableLocales as loc}
						<option value={loc.code}>{loc.flag} {loc.name}</option>
					{/each}
				</select>
			</div>
		</div>
	</div>
</aside>