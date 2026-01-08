<script lang="ts">
	import Card from "$lib/components/ui/card.svelte";
	import Button from "$lib/components/ui/button.svelte";
	import Input from "$lib/components/ui/input.svelte";
	import Label from "$lib/components/ui/label.svelte";
	import Badge from "$lib/components/ui/badge.svelte";
	import Separator from "$lib/components/ui/separator.svelte";
	import { toast } from "$lib/utils/toast";
	import {
		Book,
		FileText,
		Youtube,
		RefreshCw,
		Edit2,
		PlayCircle,
		StopCircle,
		Radio
	} from "lucide-svelte";

	// Event handlers
	export let onRecheck: () => Promise<void> = async () => {};
	export let onTextusChange: (text: string) => void = () => {};
	export let onLeckioChange: (text: string) => void = () => {};
	export let onStartObsStream: () => void = () => {};
	export let onStopObsStream: () => void = () => {};
	export let onYoutubeGoLive: () => void = () => {};
	export let onYoutubeStopLive: () => void = () => {};

	// Data props
	export let currentSermon = {
		youtubeTitle: '',
		youtubeScheduled: true,
		streamStarted: false,
		textus: '',
		leckio: '',
	};

	// Reactive state
	export let textus: string = '';
	export let leckio: string = '';
	export let obsStreaming: boolean = false;
	export let youtubeOnAir: boolean = false;

	// Local state
	let obsTitle = "The Love of God";

	// Event handlers with toast notifications
	const handleUpdateOBS = () => {
		toast({
			title: "OBS Updated",
			description: "Sermon title has been updated in OBS",
			variant: "success"
		});
	};

	const handleGenerateTextusPPT = () => {
		toast({
			title: "PPT Generated",
			description: "PowerPoint for Textus is ready for download",
			variant: "success"
		});
	};

	const handleGenerateLeckioPPT = () => {
		toast({
			title: "PPT Generated",
			description: "PowerPoint for Leckio is ready for download",
			variant: "success"
		});
	};
</script>

<!-- Section 2: Dashboard Header -->
<div class="mt-12 lg:mt-0">
	<h2 class="text-3xl font-bold tracking-tight">Dashboard</h2>
	<p class="text-muted-foreground">Manage your sermon preparation and system monitoring</p>
</div>

<!-- Section 3: Quick Access Cards -->
<div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
	<!-- Bible Editor Card -->
	<Card clickable href={'/bible'}>
		<svelte:fragment slot="title">
			<Book class="h-5 w-5" />
			Bible Editor
		</svelte:fragment>
		<svelte:fragment slot="description">Search and edit scripture texts</svelte:fragment>
		<svelte:fragment slot="content">
			<Badge>Ready to use</Badge>
		</svelte:fragment>
	</Card>

	<!-- Schedule Event Card -->
	<Card clickable href={'/youtube-schedule'}>
		<svelte:fragment slot="title">
			<Youtube class="h-5 w-5" />
			Schedule Event
		</svelte:fragment>
		<svelte:fragment slot="description">Plan your YouTube live streams</svelte:fragment>
		<svelte:fragment slot="content">
			<Badge variant="outline">Schedule Now</Badge>
		</svelte:fragment>
	</Card>

	<!-- View Events Card -->
	<Card clickable href={'/youtube-events'}>
		<svelte:fragment slot="title">
			<FileText class="h-5 w-5" />
			View Events
		</svelte:fragment>
		<svelte:fragment slot="description">See your upcoming events</svelte:fragment>
		<svelte:fragment slot="content">
			<Badge variant="secondary">View All</Badge>
		</svelte:fragment>
	</Card>
</div>

<!-- Section 4: Current Sermon Status Card -->
<Card>
	<svelte:fragment slot="title">Current Sermon Status</svelte:fragment>
	<svelte:fragment slot="description">Monitor your live streaming status</svelte:fragment>
	<svelte:fragment slot="content">
		{#if currentSermon.youtubeScheduled}
			<div class="space-y-4">
				<!-- YouTube Title -->
				<div class="flex items-center justify-between">
					<div>
						<p class="text-sm font-medium">YouTube Event</p>
						<p class="text-lg font-semibold">{currentSermon.youtubeTitle}</p>
					</div>
					<Button
							buttonVariant="ghost"
							buttonSize="icon"
							href={'/youtube-schedule'}
					>
						<Edit2 class="h-4 w-4" />
					</Button>
				</div>

				<!-- Status Badges -->
				<div class="flex gap-2">
					{#if youtubeOnAir}
						<Badge variant="destructive">
							<Radio class="h-3 w-3 mr-1 animate-pulse" />
							Live Now
						</Badge>
					{:else}
						<Badge variant="secondary">Scheduled</Badge>
					{/if}

					{#if obsStreaming}
						<Badge variant="success">OBS Streaming</Badge>
					{/if}
				</div>

				<Separator />

				<!-- Control Buttons -->
				<div class="flex flex-col sm:flex-row gap-3">
					{#if obsStreaming}
						<Button
								buttonVariant="destructive"
								onclick={onStopObsStream}
								className="flex-1"
						>
							<StopCircle class="h-4 w-4 mr-2" />
							Stop OBS Stream
						</Button>
					{:else}
						<Button
								onclick={onStartObsStream}
								className="flex-1"
						>
							<PlayCircle class="h-4 w-4 mr-2" />
							Start OBS Stream
						</Button>
					{/if}

					{#if youtubeOnAir}
						<Button
								buttonVariant="destructive"
								onclick={onYoutubeStopLive}
								disabled={!obsStreaming}
								className="flex-1"
						>
							<StopCircle class="h-4 w-4 mr-2" />
							End YouTube Stream
						</Button>
					{:else}
						<Button
								onclick={onYoutubeGoLive}
								disabled={!obsStreaming}
								className="flex-1"
						>
							<PlayCircle class="h-4 w-4 mr-2" />
							Go Live on YouTube
						</Button>
					{/if}
				</div>

				{#if !obsStreaming}
					<p class="text-sm text-muted-foreground">
						Start OBS streaming before going live on YouTube
					</p>
				{/if}
			</div>
		{:else}
			<p class="text-sm text-muted-foreground">No YouTube event scheduled</p>
		{/if}
	</svelte:fragment>
</Card>

<!-- Section 5: Sermon Texts & OBS Control -->
<div class="grid gap-6 lg:grid-cols-2">
	<!-- Sermon Texts Card -->
	<Card>
		<svelte:fragment slot="title">Sermon Texts</svelte:fragment>
		<svelte:fragment slot="description">Manage your Bible readings</svelte:fragment>
		<svelte:fragment slot="content">
			<div class="space-y-4">
				<!-- Textus Section -->
				<div class="space-y-2">
					<div class="flex items-center justify-between">
						<Label>Textus</Label>
						<Button
								buttonVariant="ghost"
								buttonSize="sm"
								href={'/bible'}
						>
							<Edit2 class="h-3 w-3 mr-1" />
							Edit
						</Button>
					</div>
					<Input
							type="text"
							bind:value={textus}
							oninput={() => onTextusChange(textus)}
							placeholder="e.g., John 3:16-21"
					/>
					<Button
							buttonVariant="outline"
							buttonSize="sm"
							onclick={handleGenerateTextusPPT}
							className="w-full"
					>
						<FileText class="h-4 w-4 mr-2" />
						Generate Textus PPT
					</Button>
				</div>

				<Separator />

				<!-- Leckio Section -->
				<div class="space-y-2">
					<div class="flex items-center justify-between">
						<Label>Leckio</Label>
						<Button
								buttonVariant="ghost"
								buttonSize="sm"
								href={'/bible'}
						>
							<Edit2 class="h-3 w-3 mr-1" />
							Edit
						</Button>
					</div>
					<Input
							type="text"
							bind:value={leckio}
							oninput={() => onLeckioChange(leckio)}
							placeholder="e.g., Romans 8:28-39"
					/>
					<Button
							buttonVariant="outline"
							buttonSize="sm"
							onclick={handleGenerateLeckioPPT}
							className="w-full"
					>
						<FileText class="h-4 w-4 mr-2" />
						Generate Leckio PPT
					</Button>
				</div>

				<Button
						buttonVariant="default"
						href={'/bible'}
						className="w-full"
				>
					<Book class="h-4 w-4 mr-2" />
					Edit Bible Texts
				</Button>
			</div>
		</svelte:fragment>
	</Card>

	<!-- OBS Control Card -->
	<Card>
		<svelte:fragment slot="title">OBS Control</svelte:fragment>
		<svelte:fragment slot="description">Update your sermon title in OBS</svelte:fragment>
		<svelte:fragment slot="content">
			<div class="space-y-4">
				<div class="space-y-2">
					<Label>Sermon Title</Label>
					<Input
							type="text"
							bind:value={obsTitle}
							placeholder="Enter sermon title"
					/>
				</div>
				<Button
						onclick={handleUpdateOBS}
						className="w-full"
				>
					<RefreshCw class="h-4 w-4 mr-2" />
					Update OBS Title
				</Button>
			</div>
		</svelte:fragment>
	</Card>
</div>
