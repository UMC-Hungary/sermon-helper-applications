<script lang="ts">
	import {
		obsConfig,
		obsState,
		vmixConfig,
		vmixState,
		atemConfig,
		atemState,
		broadlinkConfig,
		broadlinkState,
		youtubeConfig,
		youtubeState,
		facebookConfig,
		facebookState,
		discordConfig,
		discordState
	} from '$lib/stores/connectors.js';
	import { findConnector } from '$lib/connectors/registry.js';
	import ConnectorStatusBadge from './ConnectorStatusBadge.svelte';
	import type { BaseConfig, ConnectorState } from '$lib/connectors/types.js';

	interface Props {
		connectorId: string;
		compact?: boolean;
	}

	let { connectorId, compact = true }: Props = $props();

	const def = $derived(findConnector(connectorId));

	const config = $derived.by((): BaseConfig => {
		if (connectorId === 'obs') return $obsConfig;
		if (connectorId === 'vmix') return $vmixConfig;
		if (connectorId === 'atem') return $atemConfig;
		if (connectorId === 'broadlink') return $broadlinkConfig;
		if (connectorId === 'youtube') return $youtubeConfig;
		if (connectorId === 'facebook') return $facebookConfig;
		if (connectorId === 'discord') return $discordConfig;
		return { enabled: false };
	});

	const state = $derived.by((): ConnectorState => {
		if (connectorId === 'obs') return $obsState;
		if (connectorId === 'vmix') return $vmixState;
		if (connectorId === 'atem') return $atemState;
		if (connectorId === 'broadlink') return $broadlinkState;
		if (connectorId === 'youtube') return $youtubeState;
		if (connectorId === 'facebook') return $facebookState;
		if (connectorId === 'discord') return $discordState;
		return { connection: 'disconnected' };
	});

	const isConfigured = $derived(def ? def.isConfigured(config) : false);
	const isConnected = $derived(state.connection === 'connected');

	const hasFlags = $derived(
		isConnected && (
			!!def?.capabilities.streaming ||
			!!def?.capabilities.recording ||
			!!def?.capabilities.live
		)
	);
</script>

{#if def && isConfigured}
	<div class="widget" class:widget--compact={compact}>
		<ConnectorStatusBadge name={def.name} status={state.connection} />

		{#if hasFlags}
			<div class="flag-row">
				{#if def.capabilities.streaming}
					<span class="flag" class:flag--active={state.isStreaming}>
						{state.isStreaming ? 'Streaming' : 'Not Streaming'}
					</span>
				{/if}
				{#if def.capabilities.recording}
					<span class="flag" class:flag--active={state.isRecording}>
						{state.isRecording ? 'Recording' : 'Not Recording'}
					</span>
				{/if}
				{#if def.capabilities.live}
					<span class="flag" class:flag--active={state.isLive}>
						{state.isLive ? 'Live' : 'Not Live'}
					</span>
				{/if}
			</div>
		{/if}

		{#if !compact}
			<div class="widget-detail">
				<!-- Detail/actions layout — reserved for future iteration -->
			</div>
		{/if}
	</div>
{/if}

<style>
	.widget {
		padding: 0.875rem 1rem;
		border: 1px solid #e5e7eb;
		border-radius: 0.5rem;
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		align-items: flex-start;
	}

	.flag-row {
		display: flex;
		flex-wrap: wrap;
		gap: 0.375rem;
	}

	.flag {
		display: inline-flex;
		align-items: center;
		padding: 0.15rem 0.5rem;
		border-radius: 9999px;
		font-size: 0.75rem;
		font-weight: 500;
		background: #f3f4f6;
		color: #6b7280;
	}

	.flag--active {
		background: #d1fae5;
		color: #065f46;
	}
</style>
