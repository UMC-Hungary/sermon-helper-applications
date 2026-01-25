<script lang="ts">
	import Button from '$lib/components/ui/button.svelte';
	import Card from '$lib/components/ui/card.svelte';
	import Input from '$lib/components/ui/input.svelte';
	import { toast } from '$lib/utils/toast';
	import {
		rfIrStore,
		rfIrSettings,
		rfIrCommands,
		rfIrDevices,
		commandsByCategory,
		commandCategories
	} from '$lib/stores/rf-ir-store';
	import { broadlinkService } from '$lib/utils/broadlink-service';
	import type { RfIrCommand } from '$lib/types/rf-ir';
	import {
		Radio,
		Play,
		Search,
		Loader2,
		Settings,
		Power,
		Monitor,
		Thermometer,
		Lightbulb,
		Volume2,
		MoreHorizontal
	} from 'lucide-svelte';
	import { onMount } from 'svelte';

	let isLoading = true;
	let searchQuery = '';
	let selectedCategory = 'all';
	let executingCommand: string | null = null;

	// Category icons
	const categoryIcons: Record<string, typeof Power> = {
		projector: Monitor,
		screen: Power,
		hvac: Thermometer,
		lighting: Lightbulb,
		audio: Volume2,
		other: MoreHorizontal
	};

	onMount(async () => {
		try {
			await rfIrStore.load();
		} catch (error) {
			console.error('Failed to load RF/IR settings:', error);
		} finally {
			isLoading = false;
		}
	});

	$: filteredCommands = $rfIrCommands.filter((cmd) => {
		const matchesSearch =
			searchQuery === '' ||
			cmd.name.toLowerCase().includes(searchQuery.toLowerCase());
		const matchesCategory = selectedCategory === 'all' || cmd.category === selectedCategory;
		return matchesSearch && matchesCategory;
	});

	$: groupedCommands = (() => {
		const grouped = new Map<string, RfIrCommand[]>();
		for (const cmd of filteredCommands) {
			const category = cmd.category || 'other';
			if (!grouped.has(category)) grouped.set(category, []);
			grouped.get(category)!.push(cmd);
		}
		return grouped;
	})();

	async function handleExecute(command: RfIrCommand) {
		executingCommand = command.id;
		try {
			await broadlinkService.executeCommand(command);
			toast({
				title: 'Sent',
				description: command.name,
				variant: 'success'
			});
		} catch (error) {
			toast({
				title: 'Failed',
				description: error instanceof Error ? error.message : 'Failed to send command',
				variant: 'error'
			});
		} finally {
			executingCommand = null;
		}
	}

	function getCategoryIcon(category: string) {
		return categoryIcons[category] || MoreHorizontal;
	}
</script>

<svelte:head>
	<title>Remote Control - Sermon Helper</title>
</svelte:head>

<div class="mt-12 lg:mt-0">
	<div class="flex items-center justify-between">
		<div>
			<h2 class="text-3xl font-bold tracking-tight flex items-center gap-3">
				<Radio class="h-8 w-8" />
				Remote Control
			</h2>
			<p class="text-muted-foreground">
				Control projectors, screens, and other devices via IR/RF
			</p>
		</div>
		<a href="/obs-config" class="text-muted-foreground hover:text-foreground transition-colors">
			<Settings class="h-5 w-5" />
		</a>
	</div>
</div>

{#if isLoading}
	<div class="flex items-center justify-center py-12">
		<Loader2 class="h-8 w-8 animate-spin text-muted-foreground" />
	</div>
{:else if !$rfIrSettings.enabled}
	<Card class="mt-6">
		<svelte:fragment slot="content">
			<div class="py-12 text-center">
				<Radio class="h-12 w-12 mx-auto text-muted-foreground mb-4" />
				<h3 class="text-lg font-medium mb-2">RF/IR Control Disabled</h3>
				<p class="text-muted-foreground mb-4">
					Enable RF/IR remote control in settings to use this feature.
				</p>
				<Button href="/obs-config">
					<Settings class="mr-2 h-4 w-4" />
					Go to Settings
				</Button>
			</div>
		</svelte:fragment>
	</Card>
{:else if $rfIrDevices.length === 0}
	<Card class="mt-6">
		<svelte:fragment slot="content">
			<div class="py-12 text-center">
				<Radio class="h-12 w-12 mx-auto text-muted-foreground mb-4" />
				<h3 class="text-lg font-medium mb-2">No Devices Configured</h3>
				<p class="text-muted-foreground mb-4">
					Add a Broadlink device in settings to get started.
				</p>
				<Button href="/obs-config">
					<Settings class="mr-2 h-4 w-4" />
					Configure Devices
				</Button>
			</div>
		</svelte:fragment>
	</Card>
{:else if $rfIrCommands.length === 0}
	<Card class="mt-6">
		<svelte:fragment slot="content">
			<div class="py-12 text-center">
				<Power class="h-12 w-12 mx-auto text-muted-foreground mb-4" />
				<h3 class="text-lg font-medium mb-2">No Commands Configured</h3>
				<p class="text-muted-foreground mb-4">
					Learn or add RF/IR commands in settings.
				</p>
				<Button href="/obs-config">
					<Settings class="mr-2 h-4 w-4" />
					Add Commands
				</Button>
			</div>
		</svelte:fragment>
	</Card>
{:else}
	<!-- Search and Filter -->
	<div class="mt-6 flex gap-3">
		<div class="relative flex-1">
			<Search class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
			<Input
				bind:value={searchQuery}
				placeholder="Search commands..."
				class="pl-10"
			/>
		</div>
		<select
			bind:value={selectedCategory}
			class="h-10 rounded-md border border-input bg-background px-3 text-sm min-w-[140px]"
		>
			<option value="all">All Categories</option>
			{#each $commandCategories as category}
				<option value={category}>
					{category.charAt(0).toUpperCase() + category.slice(1)}
				</option>
			{/each}
		</select>
	</div>

	<!-- Command Grid -->
	<div class="mt-6 space-y-6">
		{#each [...groupedCommands.entries()] as [category, commands]}
			<div>
				<h3 class="text-sm font-medium text-muted-foreground mb-3 flex items-center gap-2">
					<svelte:component this={getCategoryIcon(category)} class="h-4 w-4" />
					{category.charAt(0).toUpperCase() + category.slice(1)}
				</h3>
				<div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-3">
					{#each commands as command (command.id)}
						<button
							class="relative flex flex-col items-center justify-center p-6 rounded-xl border bg-card hover:bg-accent hover:border-accent-foreground/20 transition-all active:scale-95 disabled:opacity-50 disabled:cursor-not-allowed"
							onclick={() => handleExecute(command)}
							disabled={executingCommand === command.id}
						>
							{#if executingCommand === command.id}
								<Loader2 class="h-8 w-8 animate-spin text-primary mb-2" />
							{:else}
								<Play class="h-8 w-8 text-primary mb-2" />
							{/if}
							<span class="font-medium text-sm text-center">{command.name}</span>
							<span class="text-xs text-muted-foreground mt-1 uppercase">
								{command.type}
							</span>
						</button>
					{/each}
				</div>
			</div>
		{/each}
	</div>

	{#if filteredCommands.length === 0}
		<div class="py-12 text-center text-muted-foreground">
			No commands match your search.
		</div>
	{/if}
{/if}
