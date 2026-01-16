<script lang="ts">
	import { onMount } from 'svelte';
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

	async function checkForUpdates(showNoUpdateToast = false) {
		try {
			// Dynamic import to avoid SSR issues
			const { check } = await import('@tauri-apps/plugin-updater');
			const update = await check();

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
					variant: 'info',
					duration: 3000
				});
			}
		} catch (error) {
			console.error('Failed to check for updates:', error);
			if (showNoUpdateToast) {
				toast({
					title: 'Update check failed',
					description: String(error),
					variant: 'error',
					duration: 5000
				});
			}
		}
	}

	async function downloadAndInstall() {
		if (!updateInstance) return;

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

			// Wait a moment for the toast to be visible
			await new Promise(resolve => setTimeout(resolve, 1500));

			// Dynamic import to avoid SSR issues
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

	$effect(() => {
		if (downloadTotal > 0) {
			const percent = Math.round((downloadProgress / downloadTotal) * 100);
			console.log(`Download progress: ${percent}%`);
		}
	});

	onMount(() => {
		// Check for updates after a short delay to not block app startup
		const timer = setTimeout(() => {
			checkForUpdates(false);
		}, 3000);

		return () => clearTimeout(timer);
	});

	// Export for manual check from settings
	export { checkForUpdates };
</script>

{#if showUpdateDialog && updateAvailable}
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
