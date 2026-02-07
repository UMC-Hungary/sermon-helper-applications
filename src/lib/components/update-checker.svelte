<script lang="ts">
	import { onMount } from 'svelte';
	import { browser } from '$app/environment';
	import { toast } from '$lib/utils/toast';
	import Button from '$lib/components/ui/button.svelte';
	import Card from '$lib/components/ui/card.svelte';

	let updateAvailable = $state(false);
	let updateVersion = $state('');
	let updateNotes = $state('');
	let isDownloading = $state(false);
	let downloadProgress = $state(0);
	let downloadTotal = $state(0);
	let showUpdateDialog = $state(false);
	let updateInstance: any = $state(null);
	let isTauri = $state(false);

	// Logging helper
	async function log(level: 'info' | 'error' | 'warn' | 'debug', message: string) {
		console.log(`[UpdateChecker][${level.toUpperCase()}] ${message}`);
		if (browser && '__TAURI_INTERNALS__' in window) {
			try {
				const { info, error, warn, debug } = await import('@tauri-apps/plugin-log');
				const logFn = { info, error, warn, debug }[level];
				await logFn(`[UpdateChecker] ${message}`);
			} catch (e) {
				// Ignore logging errors
			}
		}
	}

	function checkIsTauri(): boolean {
		if (!browser) return false;
		return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
	}

	async function checkForUpdates(showNoUpdateToast = false) {
		if (!isTauri) {
			await log('info', 'Skipping update check - not in Tauri environment');
			return;
		}

		await log('info', 'Checking for updates...');
		try {
			const { check } = await import('@tauri-apps/plugin-updater');
			await log('info', 'Updater plugin loaded, calling check()...');
			const update = await check();
			await log('info', `Update check result: ${update ? `v${update.version} available` : 'no update'}`);

			if (update) {
				updateInstance = update;
				updateVersion = update.version;
				updateNotes = update.body || '';
				updateAvailable = true;
				showUpdateDialog = true;
			} else if (showNoUpdateToast) {
				toast({
					title: 'No updates available',
					description: 'You are running the latest version.',
					variant: 'info'
				});
			}
		} catch (error) {
			const errorMsg = String(error);
			// "Could not fetch a valid release JSON" is expected when no releases exist yet
			const isNoReleaseError = errorMsg.includes('Could not fetch a valid release JSON');

			if (isNoReleaseError) {
				await log('info', 'No releases published yet - skipping update check');
			} else {
				await log('error', `Update check failed: ${errorMsg}`);
				console.error('Failed to check for updates:', error);
			}

			if (showNoUpdateToast && !isNoReleaseError) {
				toast({
					title: 'Update check failed',
					description: errorMsg,
					variant: 'error',
					duration: 5000
				});
			}
		}
	}

	async function downloadAndInstall() {
		if (!updateInstance || !isTauri) return;

		isDownloading = true;
		downloadProgress = 0;
		downloadTotal = 0;

		try {
			await updateInstance.downloadAndInstall((event: any) => {
				switch (event.event) {
					case 'Started':
						downloadTotal = event.data.contentLength || 0;
						break;
					case 'Progress':
						downloadProgress += event.data.chunkLength;
						break;
					case 'Finished':
						break;
				}
			});

			toast({
				title: 'Update installed',
				description: 'The application will restart now.',
				variant: 'success',
				duration: 2000
			});

			await new Promise(resolve => setTimeout(resolve, 1500));

			const { relaunch } = await import('@tauri-apps/plugin-process');
			await relaunch();
		} catch (error) {
			console.error('Failed to install update:', error);
			toast({
				title: 'Update failed',
				description: String(error),
				variant: 'error',
				duration: 5000
			});
			isDownloading = false;
		}
	}

	function dismissUpdate() {
		showUpdateDialog = false;
	}

	function formatBytes(bytes: number): string {
		if (bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
	}

	onMount(() => {
		log('info', 'UpdateChecker component mounted');
		isTauri = checkIsTauri();
		log('info', `Tauri environment detected: ${isTauri}`);

		if (!isTauri) {
			log('info', 'Not running in Tauri, skipping update check');
			return;
		}

		let timer: ReturnType<typeof setTimeout> | undefined;

		(async () => {
			// Skip update check in dev mode
			if (import.meta.env.DEV) {
				await log('info', 'Running in dev mode, skipping automatic update check');
				return;
			}

			// Skip update check for version 0.1.0 (development version)
			try {
				const { getVersion } = await import('@tauri-apps/api/app');
				const version = await getVersion();
				await log('info', `App version: ${version}`);
				if (version === '0.1.0') {
					await log('info', 'Version is 0.1.0 (development), skipping automatic update check');
					return;
				}
			} catch (e) {
				await log('warn', `Failed to get app version: ${e}`);
			}

			log('info', 'Scheduling update check in 3 seconds...');
			timer = setTimeout(() => {
				checkForUpdates(false);
			}, 3000);
		})();

		return () => {
			if (timer) clearTimeout(timer);
		};
	});

	export { checkForUpdates };
</script>

{#if isTauri && showUpdateDialog && updateAvailable}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
		<Card className="w-full max-w-md mx-4">
			<svelte:fragment slot="title">
				Update Available
			</svelte:fragment>
			<svelte:fragment slot="description">
				Version {updateVersion} is ready to install
			</svelte:fragment>
			<svelte:fragment slot="content">
				{#if updateNotes}
					<div class="max-h-40 overflow-y-auto rounded bg-muted p-3 text-sm">
						{updateNotes}
					</div>
				{/if}

				{#if isDownloading}
					<div class="mt-4 space-y-2">
						<div class="flex justify-between text-sm text-muted-foreground">
							<span>Downloading...</span>
							<span>
								{#if downloadTotal > 0}
									{formatBytes(downloadProgress)} / {formatBytes(downloadTotal)}
								{:else}
									{formatBytes(downloadProgress)}
								{/if}
							</span>
						</div>
						<div class="h-2 w-full overflow-hidden rounded-full bg-muted">
							<div
								class="h-full bg-primary transition-all duration-300"
								style="width: {downloadTotal > 0 ? (downloadProgress / downloadTotal) * 100 : 0}%"
							></div>
						</div>
					</div>
				{/if}
			</svelte:fragment>
			<svelte:fragment slot="footer">
				<div class="flex w-full gap-2 justify-end">
					{#if !isDownloading}
						<Button buttonVariant="outline" onclick={dismissUpdate}>
							Later
						</Button>
						<Button onclick={downloadAndInstall}>
							Update Now
						</Button>
					{:else}
						<Button disabled>
							Installing...
						</Button>
					{/if}
				</div>
			</svelte:fragment>
		</Card>
	</div>
{/if}
