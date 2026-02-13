<script lang="ts">
	import Card from '$lib/components/ui/card.svelte';
	import Button from '$lib/components/ui/button.svelte';
	import { Puzzle, ExternalLink } from 'lucide-svelte';

	const RELEASES_URL = 'https://github.com/UMC-Hungary/Sermon-Helper-Companion-Plugin/releases';

	async function handleOpenReleases() {
		try {
			const { openUrl } = await import('@tauri-apps/plugin-opener');
			await openUrl(RELEASES_URL);
		} catch {
			window.open(RELEASES_URL, '_blank');
		}
	}
</script>

<Card>
	<svelte:fragment slot="title">
		<Puzzle class="h-5 w-5" />
		Companion Plugin
	</svelte:fragment>

	<svelte:fragment slot="description">
		Install the Sermon Helper module for Bitfocus Companion to control presentations from your
		stream deck.
	</svelte:fragment>

	<svelte:fragment slot="content">
		<div class="space-y-4">
			<Button buttonVariant="outline" onclick={handleOpenReleases} className="w-full">
				<ExternalLink class="mr-2 h-4 w-4" />
				Download from GitHub
			</Button>

			<div class="rounded-lg bg-muted/50 p-4 space-y-3">
				<h4 class="font-medium text-sm">Setup Instructions</h4>
				<ol class="list-decimal list-inside space-y-2 text-sm text-muted-foreground">
					<li>Download the latest release from the link above</li>
					<li>
						In Companion, go to <span class="font-medium text-foreground">Settings</span> and
						enable
						<span class="font-medium text-foreground">Developer Modules</span> under the developer
						tab
					</li>
					<li>
						Set the developer modules path to a folder (e.g.
						<code class="text-xs bg-background px-1 py-0.5 rounded">companion-module-dev</code>)
					</li>
					<li>Extract/copy the downloaded plugin into that folder</li>
					<li>Restart Companion — the module will appear as "Sermon Helper"</li>
					<li>Configure the module with Host, Port, and Auth Token from the Discovery settings</li>
				</ol>
			</div>
		</div>
	</svelte:fragment>
</Card>
