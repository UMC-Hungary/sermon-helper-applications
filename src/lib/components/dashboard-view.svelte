<script lang="ts">
	import type { SystemStatus } from "$lib/stores/types";
	import ErrorMessages from "./ui/error-messages.svelte";

	// Event handlers - these are expected to be used by parent
	export let onNavigate: (view: string) => void = () => {};
	export let onRecheck: () => Promise<void> = async () => {};
	export let onTextusChange: (text: string) => void = () => {};
	export let onLeckioChange: (text: string) => void = () => {};
	export let onStartObsStream: () => void = () => {};
	export let onStopObsStream: () => void = () => {};
	export let onYoutubeGoLive: () => void = () => {};
	export let onYoutubeStopLive: () => void = () => {};

	// Data props - these are used for data flow
	export let currentSermon = {
		youtubeTitle: '',
		youtubeScheduled: true,
		streamStarted: false,
		textus: '',
		leckio: '',
	};
	export let systemStatus: SystemStatus = {
		obs: true,
		rodeInterface: true,
		mainDisplay: true,
		secondaryDisplay: true,
		airplayDisplay: false,
		displayAlignment: false,
		youtubeLoggedIn: false,
	};

	// Reactive state - these might change during component lifecycle
	export let textus: string = '';
	export let leckio: string = '';
	export let obsStreaming: boolean = false;
	export let youtubeOnAir: boolean = false;

	// todo: implement
</script>

<div class="p-4 lg:p-8 space-y-6 pt-20 lg:pt-8">
	<ErrorMessages systemStatus={systemStatus} onRecheck={onRecheck} />

	<!-- TODO: Implement DashboardView component -->
	<div>
		<p>DashboardView component not yet implemented</p>
	</div>
</div>
