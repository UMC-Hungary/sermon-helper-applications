<script lang="ts">
	import { _ } from 'svelte-i18n';
	import { Download, Upload, AlertTriangle, Loader2 } from 'lucide-svelte';
	import Card from '$lib/components/ui/card.svelte';
	import Button from '$lib/components/ui/button.svelte';
	import Alert from '$lib/components/ui/alert.svelte';
	import Checkbox from '$lib/components/ui/checkbox.svelte';
	import { toast } from '$lib/utils/toast';
	import { isTauriApp } from '$lib/utils/storage-helpers';
	import {
		exportSettings,
		importSettings,
		saveSettingsWithDialog,
		loadSettingsWithDialog,
		downloadSettings,
	} from '$lib/utils/import-export-service';

	let includeYoutubeTokens = false;
	let isExporting = false;
	let isImporting = false;
	let fileInput: HTMLInputElement;

	$: isTauri = isTauriApp();

	async function handleExport() {
		isExporting = true;
		try {
			if (isTauri) {
				const success = await saveSettingsWithDialog({ includeYoutubeTokens });
				if (success) {
					toast({
						title: $_('settings.importExport.exportSuccess'),
						variant: 'success',
					});
				}
			} else {
				await downloadSettings({ includeYoutubeTokens });
				toast({
					title: $_('settings.importExport.exportSuccess'),
					variant: 'success',
				});
			}
		} catch (error) {
			toast({
				title: $_('toasts.error.title'),
				description: error instanceof Error ? error.message : String(error),
				variant: 'error',
			});
		} finally {
			isExporting = false;
		}
	}

	async function handleImport() {
		if (isTauri) {
			isImporting = true;
			try {
				const success = await loadSettingsWithDialog();
				if (success) {
					toast({
						title: $_('settings.importExport.importSuccess'),
						variant: 'success',
					});
					// Reload the page to apply imported settings
					window.location.reload();
				}
			} catch (error) {
				toast({
					title: $_('settings.importExport.importError'),
					description: error instanceof Error ? error.message : String(error),
					variant: 'error',
				});
			} finally {
				isImporting = false;
			}
		} else {
			// Browser mode: trigger file input
			fileInput?.click();
		}
	}

	async function handleFileSelect(event: Event) {
		const target = event.target as HTMLInputElement;
		const file = target.files?.[0];
		if (!file) return;

		isImporting = true;
		try {
			await importSettings(file);
			toast({
				title: $_('settings.importExport.importSuccess'),
				variant: 'success',
			});
			// Reload the page to apply imported settings
			window.location.reload();
		} catch (error) {
			toast({
				title: $_('settings.importExport.importError'),
				description: error instanceof Error ? error.message : String(error),
				variant: 'error',
			});
		} finally {
			isImporting = false;
			// Reset file input
			target.value = '';
		}
	}
</script>

<Card>
	<svelte:fragment slot="title">{$_('settings.importExport.title')}</svelte:fragment>
	<svelte:fragment slot="description">{$_('settings.importExport.description')}</svelte:fragment>

	<svelte:fragment slot="content">
		<div class="space-y-4">
			<!-- Export Section -->
			<div class="space-y-3">
				<label class="flex items-center gap-2 cursor-pointer">
					<Checkbox
						bind:checked={includeYoutubeTokens}
					/>
					<span class="text-sm text-muted-foreground">
						{$_('settings.importExport.includeYoutubeTokens')}
					</span>
				</label>

				<Button
					buttonVariant="outline"
					className="w-full"
					onclick={handleExport}
					disabled={isExporting}
				>
					{#if isExporting}
						<Loader2 class="h-4 w-4 mr-2 animate-spin" />
					{:else}
						<Download class="h-4 w-4 mr-2" />
					{/if}
					{$_('settings.importExport.export')}
				</Button>
			</div>

			<!-- Import Section -->
			<div class="pt-4 border-t border-border">
				<Button
					buttonVariant="outline"
					className="w-full"
					onclick={handleImport}
					disabled={isImporting}
				>
					{#if isImporting}
						<Loader2 class="h-4 w-4 mr-2 animate-spin" />
					{:else}
						<Upload class="h-4 w-4 mr-2" />
					{/if}
					{$_('settings.importExport.import')}
				</Button>

				<!-- Hidden file input for browser mode -->
				<input
					type="file"
					accept=".json"
					class="hidden"
					bind:this={fileInput}
					onchange={handleFileSelect}
				/>
			</div>

			<!-- Warning -->
			<Alert variant="warning" className="mt-4">
				<AlertTriangle class="h-4 w-4" />
				<div data-slot="alert-description">{$_('settings.importExport.warning')}</div>
			</Alert>
		</div>
	</svelte:fragment>
</Card>
