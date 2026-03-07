<script lang="ts">
	import { Toaster } from 'svelte-sonner';
	import '$lib/i18n';
	import { _ } from 'svelte-i18n';
	import ConnectorInit from '$lib/components/layout/ConnectorInit.svelte';
	import ReLoginHandler from '$lib/components/layout/ReLoginHandler.svelte';
	import NavConnectors from '$lib/components/layout/NavConnectors.svelte';
	import NavErrorBadge from '$lib/components/layout/NavErrorBadge.svelte';
	import FloatingStreamPlayer from '$lib/components/FloatingStreamPlayer.svelte';
	import { streamPreviewEnabled } from '$lib/stores/stream-preview.js';

	let { children } = $props();
</script>

<Toaster richColors position="top-right" />
<ConnectorInit />
<ReLoginHandler />
{#if $streamPreviewEnabled}
	<FloatingStreamPlayer />
{/if}

<div class="app">
	<nav>
		<a href="/">{$_('nav.dashboard')}</a>
		<a href="/events">{$_('nav.events')}</a>
		<a href="/live-events">{$_('nav.liveEvents')}</a>
		<a href="/presentations">{$_('nav.presentations')}</a>
		<a href="/connect">{$_('nav.connect')}</a>
		<a href="/settings">{$_('nav.settings')}</a>
		<NavErrorBadge />
		<NavConnectors />
	</nav>
	<main>
		{@render children()}
	</main>
</div>

<style>
	.app {
		font-family: system-ui, sans-serif;
		max-width: 1200px;
		margin: 0 auto;
		padding: 1rem;
	}

	nav {
		display: flex;
		align-items: center;
		gap: 1rem;
		padding: 0.75rem 0;
		border-bottom: 1px solid #e5e7eb;
		margin-bottom: 1.5rem;
		flex-wrap: wrap;
	}

	nav a {
		color: #374151;
		text-decoration: none;
		font-weight: 500;
	}

	nav a:hover {
		color: #1d4ed8;
	}
</style>
