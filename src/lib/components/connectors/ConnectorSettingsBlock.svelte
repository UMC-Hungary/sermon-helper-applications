<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { openUrl } from '@tauri-apps/plugin-opener';
	import { _ } from 'svelte-i18n';
	import {
		obsConfig,
		obsStatus,
		vmixConfig,
		vmixStatus,
		atemConfig,
		atemStatus,
		youtubeConfig,
		youtubeStatus,
		facebookConfig,
		facebookStatus,
		discordConfig,
		discordStatus,
		broadlinkConfig,
		broadlinkStatus
	} from '$lib/stores/connectors.js';
	import type {
		ObsConfig,
		VmixConfig,
		AtemConfig,
		YouTubeConfig,
		FacebookConfig,
		DiscordConfig,
		BroadlinkConfig
	} from '$lib/stores/connectors.js';
	import { findConnector } from '$lib/connectors/registry.js';
	import ConnectorStatusBadge from './ConnectorStatusBadge.svelte';
	import { youtubeLogout, facebookLogout } from '$lib/api/connectors.js';
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
				{:else if connectorId === 'youtube'}
					<p class="note">{$_('appSettings.connectors.youtube.subtitle')}</p>
				{:else if connectorId === 'facebook'}
					<p class="note">{$_('appSettings.connectors.facebook.subtitle')}</p>
				{:else if connectorId === 'discord'}
					<p class="note">{$_('appSettings.connectors.discord.subtitle')}</p>
				{:else if connectorId === 'broadlink'}
					<p class="note">{$_('appSettings.connectors.broadlink.subtitle')}</p>
				{/if}
			</div>
			{#if connectorId === 'obs'}
				<ConnectorStatusBadge name="OBS" status={$obsStatus} />
			{:else if connectorId === 'vmix'}
				<ConnectorStatusBadge name="VMix" status={$vmixStatus} />
			{:else if connectorId === 'atem'}
				<ConnectorStatusBadge name="ATEM" status={$atemStatus} />
			{:else if connectorId === 'youtube'}
				<ConnectorStatusBadge name="YouTube" status={$youtubeStatus} />
			{:else if connectorId === 'facebook'}
				<ConnectorStatusBadge name="Facebook" status={$facebookStatus} />
			{:else if connectorId === 'discord'}
				<ConnectorStatusBadge name="Discord" status={$discordStatus} />
			{:else if connectorId === 'broadlink'}
				<ConnectorStatusBadge name="Broadlink" status={$broadlinkStatus} />
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

		<!-- ── Broadlink form ──────────────────────────────────────────────── -->
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
