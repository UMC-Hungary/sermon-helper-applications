<script lang="ts">
	import Card from "$lib/components/ui/card.svelte";
	import Button from "$lib/components/ui/button.svelte";
	import Input from "$lib/components/ui/input.svelte";
	import Label from "$lib/components/ui/label.svelte";
	import Badge from "$lib/components/ui/badge.svelte";
	import Separator from "$lib/components/ui/separator.svelte";
	import { toast } from "$lib/utils/toast";
	import { _ } from 'svelte-i18n';
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
			title: $_('toasts.obsUpdated.title'),
			description: $_('toasts.obsUpdated.description'),
			variant: "success"
		});
	};

	const handleGenerateTextusPPT = () => {
		toast({
			title: $_('toasts.pptGenerated.title'),
			description: $_('toasts.pptGenerated.descriptionTextus'),
			variant: "success"
		});
	};

	const handleGenerateLeckioPPT = () => {
		toast({
			title: $_('toasts.pptGenerated.title'),
			description: $_('toasts.pptGenerated.descriptionLeckio'),
			variant: "success"
		});
	};
</script>

<!-- Section 2: Dashboard Header -->
<div class="mt-12 lg:mt-0">
	<h2 class="text-3xl font-bold tracking-tight">{$_('dashboard.title')}</h2>
	<p class="text-muted-foreground">{$_('dashboard.subtitle')}</p>
</div>

<!-- Section 3: Quick Access Cards -->
<div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
	<!-- Bible Editor Card -->
	<Card clickable href={'/bible'}>
		<svelte:fragment slot="title">
			<Book class="h-5 w-5" />
			{$_('dashboard.cards.bibleEditor.title')}
		</svelte:fragment>
		<svelte:fragment slot="description">{$_('dashboard.cards.bibleEditor.description')}</svelte:fragment>
		<svelte:fragment slot="content">
			<Badge>{$_('dashboard.cards.bibleEditor.badge')}</Badge>
		</svelte:fragment>
	</Card>

	<!-- Schedule Event Card -->
	<Card clickable href={'/youtube-schedule'}>
		<svelte:fragment slot="title">
			<Youtube class="h-5 w-5" />
			{$_('dashboard.cards.scheduleEvent.title')}
		</svelte:fragment>
		<svelte:fragment slot="description">{$_('dashboard.cards.scheduleEvent.description')}</svelte:fragment>
		<svelte:fragment slot="content">
			<Badge variant="outline">{$_('dashboard.cards.scheduleEvent.badge')}</Badge>
		</svelte:fragment>
	</Card>

	<!-- View Events Card -->
	<Card clickable href={'/youtube-events'}>
		<svelte:fragment slot="title">
			<FileText class="h-5 w-5" />
			{$_('dashboard.cards.viewEvents.title')}
		</svelte:fragment>
		<svelte:fragment slot="description">{$_('dashboard.cards.viewEvents.description')}</svelte:fragment>
		<svelte:fragment slot="content">
			<Badge variant="secondary">{$_('dashboard.cards.viewEvents.badge')}</Badge>
		</svelte:fragment>
	</Card>
</div>

<!-- Section 4: Current Sermon Status Card -->
<Card>
	<svelte:fragment slot="title">{$_('dashboard.sermonStatus.title')}</svelte:fragment>
	<svelte:fragment slot="description">{$_('dashboard.sermonStatus.description')}</svelte:fragment>
	<svelte:fragment slot="content">
		{#if currentSermon.youtubeScheduled}
			<div class="space-y-4">
				<!-- YouTube Title -->
				<div class="flex items-center justify-between">
					<div>
						<p class="text-sm font-medium">{$_('dashboard.sermonStatus.youtubeEvent')}</p>
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
							{$_('dashboard.sermonStatus.liveNow')}
						</Badge>
					{:else}
						<Badge variant="secondary">{$_('dashboard.sermonStatus.scheduled')}</Badge>
					{/if}

					{#if obsStreaming}
						<Badge variant="success">{$_('dashboard.sermonStatus.obsStreaming')}</Badge>
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
							{$_('dashboard.sermonStatus.stopObs')}
						</Button>
					{:else}
						<Button
								onclick={onStartObsStream}
								className="flex-1"
						>
							<PlayCircle class="h-4 w-4 mr-2" />
							{$_('dashboard.sermonStatus.startObs')}
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
							{$_('dashboard.sermonStatus.endStream')}
						</Button>
					{:else}
						<Button
								onclick={onYoutubeGoLive}
								disabled={!obsStreaming}
								className="flex-1"
						>
							<PlayCircle class="h-4 w-4 mr-2" />
							{$_('dashboard.sermonStatus.goLive')}
						</Button>
					{/if}
				</div>

				{#if !obsStreaming}
					<p class="text-sm text-muted-foreground">
						{$_('dashboard.sermonStatus.startObsFirst')}
					</p>
				{/if}
			</div>
		{:else}
			<p class="text-sm text-muted-foreground">{$_('dashboard.sermonStatus.noEventScheduled')}</p>
		{/if}
	</svelte:fragment>
</Card>

<!-- Section 5: Sermon Texts & OBS Control -->
<div class="grid gap-6 lg:grid-cols-2">
	<!-- Sermon Texts Card -->
	<Card>
		<svelte:fragment slot="title">{$_('dashboard.sermonTexts.title')}</svelte:fragment>
		<svelte:fragment slot="description">{$_('dashboard.sermonTexts.description')}</svelte:fragment>
		<svelte:fragment slot="content">
			<div class="space-y-4">
				<!-- Textus Section -->
				<div class="space-y-2">
					<div class="flex items-center justify-between">
						<Label>{$_('dashboard.sermonTexts.textus')}</Label>
						<Button
								buttonVariant="ghost"
								buttonSize="sm"
								href={'/bible'}
						>
							<Edit2 class="h-3 w-3 mr-1" />
							{$_('dashboard.sermonTexts.edit')}
						</Button>
					</div>
					<Input
							type="text"
							bind:value={textus}
							oninput={() => onTextusChange(textus)}
							placeholder={$_('dashboard.sermonTexts.textusPlaceholder')}
					/>
					<Button
							buttonVariant="outline"
							buttonSize="sm"
							onclick={handleGenerateTextusPPT}
							className="w-full"
					>
						<FileText class="h-4 w-4 mr-2" />
						{$_('dashboard.sermonTexts.generateTextusPpt')}
					</Button>
				</div>

				<Separator />

				<!-- Leckio Section -->
				<div class="space-y-2">
					<div class="flex items-center justify-between">
						<Label>{$_('dashboard.sermonTexts.leckio')}</Label>
						<Button
								buttonVariant="ghost"
								buttonSize="sm"
								href={'/bible'}
						>
							<Edit2 class="h-3 w-3 mr-1" />
							{$_('dashboard.sermonTexts.edit')}
						</Button>
					</div>
					<Input
							type="text"
							bind:value={leckio}
							oninput={() => onLeckioChange(leckio)}
							placeholder={$_('dashboard.sermonTexts.leckioPlaceholder')}
					/>
					<Button
							buttonVariant="outline"
							buttonSize="sm"
							onclick={handleGenerateLeckioPPT}
							className="w-full"
					>
						<FileText class="h-4 w-4 mr-2" />
						{$_('dashboard.sermonTexts.generateLeckioPpt')}
					</Button>
				</div>

				<Button
						buttonVariant="default"
						href={'/bible'}
						className="w-full"
				>
					<Book class="h-4 w-4 mr-2" />
					{$_('dashboard.sermonTexts.editBibleTexts')}
				</Button>
			</div>
		</svelte:fragment>
	</Card>

	<!-- OBS Control Card -->
	<Card>
		<svelte:fragment slot="title">{$_('dashboard.obsControl.title')}</svelte:fragment>
		<svelte:fragment slot="description">{$_('dashboard.obsControl.description')}</svelte:fragment>
		<svelte:fragment slot="content">
			<div class="space-y-4">
				<div class="space-y-2">
					<Label>{$_('dashboard.obsControl.sermonTitle')}</Label>
					<Input
							type="text"
							bind:value={obsTitle}
							placeholder={$_('dashboard.obsControl.sermonTitlePlaceholder')}
					/>
				</div>
				<Button
						onclick={handleUpdateOBS}
						className="w-full"
				>
					<RefreshCw class="h-4 w-4 mr-2" />
					{$_('dashboard.obsControl.updateObsTitle')}
				</Button>
			</div>
		</svelte:fragment>
	</Card>
</div>
