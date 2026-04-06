<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { openUrl } from '@tauri-apps/plugin-opener';
	import { toast } from 'svelte-sonner';
	import { _ } from 'svelte-i18n';
	import { updaterStore, type UpdateInfo } from '$lib/stores/updater.js';
	import { get } from 'svelte/store';

	onMount(async () => {
		try {
			const info = await invoke<UpdateInfo | null>('check_for_updates');
			if (info) {
				updaterStore.set({
					status: 'available',
					info,
					error: null,
					lastChecked: new Date(),
				});
				const t = get(_);
				toast.info(t('appSettings.updater.toast.title'), {
					description: t('appSettings.updater.toast.description', {
						values: { version: info.latestVersion },
					}),
					action: {
						label: t('appSettings.updater.download'),
						onClick: () => openUrl(info.releaseUrl),
					},
				});
			} else {
				updaterStore.update((s) => ({
					...s,
					status: 'up-to-date',
					lastChecked: new Date(),
				}));
			}
		} catch {
			// Silent failure on startup — don't disturb the user
			updaterStore.update((s) => ({ ...s, status: 'idle' }));
		}
	});
</script>
