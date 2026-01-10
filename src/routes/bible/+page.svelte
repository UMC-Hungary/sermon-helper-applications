<script lang="ts">
	import { _ } from 'svelte-i18n';
	import Card from "$lib/components/ui/card.svelte";
	import Button from "$lib/components/ui/button.svelte";
	import Input from "$lib/components/ui/input.svelte";
	import Label from "$lib/components/ui/label.svelte";
	import Textarea from "$lib/components/ui/textarea.svelte";
	import Badge from "$lib/components/ui/badge.svelte";
	import Tabs from "$lib/components/ui/tabs.svelte";
	import TabsList from "$lib/components/ui/tabs-list.svelte";
	import TabsTrigger from "$lib/components/ui/tabs-trigger.svelte";
	import TabsContent from "$lib/components/ui/tabs-content.svelte";
	import { toast } from "$lib/utils/toast";
	import { Search, Save, Edit2, Check } from "lucide-svelte";

	interface Verse {
		number: number;
		text: string;
		editing: boolean;
	}

	// State
	let textusQuery = "John 3:16-21";
	let leckioQuery = "Romans 8:28-39";

	let textusVerses: Verse[] = [
		{
			number: 16,
			text: "For God so loved the world, that he gave his only Son, that whoever believes in him should not perish but have eternal life.",
			editing: false,
		},
		{
			number: 17,
			text: "For God did not send his Son into the world to condemn the world, but in order that the world might be saved through him.",
			editing: false,
		},
		{
			number: 18,
			text: "Whoever believes in him is not condemned, but whoever does not believe is condemned already, because he has not believed in the name of the only Son of God.",
			editing: false,
		},
		{
			number: 19,
			text: "And this is the judgment: the light has come into the world, and people loved the darkness rather than the light because their works were evil.",
			editing: false,
		},
		{
			number: 20,
			text: "For everyone who does wicked things hates the light and does not come to the light, lest his works should be exposed.",
			editing: false,
		},
		{
			number: 21,
			text: "But whoever does what is true comes to the light, so that it may be clearly seen that his works have been carried out in God.",
			editing: false,
		},
	];

	let leckioVerses: Verse[] = [
		{
			number: 28,
			text: "And we know that for those who love God all things work together for good, for those who are called according to his purpose.",
			editing: false,
		},
		{
			number: 29,
			text: "For those whom he foreknew he also predestined to be conformed to the image of his Son, in order that he might be the firstborn among many brothers.",
			editing: false,
		},
		{
			number: 30,
			text: "And those whom he predestined he also called, and those whom he called he also justified, and those whom he justified he also glorified.",
			editing: false,
		},
	];

	// Handlers
	function handleSearch(type: "textus" | "leckio") {
		const reference = type === "textus" ? textusQuery : leckioQuery;
		toast({
			title: $_('bible.toasts.fetching.title'),
			description: $_('bible.toasts.fetching.description', { values: { reference } }),
			variant: "info"
		});
	}

	function handleVerseEdit(type: "textus" | "leckio", verseNumber: number, newText: string) {
		if (type === "textus") {
			textusVerses = textusVerses.map((v) =>
				v.number === verseNumber ? { ...v, text: newText } : v
			);
		} else {
			leckioVerses = leckioVerses.map((v) =>
				v.number === verseNumber ? { ...v, text: newText } : v
			);
		}
	}

	function toggleEditing(type: "textus" | "leckio", verseNumber: number) {
		if (type === "textus") {
			textusVerses = textusVerses.map((v) =>
				v.number === verseNumber ? { ...v, editing: !v.editing } : v
			);
		} else {
			leckioVerses = leckioVerses.map((v) =>
				v.number === verseNumber ? { ...v, editing: !v.editing } : v
			);
		}
	}

	function handleSave() {
		toast({
			title: $_('bible.toasts.saved.title'),
			description: $_('bible.toasts.saved.description'),
			variant: "success"
		});
	}
</script>

<!-- Page Header -->
<div class="mt-12 lg:mt-0">
	<h2 class="text-3xl font-bold tracking-tight">{$_('bible.title')}</h2>
	<p class="text-muted-foreground">{$_('bible.subtitle')}</p>
</div>

<!-- Tabs -->
<Tabs defaultValue="textus">
	<TabsList className="grid w-full max-w-md grid-cols-2">
		<TabsTrigger value="textus">{$_('bible.tabs.textus')}</TabsTrigger>
		<TabsTrigger value="leckio">{$_('bible.tabs.leckio')}</TabsTrigger>
	</TabsList>

	<!-- Textus Tab -->
	<TabsContent value="textus">
		<!-- Search Card -->
		<Card>
			<svelte:fragment slot="title">{$_('bible.search.titleTextus')}</svelte:fragment>
			<svelte:fragment slot="description">{$_('bible.search.description')}</svelte:fragment>
			<svelte:fragment slot="content">
				<div class="flex gap-2">
					<div class="flex-1 space-y-2">
						<Label for="textus-search">{$_('bible.search.label')}</Label>
						<Input
							id="textus-search"
							bind:value={textusQuery}
							placeholder={$_('bible.search.placeholderTextus')}
						/>
					</div>
					<Button className="mt-auto" onclick={() => handleSearch("textus")}>
						<Search class="mr-2 h-4 w-4" />
						{$_('bible.search.fetch')}
					</Button>
				</div>
			</svelte:fragment>
		</Card>

		<!-- Verses Card -->
		<Card>
			<svelte:fragment slot="header">
				<div class="flex items-center justify-between w-full">
					<div>
						<div data-slot="card-title" class="leading-none font-semibold">{$_('bible.verses.title')} - {textusQuery}</div>
						<div data-slot="card-description" class="text-muted-foreground text-sm">{$_('bible.verses.description')}</div>
					</div>
					<Button buttonSize="sm" onclick={handleSave}>
						<Save class="mr-2 h-4 w-4" />
						{$_('bible.verses.saveAll')}
					</Button>
				</div>
			</svelte:fragment>
			<svelte:fragment slot="content">
				<div class="space-y-1">
					{#each textusVerses as verse, index (verse.number)}
						<div
							class="flex gap-3 items-start p-2 hover:bg-accent/30 transition-colors {index !== textusVerses.length - 1 ? 'border-b' : ''}"
						>
							<Badge variant="outline" className="shrink-0 mt-0.5 text-xs">
								{verse.number}
							</Badge>
							{#if verse.editing}
								<Textarea
									bind:value={verse.text}
									className="flex-1 min-h-[60px] text-sm"
								/>
							{:else}
								<p class="flex-1 text-sm leading-relaxed">{verse.text}</p>
							{/if}
							<Button
								buttonVariant="ghost"
								buttonSize="icon"
								className="shrink-0 h-8 w-8"
								onclick={() => toggleEditing("textus", verse.number)}
							>
								{#if verse.editing}
									<Check class="h-4 w-4" />
								{:else}
									<Edit2 class="h-4 w-4" />
								{/if}
							</Button>
						</div>
					{/each}
				</div>
			</svelte:fragment>
		</Card>
	</TabsContent>

	<!-- Leckio Tab -->
	<TabsContent value="leckio">
		<!-- Search Card -->
		<Card>
			<svelte:fragment slot="title">{$_('bible.search.titleLeckio')}</svelte:fragment>
			<svelte:fragment slot="description">{$_('bible.search.description')}</svelte:fragment>
			<svelte:fragment slot="content">
				<div class="flex gap-2">
					<div class="flex-1 space-y-2">
						<Label for="leckio-search">{$_('bible.search.label')}</Label>
						<Input
							id="leckio-search"
							bind:value={leckioQuery}
							placeholder={$_('bible.search.placeholderLeckio')}
						/>
					</div>
					<Button className="mt-auto" onclick={() => handleSearch("leckio")}>
						<Search class="mr-2 h-4 w-4" />
						{$_('bible.search.fetch')}
					</Button>
				</div>
			</svelte:fragment>
		</Card>

		<!-- Verses Card -->
		<Card>
			<svelte:fragment slot="header">
				<div class="flex items-center justify-between w-full">
					<div>
						<div data-slot="card-title" class="leading-none font-semibold">{$_('bible.verses.title')} - {leckioQuery}</div>
						<div data-slot="card-description" class="text-muted-foreground text-sm">{$_('bible.verses.description')}</div>
					</div>
					<Button buttonSize="sm" onclick={handleSave}>
						<Save class="mr-2 h-4 w-4" />
						{$_('bible.verses.saveAll')}
					</Button>
				</div>
			</svelte:fragment>
			<svelte:fragment slot="content">
				<div class="space-y-1">
					{#each leckioVerses as verse, index (verse.number)}
						<div
							class="flex gap-3 items-start p-2 hover:bg-accent/30 transition-colors {index !== leckioVerses.length - 1 ? 'border-b' : ''}"
						>
							<Badge variant="outline" className="shrink-0 mt-0.5 text-xs">
								{verse.number}
							</Badge>
							{#if verse.editing}
								<Textarea
									bind:value={verse.text}
									className="flex-1 min-h-[60px] text-sm"
								/>
							{:else}
								<p class="flex-1 text-sm leading-relaxed">{verse.text}</p>
							{/if}
							<Button
								buttonVariant="ghost"
								buttonSize="icon"
								className="shrink-0 h-8 w-8"
								onclick={() => toggleEditing("leckio", verse.number)}
							>
								{#if verse.editing}
									<Check class="h-4 w-4" />
								{:else}
									<Edit2 class="h-4 w-4" />
								{/if}
							</Button>
						</div>
					{/each}
				</div>
			</svelte:fragment>
		</Card>
	</TabsContent>
</Tabs>
