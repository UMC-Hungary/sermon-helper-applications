<script lang="ts">
	import { open } from '@tauri-apps/plugin-dialog';
	import { FilePlus } from 'lucide-svelte';
	import Button from '$lib/components/ui/button.svelte';
	import { _ } from 'svelte-i18n';
	import { getFileInfo } from '$lib/utils/file-info';
	import type { RecordingFile } from '$lib/types/event';

	interface Props {
		onFileAdded: (file: RecordingFile) => void;
	}

	let { onFileAdded }: Props = $props();

	async function handleSelectFiles() {
		const selected = await open({
			multiple: true,
			filters: [
				{
					name: 'Video files',
					extensions: ['mp4', 'mkv', 'flv', 'mov', 'avi', 'webm', 'ts']
				}
			]
		});

		if (!selected) return;

		const paths = Array.isArray(selected) ? selected : [selected];

		for (const filePath of paths) {
			try {
				const fileInfo = await getFileInfo(filePath);
				onFileAdded(fileInfo);
			} catch (error) {
				console.error('[FinalizeManualAdd] Failed to get file info:', error);
			}
		}
	}
</script>

<div class="space-y-3">
	<h3 class="text-sm font-medium">{$_('finalize.manual.title')}</h3>
	<Button
		buttonVariant="outline"
		buttonSize="sm"
		onclick={handleSelectFiles}
	>
		<FilePlus class="h-4 w-4" />
		{$_('finalize.manual.selectFiles')}
	</Button>
</div>
