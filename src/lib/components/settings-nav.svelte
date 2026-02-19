<script lang="ts">
	import { page } from '$app/stores';
	import { _ } from 'svelte-i18n';
	import { cn } from '$lib/utils.js';
	import { Wifi, Cpu, FileText, Settings } from 'lucide-svelte';

	const tabs = [
		{ id: '/settings/connection', labelKey: 'settings.tabs.connection', icon: Wifi },
		{ id: '/settings/devices', labelKey: 'settings.tabs.devices', icon: Cpu },
		{ id: '/settings/content', labelKey: 'settings.tabs.content', icon: FileText },
		{ id: '/settings/system', labelKey: 'settings.tabs.system', icon: Settings },
	];

	$: currentPath = $page.url.pathname;

	function isActive(tabId: string, path: string): boolean {
		return path === tabId || path.startsWith(tabId + '/');
	}
</script>

<nav class="border-b border-border mb-6">
	<div class="flex overflow-x-auto scrollbar-hide -mb-px">
		{#each tabs as tab}
			{@const Icon = tab.icon}
			{@const active = isActive(tab.id, currentPath)}
			<a
				href={tab.id}
				class={cn(
					"flex items-center gap-2 px-4 py-3 text-sm font-medium whitespace-nowrap border-b-2 transition-colors",
					"hover:text-foreground hover:border-muted-foreground/50",
					"focus:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
					active
						? "text-foreground border-primary"
						: "text-muted-foreground border-transparent"
				)}
			>
				<Icon class="h-4 w-4 shrink-0" />
				<span>{$_(tab.labelKey)}</span>
			</a>
		{/each}
	</div>
</nav>

<style>
	/* Hide scrollbar but allow scrolling on mobile */
	.scrollbar-hide {
		-ms-overflow-style: none;
		scrollbar-width: none;
	}
	.scrollbar-hide::-webkit-scrollbar {
		display: none;
	}
</style>
