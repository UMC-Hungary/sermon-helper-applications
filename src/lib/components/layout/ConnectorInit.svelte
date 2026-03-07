<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { goto } from '$app/navigation';
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import { appMode } from '$lib/stores/mode.js';
	import { serverUrl, serverPort, authToken, localNetworkUrl, appReady } from '$lib/stores/server-url.js';
	import { connectWs, disconnectWs } from '$lib/ws/client.js';
	import type { AppMode } from '$lib/stores/mode.js';
	import {
		obsStatus,
		obsConfig,
		obsState,
		vmixStatus,
		vmixConfig,
		vmixState,
		atemStatus,
		atemConfig,
		atemState,
		broadlinkStatus,
		broadlinkConfig,
		broadlinkState,
		youtubeStatus,
		youtubeConfig,
		youtubeState,
		facebookStatus,
		facebookConfig,
		facebookState,
		discordStatus,
		discordConfig,
		discordState,
		youtubeLiveActive,
		mapConnectorStatus
	} from '$lib/stores/connectors.js';
	import type {
		ObsConfig,
		VmixConfig,
		AtemConfig,
		BroadlinkConfig,
		YouTubeConfig,
		FacebookConfig,
		DiscordConfig,
		ConnectorStatus
	} from '$lib/stores/connectors.js';
	import { apiFetch } from '$lib/api/client.js';
	import { z } from 'zod';
	import { loadSavedLocale } from '$lib/i18n';
	import { findConnector } from '$lib/connectors/registry.js';
	import { pushError, clearErrors } from '$lib/stores/errors.js';

	let unlistenObs: (() => void) | undefined;
	let unlistenYt: (() => void) | undefined;
	let unlistenFb: (() => void) | undefined;

	function syncErrorStore(connectorId: string, status: ConnectorStatus) {
		const def = findConnector(connectorId);
		if (status === 'error') {
			const entry: Parameters<typeof pushError>[0] = {
				connectorId,
				connectorName: def?.name ?? connectorId,
				message: `${def?.name ?? connectorId} connection error`
			};
			if (def?.infoMarkdown !== undefined) {
				entry.infoMarkdown = def.infoMarkdown;
			}
			pushError(entry);
		} else {
			clearErrors(connectorId);
		}
	}

	$effect(() => { syncErrorStore('obs', $obsStatus); });
	$effect(() => { syncErrorStore('vmix', $vmixStatus); });
	$effect(() => { syncErrorStore('atem', $atemStatus); });
	$effect(() => { syncErrorStore('broadlink', $broadlinkStatus); });
	$effect(() => { syncErrorStore('youtube', $youtubeStatus); });
	$effect(() => { syncErrorStore('facebook', $facebookStatus); });
	$effect(() => { syncErrorStore('discord', $discordStatus); });

	$effect(() => { youtubeState.update((s) => ({ ...s, isLive: $youtubeLiveActive })); });

	onMount(async () => {
		try {
			const mode = await invoke<string | null>('get_app_mode');

			if (mode === null) {
				await goto('/setup');
				return;
			}

			appMode.set(mode as AppMode);

			if (mode === 'server') {
				const [token, port, localIp] = await Promise.all([
					invoke<string>('get_token'),
					invoke<number>('get_server_port'),
					invoke<string | null>('get_local_ip')
				]);
				authToken.set(token);
				serverPort.set(port);
				serverUrl.set(`http://localhost:${port}`);
				if (localIp) {
					localNetworkUrl.set(`http://${localIp}:${port}`);
				}
			} else if (mode === 'client') {
				const [url, token] = await Promise.all([
					invoke<string | null>('get_client_url'),
					invoke<string>('get_client_token')
				]);
				if (url) {
					serverUrl.set(url);
				}
				authToken.set(token);
			}
			appReady.set(true);
		} catch (e) {
			console.error('Layout init error:', e);
		}

		loadSavedLocale();

		const currentMode = await invoke<string | null>('get_app_mode').catch(() => null);

		if (currentMode === 'server') {
			try {
				const [cfg, status] = await Promise.all([
					invoke<ObsConfig>('get_obs_config'),
					invoke<{ type: string }>('get_obs_status')
				]);
				obsConfig.set(cfg);
				const mapped = mapConnectorStatus(status as Parameters<typeof mapConnectorStatus>[0]);
				obsStatus.set(mapped);
				obsState.update((s) => ({ ...s, connection: mapped }));
			} catch (e) {
				console.error('OBS connector init error:', e);
			}

			try {
				const [cfg, status] = await Promise.all([
					invoke<VmixConfig>('get_vmix_config'),
					invoke<{ type: string }>('get_vmix_status')
				]);
				vmixConfig.set(cfg);
				const mapped = mapConnectorStatus(status as Parameters<typeof mapConnectorStatus>[0]);
				vmixStatus.set(mapped);
				vmixState.update((s) => ({ ...s, connection: mapped }));
			} catch (e) {
				console.error('VMix connector init error:', e);
			}

			try {
				const [cfg, status] = await Promise.all([
					invoke<AtemConfig>('get_atem_config'),
					invoke<{ type: string }>('get_atem_status')
				]);
				atemConfig.set(cfg);
				const mapped = mapConnectorStatus(status as Parameters<typeof mapConnectorStatus>[0]);
				atemStatus.set(mapped);
				atemState.update((s) => ({ ...s, connection: mapped }));
			} catch (e) {
				console.error('ATEM connector init error:', e);
			}

			try {
				const [cfg, status] = await Promise.all([
					invoke<BroadlinkConfig>('get_broadlink_config'),
					invoke<{ type: string }>('get_broadlink_status')
				]);
				broadlinkConfig.set(cfg);
				const mapped = mapConnectorStatus(status as Parameters<typeof mapConnectorStatus>[0]);
				broadlinkStatus.set(mapped);
				broadlinkState.update((s) => ({ ...s, connection: mapped }));
			} catch (e) {
				console.error('BroadLink connector init error:', e);
			}

			try {
				const [cfg, status] = await Promise.all([
					invoke<YouTubeConfig>('get_youtube_config'),
					invoke<{ type: string }>('get_youtube_status')
				]);
				youtubeConfig.set(cfg);
				const mapped = mapConnectorStatus(status as Parameters<typeof mapConnectorStatus>[0]);
				youtubeStatus.set(mapped);
				youtubeState.update((s) => ({ ...s, connection: mapped }));
			} catch (e) {
				console.error('YouTube connector init error:', e);
			}

			try {
				const [cfg, status] = await Promise.all([
					invoke<FacebookConfig>('get_facebook_config'),
					invoke<{ type: string }>('get_facebook_status')
				]);
				facebookConfig.set(cfg);
				const mapped = mapConnectorStatus(status as Parameters<typeof mapConnectorStatus>[0]);
				facebookStatus.set(mapped);
				facebookState.update((s) => ({ ...s, connection: mapped }));
			} catch (e) {
				console.error('Facebook connector init error:', e);
			}

			try {
				const [cfg, status] = await Promise.all([
					invoke<DiscordConfig>('get_discord_config'),
					invoke<{ type: string }>('get_discord_status')
				]);
				discordConfig.set(cfg);
				const mapped = mapConnectorStatus(status as Parameters<typeof mapConnectorStatus>[0]);
				discordStatus.set(mapped);
				discordState.update((s) => ({ ...s, connection: mapped }));
			} catch (e) {
				console.error('Discord connector init error:', e);
			}

			unlistenObs = await listen<{ type: string }>('connector://obs-status', (event) => {
				const mapped = mapConnectorStatus(
					event.payload as Parameters<typeof mapConnectorStatus>[0]
				);
				obsStatus.set(mapped);
				obsState.update((s) => ({ ...s, connection: mapped }));
			});
			unlistenYt = await listen<{ type: string }>('connector://youtube-status', (event) => {
				const mapped = mapConnectorStatus(
					event.payload as Parameters<typeof mapConnectorStatus>[0]
				);
				youtubeStatus.set(mapped);
				youtubeState.update((s) => ({ ...s, connection: mapped }));
			});
			unlistenFb = await listen<{ type: string }>('connector://facebook-status', (event) => {
				const mapped = mapConnectorStatus(
					event.payload as Parameters<typeof mapConnectorStatus>[0]
				);
				facebookStatus.set(mapped);
				facebookState.update((s) => ({ ...s, connection: mapped }));
			});
		} else if (currentMode === 'client') {
			const ConnectorStatusSchema = z.object({
				type: z.enum(['disconnected', 'connecting', 'connected', 'error'] as const)
			});
			const ConnectorsResponseSchema = z.object({
				obs: ConnectorStatusSchema,
				vmix: ConnectorStatusSchema,
				youtube: ConnectorStatusSchema,
				facebook: ConnectorStatusSchema
			});

			try {
				const resp = await apiFetch('/api/connectors/status', ConnectorsResponseSchema);
				const mapAndSync = (
					statusStore: typeof obsStatus,
					stateStore: typeof obsState,
					raw: ConnectorStatus
				) => {
					statusStore.set(raw);
					stateStore.update((s) => ({ ...s, connection: raw }));
				};
				mapAndSync(obsStatus, obsState, resp.obs.type as ConnectorStatus);
				mapAndSync(vmixStatus, vmixState, resp.vmix.type as ConnectorStatus);
				mapAndSync(youtubeStatus, youtubeState, resp.youtube.type as ConnectorStatus);
				mapAndSync(facebookStatus, facebookState, resp.facebook.type as ConnectorStatus);
			} catch (e) {
				console.error('Connector status fetch error:', e);
			}
		}

		connectWs();
	});

	onDestroy(() => {
		unlistenObs?.();
		unlistenYt?.();
		unlistenFb?.();
		disconnectWs();
	});
</script>
