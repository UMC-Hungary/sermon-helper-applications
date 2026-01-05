<script context="module" lang="ts">
	import type { SystemStatus as SystemStatusType } from '$lib/stores/types';
	
	export type ViewType = "dashboard" | "bible" | "youtube-schedule" | "youtube-events" | "obs-settings";
</script>

<script lang="ts">
	import Sidebar from '$lib/components/sidebar.svelte';
	import DashboardView from '$lib/components/dashboard-view.svelte';
	import BibleEditorView from '$lib/components/bible-editor-view.svelte';
	import YoutubeScheduleView from '$lib/components/youtube-schedule-view.svelte';
	import YoutubeEventsView from '$lib/components/youtube-events-view.svelte';
	import ObsSettingsView from '$lib/components/obs-settings-view.svelte';

	let activeView: ViewType = 'dashboard';
	let isMobileMenuOpen = false;

	let systemStatus: SystemStatusType = {
		obs: true,
		rodeInterface: true,
		mainDisplay: true,
		secondaryDisplay: true,
		airplayDisplay: false,
		displayAlignment: false,
		youtubeLoggedIn: false,
	};

	let obsStreaming = false;
	let youtubeOnAir = false;

	let textus = 'John 3:16-21';
	let leckio = 'Romans 8:28-39';

	let currentSermon = {
		youtubeTitle: 'The Love of God - Sunday Service',
		youtubeScheduled: true,
		streamStarted: false,
	};

	const handleSystemRecheck = async () => {
		console.log('[v0] Rechecking system status...');
		await new Promise((resolve) => setTimeout(resolve, 500));
	};

	const handleYoutubeLogin = () => {
		systemStatus = { ...systemStatus, youtubeLoggedIn: true };
	};

	const handleNavigate = (view: string) => {
		activeView = view as ViewType;
	};

	let isRechecking = false;

	function handleRecheck(): void {
		isRechecking = true;
		// Simulate recheck logic
		setTimeout(() => {
			isRechecking = false;
		}, 2000);
	}

</script>

<div class="flex h-screen overflow-hidden bg-background">
	<Sidebar
		activeView={activeView}
		onViewChange={handleNavigate}
		isMobileMenuOpen={isMobileMenuOpen}
		onMobileMenuToggle={() => isMobileMenuOpen = !isMobileMenuOpen}
		systemStatus={systemStatus}
		currentSermon={{ textus, leckio, ...currentSermon }}
	/>

	<main class="flex-1 overflow-y-auto">
		{#if activeView === 'dashboard'}
				<DashboardView
					onNavigate={handleNavigate}
					currentSermon={{ textus, leckio, ...currentSermon }}
					systemStatus={systemStatus}
					onRecheck={handleSystemRecheck}
					textus={textus}
					leckio={leckio}
					onTextusChange={(text) => textus = text}
					onLeckioChange={(text) => leckio = text}
					obsStreaming={obsStreaming}
					youtubeOnAir={youtubeOnAir}
					onStartObsStream={() => obsStreaming = true}
					onStopObsStream={() => obsStreaming = false}
					onYoutubeGoLive={() => youtubeOnAir = true}
					onYoutubeStopLive={() => youtubeOnAir = false}
				/>
			{:else if activeView === 'bible'}
				<BibleEditorView systemStatus={systemStatus} onRecheck={handleSystemRecheck} />
			{:else if activeView === 'youtube-schedule'}
				<YoutubeScheduleView
					systemStatus={systemStatus}
					onRecheck={handleSystemRecheck}
					youtubeLoggedIn={systemStatus.youtubeLoggedIn}
					onYoutubeLogin={handleYoutubeLogin}
					onNavigateBack={() => activeView = "dashboard"}
				/>
			{:else if activeView === 'youtube-events'}
				<YoutubeEventsView systemStatus={systemStatus} onRecheck={handleSystemRecheck} />
			{:else if activeView === 'obs-settings'}
				<ObsSettingsView systemStatus={systemStatus} onRecheck={handleSystemRecheck} />
			{/if}
	</main>
</div>