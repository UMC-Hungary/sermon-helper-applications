<script lang="ts">
	import Button from '$lib/components/ui/button.svelte';
	import Input from '$lib/components/ui/input.svelte';
	import { toast } from '$lib/utils/toast';
	import {
		rfIrStore,
		rfIrCommands,
		rfIrDevices,
		commandsByCategory,
		commandCategories
	} from '$lib/stores/rf-ir-store';
	import { broadlinkService } from '$lib/utils/broadlink-service';
	import type { RfIrCommand } from '$lib/types/rf-ir';
	import {
		Play,
		Pencil,
		Trash2,
		Copy,
		Check,
		Loader2,
		Search,
		Filter,
		X
	} from 'lucide-svelte';

	let searchQuery = '';
	let selectedCategory = 'all';
	let executingCommand: string | null = null;
	let copiedSlug: string | null = null;
	let editingCommand: RfIrCommand | null = null;

	// Edit form state
	let editName = '';
	let editCategory = '';
	let editCode = '';

	$: filteredCommands = $rfIrCommands.filter((cmd) => {
		const matchesSearch =
			searchQuery === '' ||
			cmd.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
			cmd.slug.toLowerCase().includes(searchQuery.toLowerCase());
		const matchesCategory = selectedCategory === 'all' || cmd.category === selectedCategory;
		return matchesSearch && matchesCategory;
	});

	$: categoriesWithCounts = ['all', ...$commandCategories].map((cat) => ({
		value: cat,
		label: cat === 'all' ? 'All' : cat.charAt(0).toUpperCase() + cat.slice(1),
		count: cat === 'all' ? $rfIrCommands.length : $rfIrCommands.filter((c) => c.category === cat).length
	}));

	function getDeviceName(deviceId: string): string {
		const device = $rfIrDevices.find((d) => d.id === deviceId);
		return device?.name || 'Unknown';
	}

	async function handleExecute(command: RfIrCommand) {
		executingCommand = command.id;
		try {
			await broadlinkService.executeCommand(command);
			toast({
				title: 'Command Sent',
				description: `Executed: ${command.name}`,
				variant: 'success'
			});
		} catch (error) {
			toast({
				title: 'Command Failed',
				description: error instanceof Error ? error.message : 'Failed to send command',
				variant: 'error'
			});
		} finally {
			executingCommand = null;
		}
	}

	async function handleCopySlug(slug: string) {
		await navigator.clipboard.writeText(slug);
		copiedSlug = slug;
		setTimeout(() => {
			copiedSlug = null;
		}, 2000);
	}

	function handleEdit(command: RfIrCommand) {
		editingCommand = command;
		editName = command.name;
		editCategory = command.category;
		editCode = command.code;
	}

	function handleCancelEdit() {
		editingCommand = null;
		editName = '';
		editCategory = '';
		editCode = '';
	}

	async function handleSaveEdit() {
		if (!editingCommand) return;

		if (!editName.trim()) {
			toast({
				title: 'Invalid Name',
				description: 'Command name is required',
				variant: 'warning'
			});
			return;
		}

		try {
			await rfIrStore.updateCommand(editingCommand.id, {
				name: editName.trim(),
				category: editCategory.trim() || 'other',
				code: editCode.trim()
			});

			toast({
				title: 'Command Updated',
				description: 'Command has been updated',
				variant: 'success'
			});

			handleCancelEdit();
		} catch (error) {
			toast({
				title: 'Update Failed',
				description: error instanceof Error ? error.message : 'Failed to update command',
				variant: 'error'
			});
		}
	}

	async function handleDelete(command: RfIrCommand) {
		if (!confirm(`Delete command "${command.name}"?`)) {
			return;
		}

		try {
			await rfIrStore.removeCommand(command.id);
			toast({
				title: 'Command Deleted',
				description: `${command.name} has been deleted`,
				variant: 'success'
			});
		} catch (error) {
			toast({
				title: 'Delete Failed',
				description: error instanceof Error ? error.message : 'Failed to delete command',
				variant: 'error'
			});
		}
	}
</script>

<div class="space-y-3">
	<!-- Search and Filter -->
	{#if $rfIrCommands.length > 0}
		<div class="flex gap-2">
			<div class="relative flex-1">
				<Search class="absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-muted-foreground" />
				<Input
					bind:value={searchQuery}
					placeholder="Search commands..."
					class="h-8 pl-8 text-sm"
				/>
				{#if searchQuery}
					<button
						class="absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
						onclick={() => (searchQuery = '')}
					>
						<X class="h-3.5 w-3.5" />
					</button>
				{/if}
			</div>
			<select
				bind:value={selectedCategory}
				class="h-8 rounded-md border border-input bg-background px-2 text-sm"
			>
				{#each categoriesWithCounts as cat}
					<option value={cat.value}>{cat.label} ({cat.count})</option>
				{/each}
			</select>
		</div>
	{/if}

	<!-- Command List -->
	{#if $rfIrCommands.length === 0}
		<p class="text-sm text-muted-foreground text-center py-4">
			No commands configured. Learn a new command or add one manually.
		</p>
	{:else if filteredCommands.length === 0}
		<p class="text-sm text-muted-foreground text-center py-4">
			No commands match your search.
		</p>
	{:else}
		<div class="space-y-2 max-h-64 overflow-y-auto">
			{#each filteredCommands as command (command.id)}
				{#if editingCommand?.id === command.id}
					<!-- Edit Mode -->
					<div class="rounded-lg bg-muted p-3 space-y-3">
						<div class="grid grid-cols-2 gap-2">
							<Input
								bind:value={editName}
								placeholder="Command name"
								class="h-7 text-sm"
							/>
							<Input
								bind:value={editCategory}
								placeholder="Category"
								class="h-7 text-sm"
							/>
						</div>
						<Input
							bind:value={editCode}
							placeholder="Hex code"
							class="h-7 text-sm font-mono text-xs"
						/>
						<div class="flex gap-2">
							<Button
								buttonVariant="outline"
								buttonSize="sm"
								onclick={handleCancelEdit}
								className="flex-1 h-7"
							>
								Cancel
							</Button>
							<Button buttonSize="sm" onclick={handleSaveEdit} className="flex-1 h-7">
								Save
							</Button>
						</div>
					</div>
				{:else}
					<!-- View Mode -->
					<div
						class="flex items-center justify-between p-2.5 rounded-lg bg-muted/50 hover:bg-muted transition-colors group"
					>
						<div class="flex items-center gap-2 min-w-0 flex-1">
							<Button
								buttonVariant="ghost"
								buttonSize="sm"
								onclick={() => handleExecute(command)}
								disabled={executingCommand === command.id}
								className="h-7 w-7 p-0 flex-shrink-0"
							>
								{#if executingCommand === command.id}
									<Loader2 class="h-3.5 w-3.5 animate-spin" />
								{:else}
									<Play class="h-3.5 w-3.5" />
								{/if}
							</Button>
							<div class="min-w-0 flex-1">
								<div class="flex items-center gap-2">
									<span class="font-medium text-sm truncate">{command.name}</span>
									<span
										class="text-[10px] px-1.5 py-0.5 rounded bg-muted text-muted-foreground uppercase"
									>
										{command.type}
									</span>
								</div>
								<div class="flex items-center gap-2 text-xs text-muted-foreground">
									<span class="capitalize">{command.category}</span>
									<span>-</span>
									<button
										class="flex items-center gap-1 hover:text-foreground font-mono"
										onclick={() => handleCopySlug(command.slug)}
										title="Copy slug for API"
									>
										<code>{command.slug}</code>
										{#if copiedSlug === command.slug}
											<Check class="h-2.5 w-2.5 text-green-600" />
										{:else}
											<Copy class="h-2.5 w-2.5 opacity-0 group-hover:opacity-100" />
										{/if}
									</button>
								</div>
							</div>
						</div>
						<div class="flex items-center gap-1 flex-shrink-0 opacity-0 group-hover:opacity-100 transition-opacity">
							<Button
								buttonVariant="ghost"
								buttonSize="sm"
								onclick={() => handleEdit(command)}
								className="h-6 w-6 p-0"
							>
								<Pencil class="h-3 w-3" />
							</Button>
							<Button
								buttonVariant="ghost"
								buttonSize="sm"
								onclick={() => handleDelete(command)}
								className="h-6 w-6 p-0 text-red-600 hover:text-red-700 hover:bg-red-100"
							>
								<Trash2 class="h-3 w-3" />
							</Button>
						</div>
					</div>
				{/if}
			{/each}
		</div>
	{/if}
</div>
