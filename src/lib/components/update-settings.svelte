<script lang="ts">
	import { onMount } from 'svelte';
	import { _ } from 'svelte-i18n';
	import { browser } from '$app/environment';
	import { RefreshCw, Loader2 } from 'lucide-svelte';
	import Card from '$lib/components/ui/card.svelte';
	import Button from '$lib/components/ui/button.svelte';
	import { toast } from '$lib/utils/toast';

	let currentVersion = $state('...');
	let isChecking = $state(false);
	let isTauri = $state(false);

	function checkIsTauri(): boolean {
		if (!browser) return false;
		return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
	}

	async function loadVersion() {
		if (!isTauri) {
			currentVersion = '';
			return;
		}

		try {
			const { getVersion } = await import('@tauri-apps/api/app');
			currentVersion = await getVersion();
		} catch (error) {
			console.error('Failed to get app version:', error);
			currentVersion = '';
		}
	}

	async function handleCheckForUpdates() {
		if (!isTauri) return;

		isChecking = true;
		try {
			const { check } = await import('@tauri-apps/plugin-updater');
			const update = await check();

			if (update) {
				// If update found, the update-checker component will handle showing the dialog
				// We need to trigger the global update check
				window.dispatchEvent(new CustomEvent('check-for-updates'));
			} else {
				toast({
					title: $_('settings.update.upToDate'),
					variant: 'info'
				});
			}
		} catch (error) {
			const errorMsg = String(error);
			const isNoReleaseError = errorMsg.includes('Could not fetch a valid release JSON');

			if (isNoReleaseError) {
				toast({
					title: $_('settings.update.upToDate'),
					variant: 'info'
				});
			} else {
				console.error('Failed to check for updates:', error);
				toast({
					title: $_('toasts.error.title'),
					description: errorMsg,
					variant: 'error'
				});
			}
		} finally {
			isChecking = false;
		}
	}

	onMount(() => {
		isTauri = checkIsTauri();
		loadVersion();
	});
</script>

<Card>
	<svelte:fragment slot="title">{$_('settings.update.title')}</svelte:fragment>
	<svelte:fragment slot="description">{$_('settings.update.description')}</svelte:fragment>

	<svelte:fragment slot="content">
		<div class="space-y-4">
			<!-- Version Display -->
			<div class="rounded-md bg-muted px-4 py-3">
				{#if isTauri}
					{#if currentVersion}
						<p class="text-sm">
							<span class="text-muted-foreground">{$_('settings.update.currentVersion')}:</span>
							<span class="ml-2 font-mono font-medium">{currentVersion}</span>
						</p>
					{:else}
						<p class="text-sm text-muted-foreground">{$_('common.loading')}</p>
					{/if}
				{:else}
					<p class="text-sm text-muted-foreground">{$_('settings.update.devMode')}</p>
				{/if}
			</div>

			<!-- Check for Updates Button -->
			<Button
				buttonVariant="outline"
				className="w-full"
				onclick={handleCheckForUpdates}
				disabled={isChecking || !isTauri}
			>
				{#if isChecking}
					<Loader2 class="h-4 w-4 mr-2 animate-spin" />
					{$_('settings.update.checking')}
				{:else}
					<RefreshCw class="h-4 w-4 mr-2" />
					{$_('settings.update.checkForUpdates')}
				{/if}
			</Button>
		</div>
	</svelte:fragment>
</Card>
