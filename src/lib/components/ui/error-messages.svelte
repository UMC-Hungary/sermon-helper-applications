<script lang="ts">
	import { cn } from "$lib/utils";
	import type { SystemStatus } from "$lib/stores/types";
	import Alert from "./alert.svelte";
	import AlertTitle from "./alert-title.svelte";
	import AlertDescription from "./alert-description.svelte";
	import Button from "./button.svelte";
	import Badge from "./badge.svelte";
	import ScrollArea from "./scroll-area.svelte";
	import { AlertCircle, Info, RefreshCw } from "lucide-svelte";

	interface ErrorMessage {
		id: string;
		title: string;
		description: string;
		status: keyof SystemStatus;
		detailedSteps: string[];
		imageUrl?: string;
	}

	export let systemStatus: SystemStatus;
	export let onRecheck: () => void = () => {};

	let isRechecking = false;
	let selectedErrorId: string | null = null;
	let dialogElement: HTMLDialogElement;

	const errorMessages: ErrorMessage[] = [
		{
			id: "airplay",
			title: "AirPlay Display Not Connected",
			description: "The AirPlay display is not detected. Check your connection.",
			status: "airplayDisplay",
			detailedSteps: [
				"Open System Settings on your Mac",
				"Navigate to Displays section",
				"Click on the AirPlay Display dropdown",
				"Select your AirPlay device from the list",
				"Ensure your Mac and AirPlay device are on the same WiFi network",
				"If the device doesn't appear, restart both your Mac and the AirPlay receiver",
			],
			imageUrl: "/macos-display-settings-airplay.jpg",
		},
		{
			id: "display-alignment",
			title: "Display Alignment Incorrect",
			description: "Your displays are not properly aligned. Adjust in System Settings.",
			status: "displayAlignment",
			detailedSteps: [
				"Open System Settings on your Mac",
				"Go to Displays section",
				"Click on Arrange Displays",
				"Drag the display rectangles to match your physical setup",
				"Align the menu bar to your primary display",
				"Click Done to save the arrangement",
			],
			imageUrl: "/macos-display-arrangement-settings.jpg",
		},
		{
			id: "obs",
			title: "OBS Not Running",
			description: "OBS Studio is not currently running on your system.",
			status: "obs",
			detailedSteps: [
				"Open OBS Studio application",
				"Ensure OBS is fully loaded before proceeding",
				"Check that the WebSocket server is enabled in OBS Tools > WebSocket Server Settings",
				"Verify the port and password match your configuration",
				"Click Recheck to verify the connection",
			],
			imageUrl: "/obs-studio-websocket-settings.jpg",
		},
		{
			id: "rode",
			title: "Rode Audio Interface Not Connected",
			description: "The Rode audio interface is not detected.",
			status: "rodeInterface",
			detailedSteps: [
				"Check the USB/Thunderbolt connection to your Mac",
				"Open System Settings > Sound",
				"Select the Input tab",
				"Verify your Rode interface appears in the list",
				"Set it as the default input device if needed",
				"Try unplugging and reconnecting the interface",
			],
			imageUrl: "/macos-sound-settings-audio-interface.jpg",
		},
		{
			id: "youtube-login",
			title: "YouTube Not Logged In",
			description: "You need to log in to YouTube to schedule events.",
			status: "youtubeLoggedIn",
			detailedSteps: [
				"Click the 'Login with Google' button",
				"Select your church's Google account",
				"Grant the necessary permissions for YouTube access",
				"Ensure your account has permission to create live streams",
				"After successful login, you can schedule YouTube events",
			],
		},
	];

	$: activeErrors = errorMessages.filter((error) => !systemStatus[error.status]);

	$: selectedError = selectedErrorId
		? errorMessages.find(e => e.id === selectedErrorId)
		: null;

	const handleRecheck = async () => {
		isRechecking = true;
		await onRecheck();
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
				<h3 class="font-semibold">System Issues Detected</h3>
				<Badge variant="destructive">{activeErrors.length}</Badge>
			</div>
			<Button
				buttonVariant="outline"
				buttonSize="sm"
				onclick={handleRecheck}
				disabled={isRechecking}
			>
				<RefreshCw class={cn("h-4 w-4 mr-2", isRechecking && "animate-spin")} />
				Re-check
			</Button>
		</div>

		<div class="space-y-3">
			{#each activeErrors as error (error.id)}
				<Alert variant="destructive">
					<AlertCircle class="h-4 w-4" />
					<AlertTitle>{error.title}</AlertTitle>
					<AlertDescription className="flex items-start justify-between gap-4">
						<span>{error.description}</span>
						<Button
							buttonVariant="outline"
							buttonSize="sm"
							className="shrink-0 bg-transparent"
							onclick={() => openDialog(error.id)}
						>
							<Info class="h-4 w-4 mr-2" />
							Read More
						</Button>
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
						{selectedError.title}
					</h2>
					<p class="text-sm text-muted-foreground">
						Follow these steps to resolve the issue
					</p>
				</div>
				<ScrollArea className="max-h-[60vh] pr-4">
					<div class="space-y-4">
						{#if selectedError.imageUrl}
							<div class="rounded-lg overflow-hidden border">
								<img
									src={selectedError.imageUrl || "/placeholder.svg"}
									alt={selectedError.title}
									class="w-full h-auto"
									onerror={handleImageError}
								/>
							</div>
						{/if}
						<ol class="space-y-3 list-decimal list-inside">
							{#each selectedError.detailedSteps as step, index (index)}
								<li class="text-sm leading-relaxed">
									{step}
								</li>
							{/each}
						</ol>
					</div>
				</ScrollArea>
				<div class="mt-4 flex justify-end">
					<Button buttonVariant="outline" onclick={closeDialog}>
						Close
					</Button>
				</div>
			</div>
		{/if}
	</dialog>
{/if}
