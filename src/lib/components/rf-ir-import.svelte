<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import Button from '$lib/components/ui/button.svelte';
	import Input from '$lib/components/ui/input.svelte';
	import Label from '$lib/components/ui/label.svelte';
	import { toast } from '$lib/utils/toast';
	import { rfIrStore, rfIrDevices, defaultDevice } from '$lib/stores/rf-ir-store';
	import { COMMAND_CATEGORIES } from '$lib/types/rf-ir';
	import { Upload, FileText, Check, AlertCircle, Loader2 } from 'lucide-svelte';

	export let open = false;

	const dispatch = createEventDispatcher<{ imported: { count: number } }>();

	let dialogElement: HTMLDialogElement;
	let isImporting = false;

	// Predefined commands from sermon-helper-service
	const predefinedCommands = [
		{ name: 'Projector On', filename: 'NEC.on', category: 'projector', type: 'ir' as const },
		{ name: 'Projector Off', filename: 'NEC.off', category: 'projector', type: 'ir' as const },
		{ name: 'Projector Menu', filename: 'NEC.menu', category: 'projector', type: 'ir' as const },
		{ name: 'Projector OK', filename: 'NEC.ok', category: 'projector', type: 'ir' as const },
		{ name: 'Projector Up', filename: 'NEC.up', category: 'projector', type: 'ir' as const },
		{ name: 'Projector Back', filename: 'NEC.back', category: 'projector', type: 'ir' as const },
		{ name: 'Projector Left', filename: 'NEC.left', category: 'projector', type: 'ir' as const },
		{ name: 'Projector Right', filename: 'NEC.right', category: 'projector', type: 'ir' as const },
		{ name: 'Projector Power', filename: 'NEC.power', category: 'projector', type: 'ir' as const },
		{ name: 'Picture Mute', filename: 'NEC.picture_mute', category: 'projector', type: 'ir' as const },
		{ name: 'Screen Up', filename: 'ROLL.up', category: 'screen', type: 'rf' as const },
		{ name: 'Screen Down', filename: 'ROLL.down', category: 'screen', type: 'rf' as const },
		{ name: 'Screen Stop', filename: 'ROLL.stop', category: 'screen', type: 'rf' as const },
		{ name: 'AC Temp Up', filename: 'CASCADE.temp_up', category: 'hvac', type: 'ir' as const },
	];

	// Hardcoded codes from sermon-helper-service (these are the actual learned codes)
	const codeFiles: Record<string, string> = {
		'NEC.on': '2600500000012396121312131213123713371114121311141237121312131238121311381238113811141213111412371213121312131313123712371237121312381237123712381200054b0001264b11000d05',
		'NEC.off': '2600500000012395121411131114123712381114111411141238111412131238121311381238113811141213111412131238111412381237123812371237123712381200054c0001254b12000d05',
		'NEC.menu': '2600500000012396121312141213123712381113121311141237121312131238121312371238123711141213111412131238121311141237123812371237123712381200054c0001254b12000d05',
		'NEC.ok': '2600500000012395121411131114123712381113121311141237121312131238121312371238113812131114111412131238121311141114123812371237123712381200054b0001264b11000d05',
		'NEC.up': '2600500000012396121312131213123712381114111411141237121312131238121311381238113811141213121311141238111412381237121312381237123712381200054b0001264b11000d05',
		'NEC.back': '2600500000012396111412131213123712381113121312131238111412131238121312371238113811141213121311141238111412371238121312381237123712381200054b0001264a12000d05',
		'NEC.left': '2600500000012396121312131213123712381114111411141237121411131238121312371238113811141213111412371114123811141237123812371237123712371200054c0001254b12000d05',
		'NEC.right': '2600500000012395121411131114123712381113121312131238111412131238121311381237123712141114111412131238121311141114123812371237123712381200054b0001264b11000d05',
		'NEC.power': '2600500000012396121312131213123712381114111412131238121312131238121311381238113811141213111412381213111412381237123712381237123712381200054b0001264b11000d05',
		'NEC.picture_mute': '2600500000012396121312131213123712381114111411141238121312131238121312371238113811141213111412371213121311141238123712381237123712381200054b0001264b11000d05',
		'ROLL.up': 'b1c0fc01469f060008bc07111405140507111505130513060712130513070612061312070612130612071306120607130513051307121306051307bd07121207130605131207130612060713120612070613051312070613120612081107120705140514041406131207051306be06121307120606121307120612070613120712060613061312060613120712061208120605140613051305131208051305c005131207110805131207120712060614120612070613051312070613120612071208110705130613051405131208041405bf05131207120705131207120712070513120811070514061312060613120712061208110705140514051305131208051305c005131206120805131206120812060514120712070513061312070513120712071107120805130515041405131208041405bf05131207120705141107120811070513120812060513071312060514110812071107120804140513061305141107061305bf06131207110706131207120712060613120712070513061312070513120811071207110805130514061305131207061305be07131206120706131206120712070514110712080414051411080414120711081107120805130513061405131206061306be061312071206071213061206120805131207110805130613110805131207120712071107061404140514051411070514050005dc',
		'ROLL.down': 'b1c0fc018d9e05000a3e061312071207060b131007130612070613060712070514051405140514050712061311080612051306be0612130712060613110812060713061206140506140614051406140505130614061306150504140505140605140505bd0613120712060613110812060613070612061305071305140515041405051306140614051504051405051306051405050005dc',
		'ROLL.stop': 'b1c0fc014e9f0500094906131307110706131307120712060614110712070514051312070514120612071208110705131406130605140514060613050005dc',
		'CASCADE.temp_up': '2600a600010195161015101510161015101510151016100f1610151010151010151016100f16100f16101510151010151016100f1610151010151015101510151016100f16101510101510161015100f16101510151010151015101510151010151010151016100f1610151016100f16100f1610151010151016100f161010151015100f160001a00100960f16100f16101510151015101510151015100005dc',
	};

	let selectedCommands = new Set<string>();
	let selectedDeviceId = '';

	$: if (open && $defaultDevice && !selectedDeviceId) {
		selectedDeviceId = $defaultDevice.id;
	}

	$: if (open && dialogElement) {
		dialogElement.showModal();
		// Pre-select all commands
		selectedCommands = new Set(predefinedCommands.map(c => c.filename));
	} else if (!open && dialogElement) {
		dialogElement.close();
	}

	function toggleCommand(filename: string) {
		if (selectedCommands.has(filename)) {
			selectedCommands.delete(filename);
		} else {
			selectedCommands.add(filename);
		}
		selectedCommands = selectedCommands; // Trigger reactivity
	}

	function selectAll() {
		selectedCommands = new Set(predefinedCommands.map(c => c.filename));
	}

	function selectNone() {
		selectedCommands = new Set();
	}

	async function handleImport() {
		if (!selectedDeviceId) {
			toast({
				title: 'No Device Selected',
				description: 'Please select a device to associate with these commands',
				variant: 'warning'
			});
			return;
		}

		if (selectedCommands.size === 0) {
			toast({
				title: 'No Commands Selected',
				description: 'Please select at least one command to import',
				variant: 'warning'
			});
			return;
		}

		isImporting = true;

		try {
			const commandsToImport = predefinedCommands
				.filter(c => selectedCommands.has(c.filename))
				.map(c => ({
					name: c.name,
					deviceId: selectedDeviceId,
					code: codeFiles[c.filename] || '',
					type: c.type,
					category: c.category
				}))
				.filter(c => c.code); // Only import commands with valid codes

			const result = await rfIrStore.importCommands(commandsToImport, { skipExisting: true });

			toast({
				title: 'Import Complete',
				description: `Imported ${result.imported} commands, skipped ${result.skipped} existing`,
				variant: 'success'
			});

			dispatch('imported', { count: result.imported });
			open = false;
		} catch (error) {
			toast({
				title: 'Import Failed',
				description: error instanceof Error ? error.message : 'Failed to import commands',
				variant: 'error'
			});
		} finally {
			isImporting = false;
		}
	}

	function handleDialogClose() {
		open = false;
	}
</script>

<dialog
	bind:this={dialogElement}
	class="fixed z-50 bg-background border rounded-lg shadow-lg max-w-lg w-full backdrop:bg-black/50 backdrop:backdrop-blur-sm p-0 max-h-[80vh] overflow-hidden"
	onclose={handleDialogClose}
>
	<div class="p-6 space-y-4 overflow-y-auto max-h-[calc(80vh-2rem)]">
		<div class="space-y-2">
			<div class="flex items-center gap-2">
				<Upload class="h-5 w-5" />
				<h2 class="text-lg font-semibold">Import Predefined Commands</h2>
			</div>
			<p class="text-sm text-muted-foreground">
				Import IR/RF codes from sermon-helper-service. Select the commands you want to add.
			</p>
		</div>

		{#if $rfIrDevices.length === 0}
			<div class="rounded-lg bg-amber-50 border border-amber-200 p-4 flex items-start gap-3">
				<AlertCircle class="h-5 w-5 text-amber-600 flex-shrink-0 mt-0.5" />
				<div>
					<p class="text-sm font-medium text-amber-800">No Devices Configured</p>
					<p class="text-sm text-amber-700 mt-1">
						Please add a Broadlink device first before importing commands.
					</p>
				</div>
			</div>
		{:else}
			<div class="space-y-2">
				<Label for="import-device">Associate with Device</Label>
				<select
					id="import-device"
					bind:value={selectedDeviceId}
					class="h-9 w-full rounded-md border border-input bg-background px-3 text-sm"
				>
					{#each $rfIrDevices as device}
						<option value={device.id}>
							{device.name} ({device.model})
						</option>
					{/each}
				</select>
			</div>

			<div class="space-y-2">
				<div class="flex items-center justify-between">
					<Label>Commands to Import ({selectedCommands.size}/{predefinedCommands.length})</Label>
					<div class="flex gap-2">
						<button
							type="button"
							class="text-xs text-primary hover:underline"
							onclick={selectAll}
						>
							Select All
						</button>
						<span class="text-muted-foreground">|</span>
						<button
							type="button"
							class="text-xs text-primary hover:underline"
							onclick={selectNone}
						>
							Select None
						</button>
					</div>
				</div>

				<div class="max-h-64 overflow-y-auto rounded-lg border divide-y">
					{#each predefinedCommands as cmd}
						{@const hasCode = !!codeFiles[cmd.filename]}
						<label
							class="flex items-center gap-3 p-3 hover:bg-muted/50 cursor-pointer transition-colors"
							class:opacity-50={!hasCode}
						>
							<input
								type="checkbox"
								checked={selectedCommands.has(cmd.filename)}
								disabled={!hasCode}
								onchange={() => toggleCommand(cmd.filename)}
								class="h-4 w-4 rounded"
							/>
							<div class="flex-1 min-w-0">
								<div class="flex items-center gap-2">
									<span class="font-medium text-sm">{cmd.name}</span>
									<span class="text-[10px] px-1.5 py-0.5 rounded bg-muted text-muted-foreground uppercase">
										{cmd.type}
									</span>
								</div>
								<span class="text-xs text-muted-foreground capitalize">{cmd.category}</span>
							</div>
							{#if !hasCode}
								<span class="text-xs text-amber-600">No code</span>
							{/if}
						</label>
					{/each}
				</div>
			</div>
		{/if}

		<div class="flex gap-2 pt-4 border-t">
			<Button buttonVariant="outline" onclick={() => open = false} className="flex-1">
				Cancel
			</Button>
			<Button
				onclick={handleImport}
				disabled={isImporting || $rfIrDevices.length === 0 || selectedCommands.size === 0}
				className="flex-1"
			>
				{#if isImporting}
					<Loader2 class="mr-2 h-4 w-4 animate-spin" />
					Importing...
				{:else}
					<Upload class="mr-2 h-4 w-4" />
					Import Selected
				{/if}
			</Button>
		</div>
	</div>
</dialog>
