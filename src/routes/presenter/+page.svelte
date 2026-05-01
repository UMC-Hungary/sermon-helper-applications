<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { page } from '$app/stores';
	import { WsMessageSchema } from '$lib/schemas/ws-messages.js';
	import type { PresenterState } from '$lib/schemas/ws-messages.js';

	// ── Standalone WS connection ──────────────────────────────────────────────

	let standaloneSocket: WebSocket | null = null;
	let standaloneState = $state<PresenterState | null>(null);
	let token = $state<string | null>(null);

	function connect() {
		// wsPort param lets the presenter page connect to the API server even when
		// the frontend is served from a different port (e.g. Vite dev server).
		const wsPort = $page.url.searchParams.get('wsPort');
		const host = wsPort
			? `${window.location.hostname}:${wsPort}`
			: window.location.host;
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

	// ── Font scaling ──────────────────────────────────────────────────────────
	// Font sizes are read directly from the PPTX (in points) and scaled to the
	// web container using the ratio of container pixels to slide EMUs.
	// 1 point = 12 700 EMU.

	const EMU_PER_PT = 12700;
	const DEFAULT_FONT_SIZE_PT = 28.0;

	let slideAreaEl = $state<HTMLElement | null>(null);
	let scaleFactor = $state(0);

	$effect(() => {
		const el = slideAreaEl;
		const state = standaloneState;

		if (!el || !state?.loaded) {
			scaleFactor = 0;
			return;
		}

		function recalc() {
			if (!el || !state) return;
			const style = getComputedStyle(el);
			const pw = parseFloat(style.paddingLeft) + parseFloat(style.paddingRight);
			const ph = parseFloat(style.paddingTop) + parseFloat(style.paddingBottom);
			const w = el.clientWidth - pw;
			const h = el.clientHeight - ph;
			if (w > 0 && h > 0) {
				scaleFactor = Math.min(
					w / state.slideWidthEmu,
					h / state.slideHeightEmu,
				);
			}
		}

		const obs = new ResizeObserver(recalc);
		obs.observe(el);
		recalc();
		return () => obs.disconnect();
	});

	function fontSizePx(fontSizePt: number): number {
		const pt = fontSizePt > 0 ? fontSizePt : DEFAULT_FONT_SIZE_PT;
		return pt * EMU_PER_PT * scaleFactor;
	}

	// ── Keyboard / navigation ─────────────────────────────────────────────────

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

	// ── Derived display values ────────────────────────────────────────────────

	const currentSlide = $derived(
		standaloneState && standaloneState.loaded && standaloneState.currentSlide > 0
			? standaloneState.slides.find((s) => s.index === standaloneState!.currentSlide) ?? null
			: null
	);

	const slideParagraphs = $derived(currentSlide?.paragraphs ?? []);
	const slideIndex = $derived(standaloneState?.currentSlide ?? 0);
	const slideTotal = $derived(standaloneState?.totalSlides ?? 0);
	const isLoaded = $derived(standaloneState?.loaded ?? false);
	const isMuted = $derived(standaloneState?.muted ?? false);

	// ── Counter paragraph detection ───────────────────────────────────────────
	// The counter (slide number / verse reference) is a center-aligned paragraph
	// whose fontSizePt is < 85 % of the maximum on the slide.  In some PPTXes
	// it comes first in the XML (its text box was inserted before the lyrics
	// text box), in others it is last — so we check both ends.

	function findCounterIdx(paras: typeof slideParagraphs): number {
		if (paras.length < 2) return -1;
		const maxPt = paras.reduce((m, p) => Math.max(m, p.fontSizePt), 0);
		if (maxPt === 0) return -1;
		const isCounter = (p: (typeof paras)[0]) =>
			p.fontSizePt > 0 && p.fontSizePt < maxPt * 0.85 && p.align === 'center';
		const first = paras[0];
		if (first && isCounter(first)) return 0;
		const lastIdx = paras.length - 1;
		const lastPara = paras[lastIdx];
		if (lastPara && isCounter(lastPara)) return lastIdx;
		return -1;
	}

	const counterIdx = $derived(findCounterIdx(slideParagraphs));
	const counterParagraph = $derived(counterIdx >= 0 ? slideParagraphs[counterIdx] : null);
	const mainParagraphs = $derived(
		counterIdx >= 0 ? slideParagraphs.filter((_, i) => i !== counterIdx) : slideParagraphs
	);
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
		<div class="slide-area" bind:this={slideAreaEl}>
			<div class="main-content" style:visibility={scaleFactor > 0 ? 'visible' : 'hidden'}>
				<div class="text-container">
					{#each mainParagraphs as para, i (i)}
						<p
							class="slide-text"
							style="text-align: {para.align}; font-size: {fontSizePx(para.fontSizePt)}px"
						>
							{#each para.lines as line, i}
								{#if i > 0}<br>{/if}{line}
							{/each}
						</p>
					{/each}
				</div>
			</div>
			{#if counterParagraph}
				<p
					class="counter-text"
					style:visibility={scaleFactor > 0 ? 'visible' : 'hidden'}
					style="font-size: {fontSizePx(counterParagraph.fontSizePt)}px"
				>
					{#each counterParagraph.lines as line, i}
						{#if i > 0}<br>{/if}{line}
					{/each}
				</p>
			{/if}
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
		flex-direction: column;
		padding: 4vw 6vw 3vw;
		min-height: 0;
		box-sizing: border-box;
	}

	.main-content {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		min-height: 0;
	}

	.text-container {
		width: 100%;
		display: flex;
		flex-direction: column;
		align-items: stretch;
		justify-content: center;
		font-family: Helvetica, Arial, sans-serif;
		font-weight: 700;
	}

	.counter-text {
		margin: 0;
		padding-top: 1vw;
		text-align: center;
		font-family: Helvetica, Arial, sans-serif;
		font-weight: 700;
		line-height: 1.2;
		color: #fff;
		width: 100%;
		flex-shrink: 0;
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

	/* Paragraph spacing proportional to each paragraph's own font size. */
	.slide-text + .slide-text {
		margin-top: 0.35em;
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
