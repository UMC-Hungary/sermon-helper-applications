<script lang="ts">
	import { serverUrl } from '$lib/stores/server-url.js';
	import { streamPreviewEnabled } from '$lib/stores/stream-preview.js';

	const HLS_PORT = 8888;
	const STREAM_PATH = 'live';

	type ConnState = 'loading' | 'playing' | 'offline';

	// ── Position / drag ─────────────────────────────────────────────────────────
	let x = $state(16);
	let y = $state(16);
	let dragging = $state(false);
	let dragOffsetX = 0;
	let dragOffsetY = 0;

	function onPointerDown(e: PointerEvent) {
		if ((e.target as HTMLElement).closest('button, input')) return;
		e.preventDefault();
		dragging = true;
		dragOffsetX = e.clientX - x;
		dragOffsetY = e.clientY - y;
		(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
	}

	function onPointerMove(e: PointerEvent) {
		if (!dragging) return;
		x = Math.max(0, Math.min(e.clientX - dragOffsetX, window.innerWidth - 320));
		y = Math.max(0, Math.min(e.clientY - dragOffsetY, window.innerHeight - 48));
	}

	function onPointerUp(e: PointerEvent) {
		if (!dragging) return;
		dragging = false;
		(e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId);
	}

	// ── Player state ─────────────────────────────────────────────────────────────
	let minimized = $state(false);
	let muted = $state(true); // start muted so autoplay is always allowed
	let volume = $state(1.0);
	let connState = $state<ConnState>('loading');
	let videoEl = $state<HTMLVideoElement | null>(null);
	let retryTimer: ReturnType<typeof setTimeout> | null = null;

	// Sync volume/mute to the video element whenever they change.
	$effect(() => {
		if (videoEl) {
			videoEl.muted = muted;
			videoEl.volume = volume;
		}
	});

	// ── HLS URL ──────────────────────────────────────────────────────────────────
	function hlsUrl(base: string): string {
		try {
			const { hostname } = new URL(base);
			return `http://${hostname}:${HLS_PORT}/${STREAM_PATH}/index.m3u8`;
		} catch {
			return `http://localhost:${HLS_PORT}/${STREAM_PATH}/index.m3u8`;
		}
	}

	// ── Video event handlers ──────────────────────────────────────────────────────
	function onCanPlay() {
		connState = 'playing';
		if (retryTimer !== null) {
			clearTimeout(retryTimer);
			retryTimer = null;
		}
		void videoEl?.play().catch(() => {});
	}

	function onError() {
		connState = 'offline';
		scheduleRetry();
	}

	function onEnded() {
		connState = 'offline';
		scheduleRetry();
	}

	function scheduleRetry() {
		if (retryTimer !== null) return;
		retryTimer = setTimeout(() => {
			retryTimer = null;
			reload();
		}, 4000);
	}

	function reload() {
		if (!videoEl) return;
		videoEl.src = hlsUrl($serverUrl);
		videoEl.load();
	}

	// ── Mount / unmount ──────────────────────────────────────────────────────────
	$effect(() => {
		reload();
		return () => {
			if (retryTimer !== null) clearTimeout(retryTimer);
			if (videoEl) {
				videoEl.src = '';
				videoEl.load();
			}
		};
	});

	function closePlayer() {
		streamPreviewEnabled.set(false);
	}
</script>

<div
	class="player"
	class:minimized
	class:dragging
	class:playing={connState === 'playing'}
	style="left: {x}px; top: {y}px"
	onpointerdown={onPointerDown}
	onpointermove={onPointerMove}
	onpointerup={onPointerUp}
	role="region"
	aria-label="OBS Stream Preview"
>
	{#if minimized}
		<!-- ── Minimized pill ─────────────────────────────────────────────── -->
		<div class="pill">
			<span class="pill-dot dot-{connState}" aria-hidden="true"></span>
			<span class="pill-label">OBS Preview</span>
			<button class="pill-btn" onclick={() => (minimized = false)} aria-label="Expand stream preview">
				<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
					<path d="M15 3h6v6M9 21H3v-6M21 3l-7 7M3 21l7-7" />
				</svg>
			</button>
			<button class="pill-btn pill-close" onclick={closePlayer} aria-label="Close stream preview">
				<svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" aria-hidden="true">
					<path d="M18 6L6 18M6 6l12 12" />
				</svg>
			</button>
		</div>
	{:else}
		<!-- ── Full player ────────────────────────────────────────────────── -->
		<div class="player-body">
			<!-- Video area -->
			<div class="video-wrap">
				<!-- svelte-ignore a11y_media_has_caption -->
				<video
					bind:this={videoEl}
					class:hidden={connState !== 'playing'}
					autoplay
					playsinline
					oncanplay={onCanPlay}
					onerror={onError}
					onended={onEnded}
					aria-label="OBS stream video"
				></video>

				{#if connState !== 'playing'}
					<div class="overlay-placeholder">
						{#if connState === 'loading'}
							<div class="spinner" aria-hidden="true"></div>
							<span class="overlay-text">Connecting…</span>
						{:else}
							<span class="overlay-text">No stream incoming</span>
							<button class="overlay-btn" onclick={reload}>Retry</button>
						{/if}
					</div>
				{/if}

				<!-- Status dot top-right -->
				<span class="status-dot dot-{connState}" aria-hidden="true"></span>
			</div>

			<!-- Controls overlay (bottom) -->
			<div class="controls">
				<div class="controls-left">
					<button
						class="ctrl-btn"
						onclick={() => (muted = !muted)}
						aria-label={muted ? 'Unmute' : 'Mute'}
					>
						{#if muted || volume === 0}
							<svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
								<path d="M16.5 12c0-1.77-1.02-3.29-2.5-4.03v2.21l2.45 2.45c.03-.2.05-.41.05-.63zm2.5 0c0 .94-.2 1.82-.54 2.64l1.51 1.51C20.63 14.91 21 13.5 21 12c0-4.28-2.99-7.86-7-8.77v2.06c2.89.86 5 3.54 5 6.71zM4.27 3L3 4.27 7.73 9H3v6h4l5 5v-6.73l4.25 4.25c-.67.52-1.42.93-2.25 1.18v2.06c1.38-.31 2.63-.95 3.69-1.81L19.73 21 21 19.73l-9-9L4.27 3zM12 4L9.91 6.09 12 8.18V4z" />
							</svg>
						{:else if volume < 0.5}
							<svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
								<path d="M18.5 12c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02zM5 9v6h4l5 5V4L9 9H5z" />
							</svg>
						{:else}
							<svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
								<path d="M3 9v6h4l5 5V4L7 9H3zm13.5 3c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02zM14 3.23v2.06c2.89.86 5 3.54 5 6.71s-2.11 5.85-5 6.71v2.06c4.01-.91 7-4.49 7-8.77s-2.99-7.86-7-8.77z" />
							</svg>
						{/if}
					</button>
					<input
						class="vol-slider"
						type="range"
						min="0"
						max="1"
						step="0.05"
						bind:value={volume}
						oninput={() => {
							if (volume > 0) muted = false;
						}}
						aria-label="Volume"
					/>
				</div>

				<div class="controls-right">
					<button class="ctrl-btn" onclick={() => (minimized = true)} aria-label="Minimize stream preview">
						<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
							<path d="M8 3v3a2 2 0 0 1-2 2H3m18 0h-3a2 2 0 0 1-2-2V3m0 18v-3a2 2 0 0 1 2-2h3M3 16h3a2 2 0 0 1 2 2v3" />
						</svg>
					</button>
					<button class="ctrl-btn ctrl-close" onclick={closePlayer} aria-label="Close stream preview">
						<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" aria-hidden="true">
							<path d="M18 6L6 18M6 6l12 12" />
						</svg>
					</button>
				</div>
			</div>
		</div>
	{/if}
</div>

<style>
	/* ── Container ───────────────────────────────────────────────────────────── */
	.player {
		position: fixed;
		z-index: 1000;
		width: 320px;
		border-radius: 12px;
		box-shadow: 0 8px 32px rgba(0, 0, 0, 0.45), 0 2px 8px rgba(0, 0, 0, 0.3);
		touch-action: none;
		cursor: grab;
		user-select: none;
	}

	.player.dragging {
		cursor: grabbing;
		box-shadow: 0 16px 48px rgba(0, 0, 0, 0.55), 0 4px 12px rgba(0, 0, 0, 0.4);
	}

	/* ── Minimized pill ──────────────────────────────────────────────────────── */
	.pill {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 10px;
		background: rgba(15, 15, 15, 0.92);
		backdrop-filter: blur(8px);
		-webkit-backdrop-filter: blur(8px);
		border-radius: 12px;
		color: #fff;
	}

	.pill-label {
		font-size: 0.75rem;
		font-weight: 600;
		flex: 1;
		white-space: nowrap;
	}

	.pill-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 22px;
		height: 22px;
		background: rgba(255, 255, 255, 0.12);
		border: none;
		border-radius: 6px;
		color: #fff;
		cursor: pointer;
		padding: 0;
		flex-shrink: 0;
		transition: background 0.15s;
	}

	.pill-btn:hover {
		background: rgba(255, 255, 255, 0.22);
	}

	.pill-close:hover {
		background: rgba(239, 68, 68, 0.4);
	}

	/* ── Full player body ────────────────────────────────────────────────────── */
	.player-body {
		position: relative;
		border-radius: 12px;
		overflow: hidden;
		background: #000;
	}

	/* ── Video area ──────────────────────────────────────────────────────────── */
	.video-wrap {
		position: relative;
		width: 320px;
		height: 180px;
		background: #111;
	}

	video {
		width: 100%;
		height: 100%;
		object-fit: contain;
		display: block;
	}

	video.hidden {
		visibility: hidden;
	}

	/* ── Placeholder overlay ─────────────────────────────────────────────────── */
	.overlay-placeholder {
		position: absolute;
		inset: 0;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 10px;
		background: #111;
	}

	.overlay-text {
		font-size: 0.75rem;
		color: #9ca3af;
		text-align: center;
		padding: 0 16px;
	}

	.overlay-btn {
		padding: 5px 14px;
		background: rgba(255, 255, 255, 0.15);
		border: 1px solid rgba(255, 255, 255, 0.2);
		border-radius: 9999px;
		color: #fff;
		font-size: 0.75rem;
		font-weight: 600;
		cursor: pointer;
		transition: background 0.15s;
	}

	.overlay-btn:hover {
		background: rgba(255, 255, 255, 0.25);
	}

	/* ── Spinner ─────────────────────────────────────────────────────────────── */
	.spinner {
		width: 24px;
		height: 24px;
		border: 2px solid rgba(255, 255, 255, 0.15);
		border-top-color: #fff;
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	/* ── Status dot (top-right of video) ─────────────────────────────────────── */
	.status-dot {
		position: absolute;
		top: 8px;
		right: 8px;
		width: 8px;
		height: 8px;
		border-radius: 50%;
		border: 1.5px solid rgba(0, 0, 0, 0.4);
	}

	/* Shared dot colours */
	.dot-loading {
		background: #fbbf24;
		animation: pulse 1.2s ease-in-out infinite;
	}

	.dot-playing {
		background: #34d399;
	}

	.dot-offline {
		background: #6b7280;
	}

	@keyframes pulse {
		0%, 100% { opacity: 1; }
		50% { opacity: 0.4; }
	}

	/* Pill dot is slightly smaller */
	.pill-dot {
		width: 7px;
		height: 7px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	/* ── Controls bar ────────────────────────────────────────────────────────── */
	.controls {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 6px 8px;
		background: rgba(0, 0, 0, 0.85);
		backdrop-filter: blur(4px);
		-webkit-backdrop-filter: blur(4px);
		opacity: 0;
		transition: opacity 0.2s;
	}

	/* Show controls on hover, or when not playing (so user can act) */
	.player-body:hover .controls,
	.player:not(.playing) .controls {
		opacity: 1;
	}

	.controls-left,
	.controls-right {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.ctrl-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		background: transparent;
		border: none;
		border-radius: 6px;
		color: #fff;
		cursor: pointer;
		padding: 0;
		flex-shrink: 0;
		transition: background 0.15s;
	}

	.ctrl-btn:hover {
		background: rgba(255, 255, 255, 0.15);
	}

	.ctrl-close:hover {
		background: rgba(239, 68, 68, 0.5);
	}

	/* ── Volume slider ───────────────────────────────────────────────────────── */
	.vol-slider {
		-webkit-appearance: none;
		appearance: none;
		width: 72px;
		height: 3px;
		border-radius: 9999px;
		background: rgba(255, 255, 255, 0.3);
		outline: none;
		cursor: pointer;
	}

	.vol-slider::-webkit-slider-thumb {
		-webkit-appearance: none;
		appearance: none;
		width: 12px;
		height: 12px;
		border-radius: 50%;
		background: #fff;
		cursor: pointer;
	}

	.vol-slider::-moz-range-thumb {
		width: 12px;
		height: 12px;
		border-radius: 50%;
		border: none;
		background: #fff;
		cursor: pointer;
	}
</style>
