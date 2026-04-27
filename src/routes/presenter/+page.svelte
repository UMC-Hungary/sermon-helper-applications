<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { page } from '$app/stores';
	import { fitText } from '$lib/actions/fitText.js';
	import { WsMessageSchema } from '$lib/schemas/ws-messages.js';
	import type { PresenterState } from '$lib/schemas/ws-messages.js';

	// ── Standalone WS connection ──────────────────────────────────────────────
	// The presenter page always uses its own WebSocket — it runs either in an
	// external browser or inside an iframe, never with access to the main app's
	// Svelte stores.  A token in the URL enables authenticated (navigable) mode;
	// no token gives read-only display mode.

	let standaloneSocket: WebSocket | null = null;
	let standaloneState = $state<PresenterState | null>(null);
	let token = $state<string | null>(null);

	function connect() {
		const host = window.location.host;
		const wsUrl = token
			? `ws://${host}/ws?token=${encodeURIComponent(token)}`
			: `ws://${host}/ws`;
		standaloneSocket = new WebSocket(wsUrl);

		standaloneSocket.addEventListener('message', (ev) => {
			const result = WsMessageSchema.safeParse(JSON.parse(ev.data as string));
			if (!result.success) return;
			const msg = result.data;
			if (msg.type === 'presenter.state') {
				standaloneState = msg.state;
			} else if (msg.type === 'presenter.slide_changed') {
				if (standaloneState) {
					standaloneState = { ...standaloneState, currentSlide: msg.currentSlide, totalSlides: msg.totalSlides };
				}
			} else if (msg.type === 'ping') {
				standaloneSocket?.send(JSON.stringify({ type: 'pong', ping_id: msg.pingId }));
			}
		});

		standaloneSocket.addEventListener('open', () => {
			standaloneSocket?.send(JSON.stringify({ type: 'presenter.register', label: 'Presenter Display' }));
			standaloneSocket?.send(JSON.stringify({ type: 'presenter.status' }));
		});

		standaloneSocket.addEventListener('close', () => {
			setTimeout(() => connect(), 3000);
		});
	}

	onMount(() => {
		token = $page.url.searchParams.get('token');
		connect();
		window.addEventListener('keydown', handleKey);
	});

	onDestroy(() => {
		standaloneSocket?.close();
		window.removeEventListener('keydown', handleKey);
	});

	function handleKey(e: KeyboardEvent) {
		if (e.key === 'ArrowRight' || e.key === 'ArrowDown' || e.key === ' ') {
			navigate('next');
		} else if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') {
			navigate('prev');
		} else if (e.key === 'Home') {
			navigate('first');
		} else if (e.key === 'End') {
			navigate('last');
		} else if ((e.key === 'b' || e.key === 'B') && token) {
			toggleMute();
		}
	}

	function navigate(direction: 'next' | 'prev' | 'first' | 'last') {
		standaloneSocket?.send(JSON.stringify({ type: `presenter.${direction}` }));
	}

	function toggleMute() {
		const cmd = standaloneState?.muted ? 'presenter.unmute' : 'presenter.mute';
		standaloneSocket?.send(JSON.stringify({ type: cmd }));
	}

	const currentSlide = $derived(
		standaloneState && standaloneState.loaded && standaloneState.currentSlide > 0
			? standaloneState.slides.find((s) => s.index === standaloneState!.currentSlide) ?? null
			: null
	);

	const slideParagraphs = $derived(currentSlide?.paragraphs ?? []);
	// A single string used to trigger fitText re-calculation when content changes.
	const slideFitKey = $derived(slideParagraphs.map((p) => p.text).join('\n'));
	const slideIndex = $derived(standaloneState?.currentSlide ?? 0);
	const slideTotal = $derived(standaloneState?.totalSlides ?? 0);
	const isLoaded = $derived(standaloneState?.loaded ?? false);
	const isMuted = $derived(standaloneState?.muted ?? false);
</script>

<svelte:head>
	<title>Presenter</title>
</svelte:head>

<div class="presenter-root">
	{#if isMuted}
		<div class="mute-overlay" aria-label="Display muted">
			{#if token}
				<button class="unmute-hint" onclick={toggleMute} aria-label="Unmute display">
					Click or press B to unmute
				</button>
			{/if}
		</div>
	{:else if !isLoaded}
		<div class="waiting">
			<p>Waiting for presentation…</p>
			<p class="hint">Load a .pptx file from the Presentations page to begin.</p>
		</div>
	{:else}
		<div class="slide-area">
			<div class="text-container" use:fitText={slideFitKey}>
				{#each slideParagraphs as para (para.text + para.align)}
					<p class="slide-text" style="text-align: {para.align}">
						{#each para.text.split('\n') as line, j}
							{#if j > 0}<br>{/if}{line}
						{/each}
					</p>
				{/each}
			</div>
		</div>

		{#if token}
			<div class="nav-bar">
				<button
					class="nav-btn"
					onclick={() => navigate('first')}
					aria-label="First slide"
				>⏮</button>
				<button
					class="nav-btn"
					onclick={() => navigate('prev')}
					aria-label="Previous slide"
				>◀</button>
				<span class="slide-counter">{slideIndex} / {slideTotal}</span>
				<button
					class="nav-btn"
					onclick={() => navigate('next')}
					aria-label="Next slide"
				>▶</button>
				<button
					class="nav-btn"
					onclick={() => navigate('last')}
					aria-label="Last slide"
				>⏭</button>
				<button
					class="nav-btn mute-btn"
					onclick={toggleMute}
					aria-label="Mute display"
				>⬛</button>
			</div>
		{/if}
	{/if}
</div>

<style>
	:global(html),
	:global(body) {
		margin: 0;
		padding: 0;
		background: #000;
		color: #fff;
		overflow: hidden;
		height: 100%;
		width: 100%;
	}

	.presenter-root {
		position: fixed;
		inset: 0;
		background: #000;
		display: flex;
		flex-direction: column;
		color: #fff;
	}

	/* ── Slide area ───────────────────────────────────────────────────────── */

	.slide-area {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		/* Padding is accounted for by fitText when measuring available space. */
		padding: 4vw 6vw;
		min-height: 0;
		box-sizing: border-box;
	}

	.text-container {
		width: 100%;
		display: flex;
		flex-direction: column;
		align-items: stretch;
		justify-content: center;
		gap: 0.35em;
		/* Initial font size before fitText kicks in; fitText will override. */
		font-size: clamp(1.5rem, 8vw, 12rem);
		font-family: Helvetica, Arial, sans-serif;
		font-weight: 700;
	}

	.slide-text {
		margin: 0;
		padding: 0;
		font-family: Helvetica, Arial, sans-serif;
		font-weight: 700;
		line-height: 1.2;
		color: #fff;
		width: 100%;
	}

	/* ── Navigation bar ───────────────────────────────────────────────────── */

	.nav-bar {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 0.75rem;
		padding: 0.5rem 1rem;
		background: rgba(255, 255, 255, 0.08);
		border-top: 1px solid rgba(255, 255, 255, 0.12);
		/* Hide on hover — shows on mouse move over the bar */
		opacity: 0.2;
		transition: opacity 0.2s ease;
	}

	.nav-bar:hover {
		opacity: 1;
	}

	.nav-btn {
		padding: 0.35rem 0.7rem;
		background: rgba(255, 255, 255, 0.15);
		color: #fff;
		border: 1px solid rgba(255, 255, 255, 0.3);
		border-radius: 0.25rem;
		cursor: pointer;
		font-size: 0.9rem;
		line-height: 1;
	}

	.nav-btn:hover {
		background: rgba(255, 255, 255, 0.3);
	}

	.slide-counter {
		font-size: 0.85rem;
		opacity: 0.8;
		min-width: 4rem;
		text-align: center;
	}

	/* ── Mute overlay ─────────────────────────────────────────────────────── */

	.mute-overlay {
		position: fixed;
		inset: 0;
		background: #000;
		display: flex;
		align-items: flex-end;
		justify-content: center;
		padding-bottom: 1rem;
	}

	.unmute-hint {
		background: transparent;
		border: none;
		color: rgba(255, 255, 255, 0.15);
		font-size: 0.75rem;
		cursor: pointer;
		padding: 0.5rem 1rem;
		transition: color 0.2s;
	}

	.unmute-hint:hover {
		color: rgba(255, 255, 255, 0.5);
	}

	/* ── Waiting state ────────────────────────────────────────────────────── */

	.waiting {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100%;
		gap: 0.5rem;
		opacity: 0.5;
	}

	.waiting p {
		margin: 0;
		font-size: 1.25rem;
	}

	.hint {
		font-size: 0.875rem !important;
		opacity: 0.7;
	}
</style>
