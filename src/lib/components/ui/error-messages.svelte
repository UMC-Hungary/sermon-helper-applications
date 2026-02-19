<script lang="ts">
	import { cn } from "$lib/utils";
	import type { SystemStatus } from "$lib/stores/types";
	import Alert from "./alert.svelte";
	import AlertTitle from "./alert-title.svelte";
	import AlertDescription from "./alert-description.svelte";
	import Button from "./button.svelte";
	import Badge from "./badge.svelte";
	import ScrollArea from "./scroll-area.svelte";
	import { AlertCircle, AlertTriangle, Info, RefreshCw, Settings, Wifi, Plus, LogIn, Cpu } from "lucide-svelte";
	import YouTubeLoginModal from '$lib/components/youtube-login-modal.svelte';
	import { page } from '$app/stores';
	import { systemStore, obsStatus } from '$lib/stores/system-store';
	import { todayEvent } from '$lib/stores/event-store';
	import { appSettingsLoaded } from '$lib/utils/app-settings-store';
	import { _ } from 'svelte-i18n';
	import { failingRequiredDevices, obsDeviceStatuses } from '$lib/stores/obs-device-status-store';
	import { recheckDevice } from '$lib/utils/obs-device-checker';

	interface ErrorMessage {
		id: string;
		titleKey: string;
		descriptionKey: string;
		status: keyof SystemStatus;
		stepsKey: string;
		imageUrl?: string;
		hasActions?: boolean;
	}

	export let onRecheck: () => void = () => {};
	export let onReconnect: () => void = () => {};

	let isRechecking = false;
	let selectedErrorId: string | null = null;
	let dialogElement: HTMLDialogElement;
	let showYoutubeLoginModal = false;

	const errorMessages: ErrorMessage[] = [
		{
			id: "obs",
			titleKey: "errors.obs.title",
			descriptionKey: "errors.obs.description",
			status: "obs",
			stepsKey: "errors.obs.steps",
			imageUrl: "/obs-studio-websocket-settings.jpg",
			hasActions: true,
		},
		{
			id: "youtube-login",
			titleKey: "errors.youtubeLogin.title",
			descriptionKey: "errors.youtubeLogin.description",
			status: "youtubeLoggedIn",
			stepsKey: "errors.youtubeLogin.steps",
		},
	];

	$: activeErrors = errorMessages.filter((error) => !$systemStore[error.status]);

	// Check if there's no event for today (only when settings are loaded)
	$: hasNoEventToday = $appSettingsLoaded && !$todayEvent;

	// Check if we should show the no-event warning (not on events pages)
	$: currentPathname = $page.url.pathname;
	$: showNoEventWarning = hasNoEventToday && !currentPathname.startsWith('/events');

	// Total issues count (system errors + no event today warning + failing OBS devices)
	$: totalIssues = activeErrors.length + (showNoEventWarning ? 1 : 0) + $failingRequiredDevices.length;

	$: selectedError = selectedErrorId
		? errorMessages.find(e => e.id === selectedErrorId)
		: null;

	const handleRecheck = async () => {
		isRechecking = true;
		await onRecheck();
		setTimeout(() => (isRechecking = false), 500);
	};

	const handleReconnect = async () => {
		isRechecking = true;
		await onReconnect();
		setTimeout(() => (isRechecking = false), 500);
	};

	const openDialog = (errorId: string) => {
		selectedErrorId = errorId;
		dialogElement?.showModal();
	};

	const closeDialog = () => {
		dialogElement?.close();
		selectedErrorId = null;
	};

	const handleImageError = (event: Event) => {
		const target = event.currentTarget as HTMLImageElement;
		target.style.display = "none";
	};
</script>

{#if totalIssues === 0}
	<!-- No issues, return null -->
{:else}
	<div class="space-y-4 mb-6">
		<div class="flex items-center justify-between">
			<div class="flex items-center gap-2">
				<AlertCircle class="h-5 w-5 text-destructive" />
					<h3 class="font-semibold">{$_('errors.title')}</h3>
					<Badge variant="destructive">{totalIssues}</Badge>
			</div>
			<div class="flex gap-2">
				<Button
					buttonVariant="outline"
					buttonSize="sm"
					onclick={handleRecheck}
					disabled={isRechecking}
				>
					<RefreshCw class={cn("h-4 w-4 mr-2", isRechecking && "animate-spin")} />
					{$_('errors.recheck')}
				</Button>
			</div>
		</div>

		<div class="space-y-3">
			<!-- No Event Today Warning -->
			{#if showNoEventWarning}
				<Alert variant="warning">
					<AlertTriangle class="h-4 w-4" />
					<AlertTitle>{$_('events.noEventToday.title')}</AlertTitle>
					<AlertDescription className="flex items-start justify-between gap-4">
						<span>{$_('events.noEventToday.description')}</span>
						<Button
							buttonVariant="outline"
							buttonSize="sm"
							className="shrink-0 bg-transparent"
							href="/events/new"
						>
							<Plus class="h-4 w-4 mr-2" />
							{$_('events.noEventToday.action')}
						</Button>
					</AlertDescription>
				</Alert>
			{/if}

			<!-- System Errors -->
			{#each activeErrors as error (error.id)}
				<Alert variant="destructive">
					<AlertCircle class="h-4 w-4" />
					<AlertTitle>{$_(error.titleKey)}</AlertTitle>
					<AlertDescription className="flex items-start justify-between gap-4">
						<span>{$_(error.descriptionKey)}</span>
						{#if error.hasActions}
							<div class="flex gap-2">
								{#if $page.url.pathname !== '/obs-config'}
								<Button
									buttonVariant="outline"
									buttonSize="sm"
									className="shrink-0 bg-transparent"
									href={'/obs-config'}
								>
									<Settings class="h-4 w-4 mr-2" />
									{$_('errors.goToObsSettings')}
								</Button>
								{/if}
								{#if error.id === 'obs'}
									<Button
										buttonVariant="outline"
										buttonSize="sm"
										onclick={handleReconnect}
										disabled={isRechecking}
									>
                                    {#if $obsStatus.loading || $obsStatus.reconnecting || isRechecking}
                                        <RefreshCw class="h-4 w-4 mr-2 animate-spin" />
                                        {$_('errors.connecting')}
                                    {:else}
                                        <Wifi class={cn("h-4 w-4 mr-2")} />
                                        {$_('errors.reconnect')}
                                    {/if}
									</Button>
								{/if}
								<Button
									buttonVariant="outline"
									buttonSize="sm"
									onclick={() => openDialog(error.id)}
								>
									<Info class="h-4 w-4 mr-2" />
									{$_('errors.readMore')}
								</Button>
							</div>
						{:else if error.id === 'youtube-login'}
							<div class="flex gap-2">
								<Button
									buttonVariant="outline"
									buttonSize="sm"
									className="shrink-0 bg-transparent"
									onclick={() => (showYoutubeLoginModal = true)}
								>
									<LogIn class="h-4 w-4 mr-2" />
									{$_('youtube.modal.loginButton')}
								</Button>
								<Button
									buttonVariant="outline"
									buttonSize="sm"
									className="shrink-0 bg-transparent"
									onclick={() => openDialog(error.id)}
								>
									<Info class="h-4 w-4 mr-2" />
									{$_('errors.readMore')}
								</Button>
							</div>
						{:else}
							<Button
								buttonVariant="outline"
								buttonSize="sm"
								className="shrink-0 bg-transparent"
								onclick={() => openDialog(error.id)}
							>
								<Info class="h-4 w-4 mr-2" />
								{$_('errors.readMore')}
							</Button>
						{/if}
					</AlertDescription>
				</Alert>
			{/each}

			<!-- Dynamic OBS Device Errors -->
			{#each $failingRequiredDevices as device (device.id)}
				{@const status = $obsDeviceStatuses.get(device.id)}
				<Alert variant="destructive">
					<Cpu class="h-4 w-4" />
					<AlertTitle>{device.name} {$_('errors.obsDevice.notFound')}</AlertTitle>
					<AlertDescription className="flex items-start justify-between gap-4">
						<span>
							{$_('errors.obsDevice.description', { values: { name: device.name, type: device.type } })}
							{#if status?.error}
								<span class="text-xs block mt-1 opacity-75">{status.error}</span>
							{/if}
						</span>
						<div class="flex gap-2">
							<Button
								buttonVariant="outline"
								buttonSize="sm"
								className="shrink-0 bg-transparent"
								onclick={() => recheckDevice(device.id)}
							>
								<RefreshCw class="h-4 w-4 mr-2" />
								{$_('errors.recheck')}
							</Button>
							{#if $page.url.pathname !== '/obs-config'}
								<Button
									buttonVariant="outline"
									buttonSize="sm"
									className="shrink-0 bg-transparent"
									href="/obs-config"
								>
									<Settings class="h-4 w-4 mr-2" />
									{$_('errors.obsDevice.configure')}
								</Button>
							{/if}
						</div>
					</AlertDescription>
				</Alert>
			{/each}
		</div>
	</div>

	<!-- Dialog Modal -->
	<dialog
		bind:this={dialogElement}
		class="fixed z-50 bg-background border rounded-lg shadow-lg max-w-2xl max-h-[80vh] backdrop:bg-black/50 backdrop:backdrop-blur-sm p-0"
		onclose={() => (selectedErrorId = null)}
	>
		{#if selectedError}
			<div class="p-6">
				<div class="space-y-2 mb-4">
					<h2 class="text-lg font-semibold leading-none tracking-tight">
						{$_(selectedError.titleKey)}
					</h2>
					<p class="text-sm text-muted-foreground">
						{$_('errors.followSteps')}
					</p>
				</div>
				<ScrollArea className="max-h-[60vh] pr-4">
					<div class="space-y-4">
						{#if selectedError.imageUrl}
							<div class="rounded-lg overflow-hidden border">
								<img
									src={selectedError.imageUrl || "/placeholder.svg"}
									alt={$_(selectedError.titleKey)}
									class="w-full h-auto"
									onerror={handleImageError}
								/>
							</div>
						{/if}
						<ol class="space-y-3 list-decimal list-inside">
							{#each $_(selectedError.stepsKey) as step, index (index)}
								<li class="text-sm leading-relaxed">
									{step}
								</li>
							{/each}
						</ol>
					</div>
				</ScrollArea>
				<div class="mt-4 flex justify-end">
					<Button buttonVariant="outline" onclick={closeDialog}>
						{$_('errors.close')}
					</Button>
				</div>
			</div>
		{/if}
	</dialog>

	<!-- YouTube Login Modal -->
	<YouTubeLoginModal
		open={showYoutubeLoginModal}
		onClose={() => (showYoutubeLoginModal = false)}
	/>
{/if}
