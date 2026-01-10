<script lang="ts">
	import { cn } from "$lib/utils";
	import type { SystemStatus } from "$lib/stores/types";
	import Alert from "./alert.svelte";
	import AlertTitle from "./alert-title.svelte";
	import AlertDescription from "./alert-description.svelte";
	import Button from "./button.svelte";
	import Badge from "./badge.svelte";
	import ScrollArea from "./scroll-area.svelte";
	import { AlertCircle, Info, RefreshCw, Settings, Wifi } from "lucide-svelte";
	import { page } from '$app/state';
	import { systemStore, obsStatus } from '$lib/stores/system-store';
	import { _ } from 'svelte-i18n';

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

	const errorMessages: ErrorMessage[] = [
		{
			id: "airplay",
			titleKey: "errors.airplayDisplay.title",
			descriptionKey: "errors.airplayDisplay.description",
			status: "airplayDisplay",
			stepsKey: "errors.airplayDisplay.steps",
			imageUrl: "/macos-display-settings-airplay.jpg",
		},
		{
			id: "display-alignment",
			titleKey: "errors.displayAlignment.title",
			descriptionKey: "errors.displayAlignment.description",
			status: "displayAlignment",
			stepsKey: "errors.displayAlignment.steps",
			imageUrl: "/macos-display-arrangement-settings.jpg",
		},
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
			id: "rode",
			titleKey: "errors.rodeInterface.title",
			descriptionKey: "errors.rodeInterface.description",
			status: "rodeInterface",
			stepsKey: "errors.rodeInterface.steps",
			imageUrl: "/macos-sound-settings-audio-interface.jpg",
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

{#if activeErrors.length === 0}
	<!-- No errors, return null -->
{:else}
	<div class="space-y-4 mb-6">
		<div class="flex items-center justify-between">
			<div class="flex items-center gap-2">
				<AlertCircle class="h-5 w-5 text-destructive" />
					<h3 class="font-semibold">{$_('errors.title')}</h3>
					<Badge variant="destructive">{activeErrors.length}</Badge>
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
			{#each activeErrors as error (error.id)}
				<Alert variant="destructive">
					<AlertCircle class="h-4 w-4" />
					<AlertTitle>{$_(error.titleKey)}</AlertTitle>
					<AlertDescription className="flex items-start justify-between gap-4">
						<span>{$_(error.descriptionKey)}</span>
						{#if error.hasActions}
							<div class="flex gap-2">
								{#if page.url.pathname !== '/obs-settings'}
								<Button
									buttonVariant="outline"
									buttonSize="sm"
									className="shrink-0 bg-transparent"
									href={'/obs-settings'}
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
{/if}
