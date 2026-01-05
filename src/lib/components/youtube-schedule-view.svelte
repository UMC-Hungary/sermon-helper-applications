<script lang="ts">
	import type { SystemStatus } from "$lib/stores/types";
	import ErrorMessages from "./ui/error-messages.svelte";

	// Event handlers
	export let onRecheck: () => Promise<void> = async () => {};
	export let onYoutubeLogin: () => void = () => {};
	export let onNavigateBack: () => void = () => {};

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
	export let youtubeLoggedIn: boolean = false;

	// todo: implement
</script>

<div class="p-4 lg:p-8 space-y-6 pt-20 lg:pt-8">
	<ErrorMessages systemStatus={systemStatus} onRecheck={onRecheck} />

	<!-- TODO: Implement YoutubeScheduleView component -->
	<div>
		<p>YoutubeScheduleView component not yet implemented</p>
	</div>
</div>
