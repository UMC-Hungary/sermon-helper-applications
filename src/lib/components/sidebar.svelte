<script lang="ts">
	import { CheckCircle2, XCircle, Menu, X, Home, Book, Youtube, Calendar, Settings } from 'lucide-svelte';
	import { cn } from '$lib/utils.js';
	import Button from '$lib/components/ui/button.svelte';
	import Card from '$lib/components/ui/card.svelte';
	import Badge from '$lib/components/ui/badge.svelte';
	import Separator from '$lib/components/ui/separator.svelte';

	type Status = "active" | "inactive" | "warning";

	export let activeView: string = 'dashboard';
	export let onViewChange: (view: "dashboard" | "bible" | "youtube-schedule" | "youtube-events" | "obs-settings") => void = () => {};
	export let isMobileMenuOpen = false;
	export let onMobileMenuToggle: () => void = () => {};
	export let systemStatus = {
		obs: true,
		rodeInterface: true,
		mainDisplay: true,
		secondaryDisplay: true,
		airplayDisplay: false,
		displayAlignment: false,
		youtubeLoggedIn: false,
	};
	export let currentSermon = {
		textus: '',
		leckio: '',
		youtubeTitle: '',
		youtubeScheduled: true,
		streamStarted: false,
	};

	const navItems = [
		{ id: 'dashboard' as const, label: 'Dashboard', icon: Home },
		{ id: 'bible' as const, label: 'Bible Editor', icon: Book },
		{ id: 'youtube-schedule' as const, label: 'Schedule Event', icon: Calendar },
		{ id: 'youtube-events' as const, label: 'YouTube Events', icon: Youtube },
		{ id: 'obs-settings' as const, label: 'OBS Settings', icon: Settings },
	];

	function handleSystemRecheck() {
		console.log('[v0] Rechecking system status...');
	}
</script>

<!-- Mobile menu button -->
<Button
	variant="outline"
	size="icon"
	class="fixed top-4 left-4 z-40 lg:hidden bg-background shadow-md"
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
			<h1 class="text-lg font-semibold text-sidebar-foreground">Church Control</h1>
			<Button variant="ghost" size="icon" class="lg:hidden" onclick={onMobileMenuToggle}>
				<X class="h-5 w-5" />
			</Button>
		</div>

		<div class="flex-1 overflow-y-auto p-4 space-y-6">
			<nav class="space-y-2">
				{#each navItems as item}
					{@const Icon = item.icon}
					<Button
						variant={activeView === item.id ? "secondary" : "ghost"}
						onclick={() => {
							onViewChange(item.id);
							if (isMobileMenuOpen) onMobileMenuToggle();
						}}
					>
						<Icon class="mr-2 h-4 w-4" />
						{item.label}
					</Button>
				{/each}
			</nav>

			<Separator />

			<Card className="p-4">
				<h3 class="font-medium text-sm mb-3 text-card-foreground">System Status</h3>
				<div class="space-y-1">
					<div class="flex items-center justify-between py-2">
						<span class="text-sm text-muted-foreground">OBS Running</span>
						{#if systemStatus.obs}
							<CheckCircle2 class="h-4 w-4 text-green-600" />
						{:else}
							<XCircle class="h-4 w-4 text-red-600" />
						{/if}
					</div>
					
					<div class="flex items-center justify-between py-2">
						<span class="text-sm text-muted-foreground">Rode Interface</span>
						{#if systemStatus.rodeInterface}
							<CheckCircle2 class="h-4 w-4 text-green-600" />
						{:else}
							<XCircle class="h-4 w-4 text-red-600" />
						{/if}
					</div>
					
					<div class="flex items-center justify-between py-2">
						<span class="text-sm text-muted-foreground">Main Display</span>
						{#if systemStatus.mainDisplay}
							<CheckCircle2 class="h-4 w-4 text-green-600" />
						{:else}
							<XCircle class="h-4 w-4 text-red-600" />
						{/if}
					</div>
					
					<div class="flex items-center justify-between py-2">
						<span class="text-sm text-muted-foreground">Secondary Display</span>
						{#if systemStatus.secondaryDisplay}
							<CheckCircle2 class="h-4 w-4 text-green-600" />
						{:else}
							<XCircle class="h-4 w-4 text-red-600" />
						{/if}
					</div>
					
					<div class="flex items-center justify-between py-2">
						<span class="text-sm text-muted-foreground">AirPlay Display</span>
						{#if systemStatus.airplayDisplay}
							<CheckCircle2 class="h-4 w-4 text-green-600" />
						{:else}
							<XCircle class="h-4 w-4 text-red-600" />
						{/if}
					</div>
					
					<div class="flex items-center justify-between py-2">
						<span class="text-sm text-muted-foreground">Display Alignment</span>
						{#if systemStatus.displayAlignment}
							<CheckCircle2 class="h-4 w-4 text-green-600" />
						{:else}
							<XCircle class="h-4 w-4 text-red-600" />
						{/if}
					</div>
					
					<div class="flex items-center justify-between py-2">
						<span class="text-sm text-muted-foreground">YouTube Logged In</span>
						{#if systemStatus.youtubeLoggedIn}
							<CheckCircle2 class="h-4 w-4 text-green-600" />
						{:else}
							<XCircle class="h-4 w-4 text-red-600" />
						{/if}
					</div>
				</div>
			</Card>

			<Card className="p-4">
				<h3 class="font-medium text-sm mb-3 text-card-foreground">Current Sermon</h3>
				<div class="space-y-3">
					<div>
						<span class="text-xs text-muted-foreground">Textus</span>
						<p class="text-sm font-medium text-card-foreground">{currentSermon.textus}</p>
					</div>
					<div>
						<span class="text-xs text-muted-foreground">Leckio</span>
						<p class="text-sm font-medium text-card-foreground">{currentSermon.leckio}</p>
					</div>
					{#if currentSermon.youtubeScheduled && currentSermon.youtubeTitle}
						<Separator />
						<div>
							<span class="text-xs text-muted-foreground">YouTube</span>
							<p class="text-sm font-medium text-card-foreground">{currentSermon.youtubeTitle}</p>
						</div>
						<Badge
							variant={currentSermon.streamStarted ? "default" : "secondary"}
							className="w-full justify-center"
						>
							{currentSermon.streamStarted ? "Stream Live" : "Scheduled"}
						</Badge>
					{/if}
					{#if !currentSermon.youtubeScheduled}
						<Badge variant="outline" className="w-full justify-center">
							Not scheduled
						</Badge>
					{/if}
				</div>
			</Card>
		</div>
	</div>
</aside>