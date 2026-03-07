<script lang="ts">
	import { getStreamStats } from '$lib/api/stream.js';

	// ── State ────────────────────────────────────────────────────────────────
	let ready = $state(false);
	let tracks = $state<string[]>([]);
	let readers = $state(0);
	let inRate = $state(0); // bytes/s from OBS
	let outRate = $state(0); // bytes/s to viewers
	let totalIn = $state(0); // cumulative bytes received
	let totalOut = $state(0); // cumulative bytes sent

	// ── Helpers ──────────────────────────────────────────────────────────────
	function formatRate(bps: number): string {
		if (bps < 1024) return `${bps.toFixed(0)} B/s`;
		if (bps < 1024 * 1024) return `${(bps / 1024).toFixed(1)} KB/s`;
		return `${(bps / (1024 * 1024)).toFixed(2)} MB/s`;
	}

	function formatBytes(b: number): string {
		if (b < 1024) return `${b} B`;
		if (b < 1024 * 1024) return `${(b / 1024).toFixed(1)} KB`;
		if (b < 1024 * 1024 * 1024) return `${(b / (1024 * 1024)).toFixed(1)} MB`;
		return `${(b / (1024 * 1024 * 1024)).toFixed(2)} GB`;
	}

	// ── Polling ──────────────────────────────────────────────────────────────
	$effect(() => {
		let prevIn = 0;
		let prevOut = 0;
		let prevTs = Date.now();

		async function poll() {
			try {
				const stats = await getStreamStats();
				const now = Date.now();
				const dt = Math.max((now - prevTs) / 1000, 0.1);

				ready = stats.ready;
				tracks = stats.tracks;
				readers = stats.readers;
				totalIn = stats.bytesReceived;
				totalOut = stats.bytesSent;

				if (stats.ready) {
					inRate = Math.max(0, (stats.bytesReceived - prevIn) / dt);
					outRate = Math.max(0, (stats.bytesSent - prevOut) / dt);
				} else {
					inRate = 0;
					outRate = 0;
				}

				prevIn = stats.bytesReceived;
				prevOut = stats.bytesSent;
				prevTs = now;
			} catch {
				ready = false;
				inRate = 0;
				outRate = 0;
			}
		}

		poll();
		const id = setInterval(poll, 1000);
		return () => clearInterval(id);
	});
</script>

<div class="widget card">
	<div class="widget-header">
		<h2>Stream</h2>
		{#if ready}
			<span class="badge badge--live">
				<span class="dot" aria-hidden="true"></span>
				LIVE
			</span>
		{:else}
			<span class="badge badge--offline">Offline</span>
		{/if}
	</div>

	{#if ready}
		<p class="tracks">{tracks.join(' · ')}</p>

		<dl class="stats-dl">
			<dt class="label label--in">↓ In</dt>
			<dd>
				<span class="rate">{formatRate(inRate)}</span>
				<span class="total">({formatBytes(totalIn)})</span>
			</dd>

			<dt class="label label--out">↑ Out</dt>
			<dd>
				<span class="rate">{formatRate(outRate)}</span>
				<span class="total">({formatBytes(totalOut)})</span>
			</dd>

			<dt class="label">Viewers</dt>
			<dd>{readers}</dd>
		</dl>
	{:else}
		<p class="offline-msg">No stream incoming</p>
	{/if}
</div>

<style>
	.widget {
		padding: 1rem;
		border: 1px solid #e5e7eb;
		border-radius: 0.5rem;
	}

	.widget-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 0.5rem;
	}

	.widget-header h2 {
		margin: 0;
		font-size: 0.875rem;
		font-weight: 600;
		color: #6b7280;
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}

	.badge {
		display: inline-flex;
		align-items: center;
		gap: 0.35rem;
		padding: 0.2rem 0.6rem;
		border-radius: 9999px;
		font-size: 0.75rem;
		font-weight: 700;
		letter-spacing: 0.05em;
	}

	.badge--live {
		background: #d1fae5;
		color: #065f46;
	}

	.badge--offline {
		background: #f3f4f6;
		color: #9ca3af;
		font-weight: 500;
	}

	/* Animated pulsing dot for LIVE badge */
	.dot {
		width: 7px;
		height: 7px;
		border-radius: 50%;
		background: #10b981;
		animation: pulse 2s ease-in-out infinite;
	}

	@keyframes pulse {
		0%, 100% { opacity: 1; }
		50% { opacity: 0.35; }
	}

	.tracks {
		margin: 0 0 0.75rem;
		font-size: 0.75rem;
		color: #6b7280;
	}

	.stats-dl {
		display: grid;
		grid-template-columns: auto 1fr;
		column-gap: 0.75rem;
		row-gap: 0.4rem;
		margin: 0;
	}

	.label {
		font-size: 0.8125rem;
		color: #6b7280;
		align-self: baseline;
		white-space: nowrap;
		font-weight: 500;
	}

	.label--in  { color: #2563eb; }
	.label--out { color: #7c3aed; }

	.stats-dl dd {
		margin: 0;
		display: flex;
		align-items: baseline;
		gap: 0.4rem;
	}

	.rate {
		font-size: 0.9375rem;
		font-weight: 600;
		font-variant-numeric: tabular-nums;
		color: #111827;
	}

	.total {
		font-size: 0.75rem;
		color: #9ca3af;
	}

	.offline-msg {
		margin: 0;
		font-size: 0.8125rem;
		color: #9ca3af;
	}
</style>
