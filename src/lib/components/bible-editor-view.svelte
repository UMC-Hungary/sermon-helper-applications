<script lang="ts">
	import type { SystemStatus } from "$lib/stores/types";
	import ErrorMessages from "./ui/error-messages.svelte";

	// Event handler
	export let onRecheck: () => Promise<void> = async () => {};

	// Data props
	export let systemStatus: SystemStatus = {
		obs: true,
		rodeInterface: true,
		mainDisplay: true,
		secondaryDisplay: true,
		airplayDisplay: false,
		displayAlignment: false,
		youtubeLoggedIn: false,
	};

	// todo: implement
</script>

<div class="p-4 lg:p-8 space-y-6 pt-20 lg:pt-8">
	<ErrorMessages systemStatus={systemStatus} onRecheck={onRecheck} />

	<!-- TODO: Implement BibleEditorView component -->
	<div>
		<p>BibleEditorView component not yet implemented</p>
	</div>
</div>
