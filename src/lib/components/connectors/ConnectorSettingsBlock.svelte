<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { openUrl } from '@tauri-apps/plugin-opener';
	import { _ } from 'svelte-i18n';
	import { streamPreviewEnabled } from '$lib/stores/stream-preview.js';
	import {
		obsConfig,
		obsStatus,
		obsState,
		vmixConfig,
		vmixStatus,
		atemConfig,
		atemStatus,
		broadlinkConfig,
		broadlinkStatus,
		youtubeConfig,
		youtubeStatus,
		facebookConfig,
		facebookStatus,
		discordConfig,
		discordStatus
	} from '$lib/stores/connectors.js';
	import { get } from 'svelte/store';
	import type {
		ObsConfig,
		VmixConfig,
		AtemConfig,
		BroadlinkConfig,
		YouTubeConfig,
		FacebookConfig,
		DiscordConfig
	} from '$lib/stores/connectors.js';
	import { findConnector } from '$lib/connectors/registry.js';
	import ConnectorStatusBadge from './ConnectorStatusBadge.svelte';
	import { youtubeLogout, facebookLogout, fetchYouTubeStreamKey, fetchFacebookStreamKey } from '$lib/api/connectors.js';
	import BroadlinkDiscoveryPanel from './broadlink/DiscoveryPanel.svelte';

	interface Props {
		connectorId: string;
		onSaveSuccess?: () => void;
	}

	let { connectorId, onSaveSuccess }: Props = $props();

	const def = $derived(findConnector(connectorId));

	// ── OBS ────────────────────────────────────────────────────────────────────
	let obsForm: ObsConfig = $state({ enabled: false, host: 'localhost', port: 4455, password: null });
	let obsSaving = $state(false);
	let obsError = $state('');

	// ── Stream Preview check ──────────────────────────────────────────────────
	let previewCheckError = $state<string | null>(null);

	// ── OBS Streaming Destination ─────────────────────────────────────────────
	type ObsDestination = 'proxy' | 'youtube' | 'facebook';
	interface ObsStreamSettings {
		serviceType: string;
		server: string;
		key: string;
	}
	let obsDestination = $state<ObsDestination>('proxy');
	let obsStreamSettings = $state<ObsStreamSettings | null>(null);
	let obsDestSaving = $state(false);
	let obsDestError = $state('');
	let obsDestYtUrl = $state('');
	let obsDestFbUrl = $state('');
	let obsDestFetchingYt = $state(false);
	let obsDestFetchingFb = $state(false);

	// ── Local Streaming ────────────────────────────────────────────────────────
	let localIp = $state('…');
	let rtmpUrlCopied = $state(false);

	// ── Multi-Stream Relay ────────────────────────────────────────────────────
	interface RelayConfig {
		youtubeRtmpUrl: string;
		facebookRtmpUrl: string;
		rtmpRestreamEnabled: boolean;
	}

	let relayForm: RelayConfig = $state({ youtubeRtmpUrl: '', facebookRtmpUrl: '', rtmpRestreamEnabled: true });
	let relaySaving = $state(false);
	let relayError = $state('');
	let relayFetchingYt = $state(false);
	let relayFetchingFb = $state(false);

	$effect(() => {
		if (connectorId === 'obs') {
			invoke<RelayConfig>('get_relay_config')
				.then((cfg) => {
					relayForm = cfg;
				})
				.catch(() => {});
			invoke<string | null>('get_local_ip')
				.then((ip) => {
					if (ip) localIp = ip;
				})
				.catch(() => {});
		}
	});

	$effect(() => {
		if (connectorId !== 'obs') return;
		if ($obsStatus === 'connected') {
			invoke<ObsStreamSettings>('get_obs_stream_settings')
				.then((s) => {
					obsStreamSettings = s;
					const srv = s.server.toLowerCase();
					if (srv.includes('localhost') || srv.includes('127.0.0.1')) {
						obsDestination = 'proxy';
					} else if (srv.includes('youtube')) {
						obsDestination = 'youtube';
					} else if (srv.includes('facebook') || srv.includes('fbcdn')) {
						obsDestination = 'facebook';
					}
				})
				.catch(() => {
					obsStreamSettings = null;
				});
		} else {
			obsStreamSettings = null;
		}
	});

	async function saveRelayConfig() {
		relaySaving = true;
		relayError = '';
		try {
			await invoke('save_relay_config', { config: relayForm });
		} catch (e) {
			relayError = String(e);
		} finally {
			relaySaving = false;
		}
	}

	async function toggleRtmpRestream() {
		relayError = '';
		try {
			await invoke('save_relay_config', { config: relayForm });
		} catch (e) {
			relayError = String(e);
		}
	}

	async function fetchRelayYouTubeKey() {
		relayFetchingYt = true;
		relayError = '';
		try {
			const result = await fetchYouTubeStreamKey();
			relayForm = { ...relayForm, youtubeRtmpUrl: result.rtmpUrl };
		} catch (e) {
			relayError = String(e);
		} finally {
			relayFetchingYt = false;
		}
	}

	async function fetchRelayFacebookKey() {
		relayFetchingFb = true;
		relayError = '';
		try {
			const result = await fetchFacebookStreamKey();
			relayForm = { ...relayForm, facebookRtmpUrl: result.rtmpUrl };
		} catch (e) {
			relayError = String(e);
		} finally {
			relayFetchingFb = false;
		}
	}

	async function fetchDestYouTubeKey() {
		obsDestFetchingYt = true;
		obsDestError = '';
		try {
			const result = await fetchYouTubeStreamKey();
			obsDestYtUrl = result.rtmpUrl;
		} catch (e) {
			obsDestError = String(e);
		} finally {
			obsDestFetchingYt = false;
		}
	}

	async function fetchDestFacebookKey() {
		obsDestFetchingFb = true;
		obsDestError = '';
		try {
			const result = await fetchFacebookStreamKey();
			obsDestFbUrl = result.rtmpUrl;
		} catch (e) {
			obsDestError = String(e);
		} finally {
			obsDestFetchingFb = false;
		}
	}

	async function applyObsDestination() {
		obsDestSaving = true;
		obsDestError = '';
		try {
			let server = '';
			let key = '';
			if (obsDestination === 'proxy') {
				server = 'rtmp://127.0.0.1:1935/live';
				key = '';
			} else if (obsDestination === 'youtube') {
				const lastSlash = obsDestYtUrl.lastIndexOf('/');
				server = lastSlash > 6 ? obsDestYtUrl.slice(0, lastSlash) : obsDestYtUrl;
				key = lastSlash > 6 ? obsDestYtUrl.slice(lastSlash + 1) : '';
			} else if (obsDestination === 'facebook') {
				const lastSlash = obsDestFbUrl.lastIndexOf('/');
				server = lastSlash > 6 ? obsDestFbUrl.slice(0, lastSlash) : obsDestFbUrl;
				key = lastSlash > 6 ? obsDestFbUrl.slice(lastSlash + 1) : '';
			}
			await invoke('set_obs_stream_settings', { server, key });
			// Refresh displayed settings
			const updated = await invoke<ObsStreamSettings>('get_obs_stream_settings');
			obsStreamSettings = updated;
		} catch (e) {
			obsDestError = String(e);
		} finally {
			obsDestSaving = false;
		}
	}

	function copyRtmpUrl() {
		const url = `rtmp://${localIp}:1935/live`;
		navigator.clipboard.writeText(url).then(() => {
			rtmpUrlCopied = true;
			setTimeout(() => { rtmpUrlCopied = false; }, 2000);
		});
	}

	function enablePreview() {
		previewCheckError = null;
		const status = get(obsStatus);
		const state = get(obsState);
		if (status !== 'connected') {
			previewCheckError =
				'OBS is not connected. Check your OBS WebSocket settings above and make sure OBS is running.';
			return;
		}
		if (!state.isStreaming) {
			previewCheckError =
				'OBS is connected but not streaming. Click "Start Streaming" in OBS, then click Recheck.';
			return;
		}
		streamPreviewEnabled.set(true);
	}

	function disablePreview() {
		streamPreviewEnabled.set(false);
		previewCheckError = null;
	}

	$effect(() => {
		if (connectorId === 'obs') obsForm = { ...$obsConfig };
	});

	async function saveObs() {
		obsSaving = true;
		obsError = '';
		try {
			await invoke('save_obs_config', { config: obsForm });
			obsConfig.set({ ...obsForm });
			onSaveSuccess?.();
		} catch (e) {
			obsError = String(e);
		} finally {
			obsSaving = false;
		}
	}

	async function connectObs() {
		obsError = '';
		try {
			await invoke('connect_obs');
		} catch (e) {
			obsError = String(e);
		}
	}

	async function disconnectObs() {
		obsError = '';
		try {
			await invoke('disconnect_obs');
		} catch (e) {
			obsError = String(e);
		}
	}

	// ── VMix ───────────────────────────────────────────────────────────────────
	let vmixForm: VmixConfig = $state({ enabled: false, host: 'localhost', port: 8088 });
	let vmixSaving = $state(false);
	let vmixError = $state('');

	$effect(() => {
		if (connectorId === 'vmix') vmixForm = { ...$vmixConfig };
	});

	async function saveVmix() {
		vmixSaving = true;
		vmixError = '';
		try {
			await invoke('save_vmix_config', { config: vmixForm });
			vmixConfig.set({ ...vmixForm });
			onSaveSuccess?.();
		} catch (e) {
			vmixError = String(e);
		} finally {
			vmixSaving = false;
		}
	}

	// ── ATEM ───────────────────────────────────────────────────────────────────
	let atemForm: AtemConfig = $state({ enabled: false, host: '', port: 9910 });
	let atemSaving = $state(false);
	let atemError = $state('');

	$effect(() => {
		if (connectorId === 'atem') atemForm = { ...$atemConfig };
	});

	async function saveAtem() {
		atemSaving = true;
		atemError = '';
		try {
			await invoke('save_atem_config', { config: atemForm });
			atemConfig.set({ ...atemForm });
			onSaveSuccess?.();
		} catch (e) {
			atemError = String(e);
		} finally {
			atemSaving = false;
		}
	}

	// ── YouTube ────────────────────────────────────────────────────────────────
	let ytForm: YouTubeConfig = $state({ enabled: false, clientId: '', clientSecret: '' });
	let ytSaving = $state(false);
	let ytError = $state('');
	let ytLoggingIn = $state(false);

	$effect(() => {
		if (connectorId === 'youtube') ytForm = { ...$youtubeConfig };
	});

	async function saveYt() {
		ytSaving = true;
		ytError = '';
		try {
			await invoke('save_youtube_config', { config: ytForm });
			youtubeConfig.set({ ...ytForm });
			onSaveSuccess?.();
		} catch (e) {
			ytError = String(e);
		} finally {
			ytSaving = false;
		}
	}

	async function loginYt() {
		ytLoggingIn = true;
		ytError = '';
		try {
			const url = await invoke<string>('get_youtube_auth_url');
			await openUrl(url);
		} catch (e) {
			ytError = String(e);
		} finally {
			ytLoggingIn = false;
		}
	}

	async function logoutYt() {
		ytError = '';
		try {
			await youtubeLogout();
			await invoke('youtube_logout');
		} catch (e) {
			ytError = String(e);
		}
	}

	// ── Facebook ───────────────────────────────────────────────────────────────
	let fbForm: FacebookConfig = $state({ enabled: false, appId: '', appSecret: '', pageId: '' });
	let fbSaving = $state(false);
	let fbError = $state('');
	let fbLoggingIn = $state(false);

	$effect(() => {
		if (connectorId === 'facebook') fbForm = { ...$facebookConfig };
	});

	async function saveFb() {
		fbSaving = true;
		fbError = '';
		try {
			await invoke('save_facebook_config', { config: fbForm });
			facebookConfig.set({ ...fbForm });
			onSaveSuccess?.();
		} catch (e) {
			fbError = String(e);
		} finally {
			fbSaving = false;
		}
	}

	async function loginFb() {
		fbLoggingIn = true;
		fbError = '';
		try {
			const url = await invoke<string>('get_facebook_auth_url');
			await openUrl(url);
		} catch (e) {
			fbError = String(e);
		} finally {
			fbLoggingIn = false;
		}
	}

	async function logoutFb() {
		fbError = '';
		try {
			await facebookLogout();
			await invoke('facebook_logout');
		} catch (e) {
			fbError = String(e);
		}
	}

	// ── Discord ────────────────────────────────────────────────────────────────
	let discordForm: DiscordConfig = $state({ enabled: false, webhookUrl: '' });
	let discordSaving = $state(false);
	let discordError = $state('');

	$effect(() => {
		if (connectorId === 'discord') discordForm = { ...$discordConfig };
	});

	async function saveDiscord() {
		discordSaving = true;
		discordError = '';
		try {
			await invoke('save_discord_config', { config: discordForm });
			discordConfig.set({ ...discordForm });
			onSaveSuccess?.();
		} catch (e) {
			discordError = String(e);
		} finally {
			discordSaving = false;
		}
	}

	// ── Broadlink ──────────────────────────────────────────────────────────────
	let broadlinkForm: BroadlinkConfig = $state({ enabled: false });
	let broadlinkSaving = $state(false);
	let broadlinkError = $state('');

	$effect(() => {
		if (connectorId === 'broadlink') broadlinkForm = { ...$broadlinkConfig };
	});

	async function saveBroadlink() {
		broadlinkSaving = true;
		broadlinkError = '';
		try {
			await invoke('save_broadlink_config', { config: broadlinkForm });
			broadlinkConfig.set({ ...broadlinkForm });
			onSaveSuccess?.();
		} catch (e) {
			broadlinkError = String(e);
		} finally {
			broadlinkSaving = false;
		}
	}
</script>

{#if def}
	<div class="settings-block">
		<!-- Header -->
		<div class="connector-header">
			<div>
				<h3>{def.name}</h3>
				{#if connectorId === 'obs'}
					<p class="note">{$_('appSettings.connectors.obs.subtitle')}</p>
				{:else if connectorId === 'vmix'}
					<p class="note">{$_('appSettings.connectors.vmix.subtitle')}</p>
				{:else if connectorId === 'atem'}
					<p class="note">{$_('appSettings.connectors.atem.subtitle')}</p>
				{:else if connectorId === 'broadlink'}
					<p class="note">{$_('appSettings.connectors.broadlink.subtitle')}</p>
				{:else if connectorId === 'youtube'}
					<p class="note">{$_('appSettings.connectors.youtube.subtitle')}</p>
				{:else if connectorId === 'facebook'}
					<p class="note">{$_('appSettings.connectors.facebook.subtitle')}</p>
				{:else if connectorId === 'discord'}
					<p class="note">{$_('appSettings.connectors.discord.subtitle')}</p>
				{/if}
			</div>
			{#if connectorId === 'obs'}
				<ConnectorStatusBadge name="OBS" status={$obsStatus} />
			{:else if connectorId === 'vmix'}
				<ConnectorStatusBadge name="VMix" status={$vmixStatus} />
			{:else if connectorId === 'atem'}
				<ConnectorStatusBadge name="ATEM" status={$atemStatus} />
			{:else if connectorId === 'broadlink'}
				<ConnectorStatusBadge name="BroadLink" status={$broadlinkStatus} />
			{:else if connectorId === 'youtube'}
				<ConnectorStatusBadge name="YouTube" status={$youtubeStatus} />
			{:else if connectorId === 'facebook'}
				<ConnectorStatusBadge name="Facebook" status={$facebookStatus} />
			{:else if connectorId === 'discord'}
				<ConnectorStatusBadge name="Discord" status={$discordStatus} />
			{/if}
		</div>

		<!-- ── OBS form ──────────────────────────────────────────────────────── -->
		{#if connectorId === 'obs'}
			<div class="form-row">
				<label class="checkbox-label">
					<input type="checkbox" bind:checked={obsForm.enabled} />
					{$_('appSettings.connectors.obs.enabled')}
				</label>
			</div>

			<div class="form-grid">
				<div class="field">
					<label for="obs-host">{$_('appSettings.connectors.obs.host')}</label>
					<input id="obs-host" type="text" bind:value={obsForm.host} />
				</div>
				<div class="field">
					<label for="obs-port">{$_('appSettings.connectors.obs.port')}</label>
					<input id="obs-port" type="number" min="1" max="65535" bind:value={obsForm.port} />
				</div>
				<div class="field field--full">
					<label for="obs-password">{$_('appSettings.connectors.obs.password')}</label>
					<input
						id="obs-password"
						type="password"
						placeholder={$_('appSettings.connectors.obs.passwordPlaceholder')}
						value={obsForm.password ?? ''}
						oninput={(e) => {
							const val = (e.currentTarget as HTMLInputElement).value;
							obsForm.password = val.length > 0 ? val : null;
						}}
					/>
				</div>
			</div>

			{#if obsError}
				<p class="error" role="alert">{obsError}</p>
			{/if}

			<div class="button-row">
				<button class="btn-primary" onclick={saveObs} disabled={obsSaving}>
					{obsSaving
						? $_('appSettings.connectors.obs.saving')
						: $_('appSettings.connectors.obs.save')}
				</button>
				{#if $obsStatus === 'disconnected' || $obsStatus === 'error'}
					<button class="btn-secondary" onclick={connectObs}>
						{$_('appSettings.connectors.obs.connect')}
					</button>
				{:else if $obsStatus === 'connected' || $obsStatus === 'connecting'}
					<button class="btn-danger" onclick={disconnectObs}>
						{$_('appSettings.connectors.obs.disconnect')}
					</button>
				{/if}
			</div>

			<!-- ── Stream Preview ────────────────────────────────────────────── -->
			<!-- OBS Streaming Destination -->
			<div class="preview-section">
				<h4 class="preview-heading">OBS Streaming Destination</h4>

				{#if obsStreamSettings}
					<p class="rtmp-label">
						Current server: <code class="inline-code">{obsStreamSettings.server || '(none)'}</code>
					</p>
				{:else}
					<p class="rtmp-label">Connect OBS to view and change stream destination.</p>
				{/if}

				<fieldset class="dest-cards" disabled={$obsStatus !== 'connected'}>
					<legend class="sr-only">Streaming destination</legend>

					<label class="dest-card" class:dest-card--active={obsDestination === 'proxy'}>
						<input type="radio" name="obs-dest" value="proxy" bind:group={obsDestination} />
						<span class="dest-card-title">Sermon Helper</span>
						<span class="dest-card-desc">Stream via this app's proxy (<code>rtmp://127.0.0.1:1935/live</code>)</span>
					</label>

					<label class="dest-card" class:dest-card--active={obsDestination === 'youtube'}>
						<input type="radio" name="obs-dest" value="youtube" bind:group={obsDestination} />
						<span class="dest-card-title">YouTube</span>
						<span class="dest-card-desc">Stream directly to YouTube Live</span>
						{#if obsDestination === 'youtube'}
							<div class="dest-url-row">
								<input
									type="text"
									class="dest-url-input"
									placeholder="rtmp://a.rtmp.youtube.com/live2/…"
									bind:value={obsDestYtUrl}
									aria-label="YouTube RTMP URL"
								/>
								<button
									class="btn-fetch"
									onclick={fetchDestYouTubeKey}
									disabled={obsDestFetchingYt || $youtubeStatus !== 'connected'}
									title={$youtubeStatus !== 'connected' ? 'Connect YouTube first' : 'Fetch stream key from YouTube API'}
								>
									{obsDestFetchingYt ? 'Fetching…' : 'Fetch'}
								</button>
							</div>
						{/if}
					</label>

					<label class="dest-card" class:dest-card--active={obsDestination === 'facebook'}>
						<input type="radio" name="obs-dest" value="facebook" bind:group={obsDestination} />
						<span class="dest-card-title">Facebook</span>
						<span class="dest-card-desc">Stream directly to Facebook Live</span>
						{#if obsDestination === 'facebook'}
							<div class="dest-url-row">
								<input
									type="text"
									class="dest-url-input"
									placeholder="rtmps://live-api-s.facebook.com:443/rtmp/…"
									bind:value={obsDestFbUrl}
									aria-label="Facebook RTMP URL"
								/>
								<button
									class="btn-fetch"
									onclick={fetchDestFacebookKey}
									disabled={obsDestFetchingFb || $facebookStatus !== 'connected'}
									title={$facebookStatus !== 'connected' ? 'Connect Facebook first' : 'Fetch stream key from Facebook API'}
								>
									{obsDestFetchingFb ? 'Fetching…' : 'Fetch'}
								</button>
							</div>
						{/if}
					</label>
				</fieldset>

				{#if obsDestError}
					<p class="error" role="alert">{obsDestError}</p>
				{/if}

				<button
					class="btn-primary"
					onclick={applyObsDestination}
					disabled={obsDestSaving || $obsStatus !== 'connected'}
				>
					{obsDestSaving ? 'Applying…' : 'Apply to OBS'}
				</button>
			</div>

			<!-- Local Streaming -->
			<div class="preview-section">
				<h4 class="preview-heading">Local Streaming</h4>

				<div class="local-streaming-row">
					<div class="local-streaming-label">
						<span class="toggle-label-main">RTMP Re-stream</span>
						<span class="toggle-label-sub">When on, other computers on your network can receive the stream</span>
					</div>
					<label class="toggle-switch" aria-label="Enable RTMP re-stream">
						<input
							type="checkbox"
							bind:checked={relayForm.rtmpRestreamEnabled}
							onchange={toggleRtmpRestream}
						/>
						<span class="toggle-track" aria-hidden="true"></span>
					</label>
				</div>

				{#if relayForm.rtmpRestreamEnabled}
					<div class="lan-url-row">
						<code class="lan-url">rtmp://{localIp}:1935/live</code>
						<button class="btn-copy" onclick={copyRtmpUrl} aria-label="Copy RTMP URL">
							{#if rtmpUrlCopied}
								<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" aria-hidden="true"><path d="M20 6L9 17l-5-5"/></svg>
								Copied
							{:else}
								<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" aria-hidden="true"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1"/></svg>
								Copy
							{/if}
						</button>
					</div>
				{/if}

				{#if relayError}
					<p class="error" role="alert">{relayError}</p>
				{/if}

				<div class="local-streaming-row">
					<div class="local-streaming-label">
						<span class="toggle-label-main">HLS Preview</span>
						<span class="toggle-label-sub">Floating video player in this app</span>
					</div>
					{#if $streamPreviewEnabled}
						<button class="preview-btn preview-btn--danger" onclick={disablePreview}>
							Disable
						</button>
					{:else}
						<button class="preview-btn" onclick={enablePreview}>
							Enable
						</button>
					{/if}
				</div>

				{#if $streamPreviewEnabled}
					<div class="check-result check-ok">
						<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" aria-hidden="true"><path d="M20 6L9 17l-5-5"/></svg>
						HLS preview is active
					</div>
				{:else if previewCheckError}
					<div class="check-result check-error">
						<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" aria-hidden="true"><circle cx="12" cy="12" r="10"/><path d="M12 8v4m0 4h.01"/></svg>
						{previewCheckError}
					</div>
				{/if}
			</div>

			<!-- ── Multi-Stream Relay ────────────────────────────────────────── -->
			<div class="relay-section">
				<h4 class="relay-heading">Multi-Stream Relay</h4>
				<p class="relay-note">
					mediamtx forwards the OBS stream to YouTube and/or Facebook simultaneously.
					Leave a field empty to skip that destination. Requires <code>ffmpeg</code> on PATH.
				</p>

				<div class="relay-field-group">
					<div class="field">
						<label for="relay-yt-url">YouTube RTMP URL</label>
						<div class="field-with-btn">
							<input
								id="relay-yt-url"
								type="text"
								bind:value={relayForm.youtubeRtmpUrl}
								placeholder="rtmp://a.rtmp.youtube.com/live2/…"
							/>
							<button
								class="btn-fetch"
								onclick={fetchRelayYouTubeKey}
								disabled={relayFetchingYt || $youtubeStatus !== 'connected'}
								title={$youtubeStatus !== 'connected' ? 'Connect YouTube first' : 'Fetch stream key from YouTube API'}
							>
								{relayFetchingYt ? 'Fetching…' : 'Fetch'}
							</button>
						</div>
					</div>

					<div class="field">
						<label for="relay-fb-url">Facebook RTMP URL</label>
						<div class="field-with-btn">
							<input
								id="relay-fb-url"
								type="text"
								bind:value={relayForm.facebookRtmpUrl}
								placeholder="rtmps://live-api-s.facebook.com:443/rtmp/…"
							/>
							<button
								class="btn-fetch"
								onclick={fetchRelayFacebookKey}
								disabled={relayFetchingFb || $facebookStatus !== 'connected'}
								title={$facebookStatus !== 'connected' ? 'Connect Facebook first' : 'Fetch stream key from Facebook API'}
							>
								{relayFetchingFb ? 'Fetching…' : 'Fetch'}
							</button>
						</div>
					</div>
				</div>

				{#if relayError}
					<p class="error" role="alert">{relayError}</p>
				{/if}

				<button class="btn-primary" onclick={saveRelayConfig} disabled={relaySaving}>
					{relaySaving ? 'Saving…' : 'Save & Apply'}
				</button>
			</div>

		<!-- ── VMix form ─────────────────────────────────────────────────────── -->
		{:else if connectorId === 'vmix'}
			<p class="coming-soon-notice">{$_('appSettings.connectors.vmix.comingSoon')}</p>

			<fieldset disabled>
				<div class="form-row">
					<label class="checkbox-label">
						<input type="checkbox" bind:checked={vmixForm.enabled} />
						{$_('appSettings.connectors.vmix.enabled')}
					</label>
				</div>
				<div class="form-grid">
					<div class="field">
						<label for="vmix-host">{$_('appSettings.connectors.vmix.host')}</label>
						<input id="vmix-host" type="text" bind:value={vmixForm.host} />
					</div>
					<div class="field">
						<label for="vmix-port">{$_('appSettings.connectors.vmix.port')}</label>
						<input id="vmix-port" type="number" min="1" max="65535" bind:value={vmixForm.port} />
					</div>
				</div>
			</fieldset>

			{#if vmixError}
				<p class="error" role="alert">{vmixError}</p>
			{/if}

			<div class="button-row">
				<button class="btn-primary" onclick={saveVmix} disabled={vmixSaving}>
					{vmixSaving
						? $_('appSettings.connectors.vmix.saving')
						: $_('appSettings.connectors.vmix.save')}
				</button>
			</div>

		<!-- ── ATEM form ─────────────────────────────────────────────────────── -->
		{:else if connectorId === 'atem'}
			<p class="coming-soon-notice">{$_('appSettings.connectors.atem.comingSoon')}</p>

			<fieldset disabled>
				<div class="form-row">
					<label class="checkbox-label">
						<input type="checkbox" bind:checked={atemForm.enabled} />
						{$_('appSettings.connectors.atem.enabled')}
					</label>
				</div>
				<div class="form-grid">
					<div class="field">
						<label for="atem-host">{$_('appSettings.connectors.atem.host')}</label>
						<input id="atem-host" type="text" bind:value={atemForm.host} />
					</div>
					<div class="field">
						<label for="atem-port">{$_('appSettings.connectors.atem.port')}</label>
						<input id="atem-port" type="number" min="1" max="65535" bind:value={atemForm.port} />
					</div>
				</div>
			</fieldset>

			{#if atemError}
				<p class="error" role="alert">{atemError}</p>
			{/if}

			<div class="button-row">
				<button class="btn-primary" onclick={saveAtem} disabled={atemSaving}>
					{atemSaving
						? $_('appSettings.connectors.atem.saving')
						: $_('appSettings.connectors.atem.save')}
				</button>
			</div>

		<!-- ── YouTube form ───────────────────────────────────────────────────── -->
		{:else if connectorId === 'youtube'}
			<div class="form-row">
				<label class="checkbox-label">
					<input type="checkbox" bind:checked={ytForm.enabled} />
					{$_('appSettings.connectors.youtube.enabled')}
				</label>
			</div>

			<div class="form-grid">
				<div class="field">
					<label for="yt-client-id">{$_('appSettings.connectors.youtube.clientId')}</label>
					<input id="yt-client-id" type="text" bind:value={ytForm.clientId} />
				</div>
				<div class="field">
					<label for="yt-client-secret">{$_('appSettings.connectors.youtube.clientSecret')}</label>
					<input id="yt-client-secret" type="password" bind:value={ytForm.clientSecret} />
				</div>
			</div>

			{#if ytError}
				<p class="error" role="alert">{ytError}</p>
			{/if}

			<div class="button-row">
				<button class="btn-primary" onclick={saveYt} disabled={ytSaving}>
					{ytSaving
						? $_('appSettings.connectors.youtube.saving')
						: $_('appSettings.connectors.youtube.save')}
				</button>
				{#if $youtubeStatus === 'disconnected' || $youtubeStatus === 'error'}
					<button class="btn-secondary" onclick={loginYt} disabled={ytLoggingIn}>
						{ytLoggingIn
							? $_('appSettings.connectors.youtube.loggingIn')
							: $_('appSettings.connectors.youtube.login')}
					</button>
				{:else if $youtubeStatus === 'connected'}
					<button class="btn-danger" onclick={logoutYt}>
						{$_('appSettings.connectors.youtube.logout')}
					</button>
				{/if}
			</div>

		<!-- ── Facebook form ──────────────────────────────────────────────────── -->
		{:else if connectorId === 'facebook'}
			<div class="form-row">
				<label class="checkbox-label">
					<input type="checkbox" bind:checked={fbForm.enabled} />
					{$_('appSettings.connectors.facebook.enabled')}
				</label>
			</div>

			<div class="form-grid">
				<div class="field">
					<label for="fb-app-id">{$_('appSettings.connectors.facebook.appId')}</label>
					<input id="fb-app-id" type="text" bind:value={fbForm.appId} />
				</div>
				<div class="field">
					<label for="fb-app-secret">{$_('appSettings.connectors.facebook.appSecret')}</label>
					<input id="fb-app-secret" type="password" bind:value={fbForm.appSecret} />
				</div>
				<div class="field field--full">
					<label for="fb-page-id">{$_('appSettings.connectors.facebook.pageId')}</label>
					<input id="fb-page-id" type="text" bind:value={fbForm.pageId} />
				</div>
			</div>

			{#if fbError}
				<p class="error" role="alert">{fbError}</p>
			{/if}

			<div class="button-row">
				<button class="btn-primary" onclick={saveFb} disabled={fbSaving}>
					{fbSaving
						? $_('appSettings.connectors.facebook.saving')
						: $_('appSettings.connectors.facebook.save')}
				</button>
				{#if $facebookStatus === 'disconnected' || $facebookStatus === 'error'}
					<button class="btn-secondary" onclick={loginFb} disabled={fbLoggingIn}>
						{fbLoggingIn
							? $_('appSettings.connectors.facebook.loggingIn')
							: $_('appSettings.connectors.facebook.login')}
					</button>
				{:else if $facebookStatus === 'connected'}
					<button class="btn-danger" onclick={logoutFb}>
						{$_('appSettings.connectors.facebook.logout')}
					</button>
				{/if}
			</div>

		<!-- ── BroadLink form ────────────────────────────────────────────────── -->
		{:else if connectorId === 'broadlink'}
			<div class="form-row">
				<label class="checkbox-label">
					<input type="checkbox" bind:checked={broadlinkForm.enabled} />
					{$_('appSettings.connectors.broadlink.enabled')}
				</label>
			</div>

			{#if broadlinkError}
				<p class="error" role="alert">{broadlinkError}</p>
			{/if}

			<div class="button-row">
				<button class="btn-primary" onclick={saveBroadlink} disabled={broadlinkSaving}>
					{broadlinkSaving
						? $_('appSettings.connectors.broadlink.saving')
						: $_('appSettings.connectors.broadlink.save')}
				</button>
			</div>

			<BroadlinkDiscoveryPanel />

		<!-- ── Discord form ───────────────────────────────────────────────────── -->
		{:else if connectorId === 'discord'}
			<p class="coming-soon-notice">{$_('appSettings.connectors.discord.comingSoon')}</p>

			<fieldset disabled>
				<div class="form-row">
					<label class="checkbox-label">
						<input type="checkbox" bind:checked={discordForm.enabled} />
						{$_('appSettings.connectors.discord.enabled')}
					</label>
				</div>
				<div class="form-grid">
					<div class="field field--full">
						<label for="discord-webhook">{$_('appSettings.connectors.discord.webhookUrl')}</label>
						<input id="discord-webhook" type="text" bind:value={discordForm.webhookUrl} />
					</div>
				</div>
			</fieldset>

			{#if discordError}
				<p class="error" role="alert">{discordError}</p>
			{/if}

			<div class="button-row">
				<button class="btn-primary" onclick={saveDiscord} disabled={discordSaving}>
					{discordSaving
						? $_('appSettings.connectors.discord.saving')
						: $_('appSettings.connectors.discord.save')}
				</button>
			</div>

		{/if}
	</div>
{/if}

<style>
	.settings-block {
		padding: 1.25rem;
		border: 1px solid #e5e7eb;
		border-radius: 0.5rem;
		margin-bottom: 1rem;
	}

	/* ── Stream Preview section ──────────────────────────────────────────────── */
	.preview-section {
		margin-top: 1.25rem;
		padding-top: 1.25rem;
		border-top: 1px solid #e5e7eb;
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.preview-heading {
		margin: 0;
		font-size: 0.8125rem;
		font-weight: 600;
		color: #374151;
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}

	.check-result {
		display: flex;
		align-items: flex-start;
		gap: 0.5rem;
		padding: 0.5rem 0.625rem;
		border-radius: 0.375rem;
		font-size: 0.8125rem;
		line-height: 1.4;
	}

	.check-ok {
		background: #d1fae5;
		color: #065f46;
	}

	.check-error {
		background: #fee2e2;
		color: #991b1b;
	}

	.check-result svg {
		flex-shrink: 0;
		margin-top: 0.1rem;
	}

	.preview-btn {
		align-self: flex-start;
		padding: 0.375rem 0.875rem;
		background: #0f0f0f;
		color: #fff;
		border: none;
		border-radius: 9999px;
		font-size: 0.8125rem;
		font-weight: 600;
		cursor: pointer;
		transition: background 0.15s;
	}

	.preview-btn:hover {
		background: #272727;
	}

	.preview-btn--danger {
		background: #fee2e2;
		color: #991b1b;
	}

	.preview-btn--danger:hover {
		background: #fecaca;
	}

	.rtmp-label {
		margin: 0;
		font-size: 0.75rem;
		color: #6b7280;
	}

	/* ── Multi-Stream Relay section ─────────────────────────────────────────── */
	.relay-section {
		margin-top: 1.25rem;
		padding-top: 1.25rem;
		border-top: 1px solid #e5e7eb;
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.relay-heading {
		margin: 0;
		font-size: 0.8125rem;
		font-weight: 600;
		color: #374151;
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}

	.relay-note {
		margin: 0;
		font-size: 0.75rem;
		color: #6b7280;
	}

	.relay-note code {
		font-size: 0.75rem;
		background: #e5e7eb;
		padding: 0.1rem 0.3rem;
		border-radius: 0.25rem;
	}

	.relay-field-group {
		display: flex;
		flex-direction: column;
		gap: 0.625rem;
	}

	.field-with-btn {
		display: flex;
		gap: 0.5rem;
		align-items: center;
	}

	.field-with-btn input {
		flex: 1;
		min-width: 0;
	}

	.btn-fetch {
		padding: 0.375rem 0.75rem;
		background: #f3f4f6;
		color: #374151;
		border: 1px solid #d1d5db;
		border-radius: 0.375rem;
		font-size: 0.8125rem;
		cursor: pointer;
		white-space: nowrap;
		flex-shrink: 0;
	}

	.btn-fetch:hover:not(:disabled) {
		background: #e5e7eb;
	}

	.btn-fetch:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	/* ── OBS Destination cards ──────────────────────────────────────────────── */
	.dest-cards {
		border: none;
		padding: 0;
		margin: 0;
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.dest-cards:disabled .dest-card {
		opacity: 0.5;
		cursor: not-allowed;
		pointer-events: none;
	}

	.dest-card {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
		padding: 0.625rem 0.75rem;
		border: 1px solid #d1d5db;
		border-radius: 0.375rem;
		cursor: pointer;
		transition: border-color 0.15s, background 0.15s;
	}

	.dest-card input[type='radio'] {
		position: absolute;
		opacity: 0;
		pointer-events: none;
		width: 0;
		height: 0;
	}

	.dest-card--active {
		border-color: #2563eb;
		background: #eff6ff;
	}

	.dest-card:hover:not(.dest-card--active) {
		border-color: #9ca3af;
		background: #f9fafb;
	}

	.dest-card-title {
		font-size: 0.875rem;
		font-weight: 600;
		color: #111827;
	}

	.dest-card--active .dest-card-title {
		color: #1d4ed8;
	}

	.dest-card-desc {
		font-size: 0.75rem;
		color: #6b7280;
	}

	.dest-card-desc code {
		font-size: 0.7rem;
		background: #e5e7eb;
		padding: 0.1rem 0.25rem;
		border-radius: 0.2rem;
	}

	.dest-url-row {
		display: flex;
		gap: 0.5rem;
		align-items: center;
		margin-top: 0.375rem;
	}

	.dest-url-input {
		flex: 1;
		min-width: 0;
		padding: 0.3125rem 0.5rem;
		border: 1px solid #d1d5db;
		border-radius: 0.375rem;
		font-size: 0.8125rem;
	}

	.inline-code {
		font-family: ui-monospace, monospace;
		font-size: 0.75rem;
		background: #e5e7eb;
		padding: 0.1rem 0.3rem;
		border-radius: 0.25rem;
	}

	.sr-only {
		position: absolute;
		width: 1px;
		height: 1px;
		padding: 0;
		margin: -1px;
		overflow: hidden;
		clip: rect(0, 0, 0, 0);
		white-space: nowrap;
		border-width: 0;
	}

	/* ── Local Streaming section ─────────────────────────────────────────────── */
	.local-streaming-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 1rem;
	}

	.local-streaming-label {
		display: flex;
		flex-direction: column;
		gap: 0.125rem;
	}

	.toggle-label-main {
		font-size: 0.875rem;
		font-weight: 500;
		color: #111827;
	}

	.toggle-label-sub {
		font-size: 0.75rem;
		color: #6b7280;
	}

	.toggle-switch {
		flex-shrink: 0;
		display: flex;
		align-items: center;
		cursor: pointer;
	}

	.toggle-switch input[type='checkbox'] {
		position: absolute;
		opacity: 0;
		pointer-events: none;
		width: 0;
		height: 0;
	}

	.toggle-track {
		display: inline-block;
		width: 2.25rem;
		height: 1.25rem;
		background: #d1d5db;
		border-radius: 9999px;
		position: relative;
		transition: background 0.2s;
	}

	.toggle-track::after {
		content: '';
		position: absolute;
		top: 0.175rem;
		left: 0.175rem;
		width: 0.875rem;
		height: 0.875rem;
		background: #fff;
		border-radius: 50%;
		transition: transform 0.2s;
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.15);
	}

	.toggle-switch input:checked + .toggle-track {
		background: #2563eb;
	}

	.toggle-switch input:checked + .toggle-track::after {
		transform: translateX(1rem);
	}

	.lan-url-row {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		background: #f9fafb;
		border: 1px solid #e5e7eb;
		border-radius: 0.375rem;
		padding: 0.5rem 0.75rem;
	}

	.lan-url {
		flex: 1;
		font-family: ui-monospace, monospace;
		font-size: 0.8125rem;
		color: #374151;
		word-break: break-all;
	}

	.btn-copy {
		display: flex;
		align-items: center;
		gap: 0.25rem;
		flex-shrink: 0;
		padding: 0.25rem 0.5rem;
		background: #f3f4f6;
		color: #374151;
		border: 1px solid #d1d5db;
		border-radius: 0.375rem;
		font-size: 0.75rem;
		cursor: pointer;
		white-space: nowrap;
	}

	.btn-copy:hover {
		background: #e5e7eb;
	}

	.connector-header {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		margin-bottom: 1rem;
	}

	.connector-header h3 {
		font-size: 1rem;
		margin: 0 0 0.25rem;
	}

	.note {
		font-size: 0.875rem;
		color: #6b7280;
		margin: 0;
	}

	.form-row {
		margin-bottom: 0.75rem;
	}

	.form-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 0.75rem;
		margin-bottom: 0.75rem;
	}

	.field {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.field--full {
		grid-column: 1 / -1;
	}

	.field label,
	.checkbox-label {
		font-size: 0.875rem;
		color: #374151;
		font-weight: 500;
	}

	.checkbox-label {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		cursor: pointer;
	}

	input[type='text'],
	input[type='number'],
	input[type='password'] {
		padding: 0.375rem 0.625rem;
		border: 1px solid #d1d5db;
		border-radius: 0.375rem;
		font-size: 0.875rem;
		width: 100%;
		box-sizing: border-box;
	}

	input[type='text']:focus,
	input[type='number']:focus,
	input[type='password']:focus {
		outline: 2px solid #2563eb;
		outline-offset: 1px;
		border-color: #2563eb;
	}

	fieldset {
		border: none;
		padding: 0;
		margin: 0;
	}

	fieldset:disabled input,
	fieldset:disabled label {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.coming-soon-notice {
		font-size: 0.875rem;
		color: #6b7280;
		background: #f9fafb;
		border: 1px solid #e5e7eb;
		border-radius: 0.375rem;
		padding: 0.5rem 0.75rem;
		margin-bottom: 0.75rem;
	}

	.button-row {
		display: flex;
		gap: 0.5rem;
		flex-wrap: wrap;
	}

	.error {
		color: #dc2626;
		font-size: 0.875rem;
		margin: 0 0 0.75rem;
	}

	.btn-primary {
		padding: 0.5rem 1rem;
		background: #1d4ed8;
		color: #fff;
		border: none;
		border-radius: 0.375rem;
		font-size: 0.875rem;
		cursor: pointer;
	}

	.btn-primary:hover:not(:disabled) {
		background: #1e40af;
	}

	.btn-secondary {
		padding: 0.5rem 1rem;
		background: transparent;
		color: #1d4ed8;
		border: 1px solid #1d4ed8;
		border-radius: 0.375rem;
		font-size: 0.875rem;
		cursor: pointer;
	}

	.btn-secondary:hover:not(:disabled) {
		background: #eff6ff;
	}

	.btn-danger {
		padding: 0.5rem 1rem;
		background: #dc2626;
		color: #fff;
		border: none;
		border-radius: 0.375rem;
		font-size: 0.875rem;
		cursor: pointer;
	}

	.btn-danger:hover:not(:disabled) {
		background: #b91c1c;
	}

	button:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}
</style>
