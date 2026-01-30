<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import Button from '$lib/components/ui/button.svelte';
	import Input from '$lib/components/ui/input.svelte';
	import Label from '$lib/components/ui/label.svelte';
	import { toast } from '$lib/utils/toast';
	import { rfIrStore, rfIrDevices, defaultDevice, learnModeState } from '$lib/stores/rf-ir-store';
	import { broadlinkService } from '$lib/utils/broadlink-service';
	import { COMMAND_CATEGORIES } from '$lib/types/rf-ir';
	import { Radio, Loader2, Check, X, AlertCircle } from 'lucide-svelte';

	export let open = false;

	const dispatch = createEventDispatcher<{ saved: { slug: string } }>();

	let dialogElement: HTMLDialogElement;

	// Form state
	let selectedDeviceId = '';
	let signalType: 'ir' | 'rf' = 'ir';
	let commandName = '';
	let commandCategory = 'other';

	// Learning state
	let isLearning = false;
	let learnedCode: string | null = null;
	let learnError: string | null = null;

	// Set default device when dialog opens
	$: if (open && $defaultDevice && !selectedDeviceId) {
		selectedDeviceId = $defaultDevice.id;
	}

	// Dialog open/close effect
	$: if (open && dialogElement) {
		dialogElement.showModal();
		resetForm();
	} else if (!open && dialogElement) {
		dialogElement.close();
	}

	function resetForm() {
		commandName = '';
		commandCategory = 'other';
		learnedCode = null;
		learnError = null;
		isLearning = false;
		if ($defaultDevice) {
			selectedDeviceId = $defaultDevice.id;
		}
	}

	function handleClose() {
		if (isLearning) {
			broadlinkService.cancelLearning();
		}
		open = false;
	}

	async function handleStartLearning() {
		if (!selectedDeviceId) {
			toast({
				title: 'No Device Selected',
				description: 'Please select a device',
				variant: 'warning'
			});
			return;
		}

		isLearning = true;
		learnedCode = null;
		learnError = null;

		try {
			console.log('Starting learning for device:', selectedDeviceId, 'signal type:', signalType);
			const code = await broadlinkService.startLearning(selectedDeviceId, signalType);
			console.log('Learning successful, code length:', code?.length);
			learnedCode = code;
		} catch (error) {
			console.error('Learning failed:', error);
			learnError = error instanceof Error ? error.message : 'Learning failed';
		} finally {
			isLearning = false;
		}
	}

	async function handleCancelLearning() {
		await broadlinkService.cancelLearning();
		isLearning = false;
	}

	async function handleSave() {
		if (!commandName.trim()) {
			toast({
				title: 'Name Required',
				description: 'Please enter a command name',
				variant: 'warning'
			});
			return;
		}

		if (!learnedCode) {
			toast({
				title: 'No Code',
				description: 'Please learn a code first',
				variant: 'warning'
			});
			return;
		}

		try {
			const slug = await rfIrStore.addCommand({
				name: commandName.trim(),
				deviceId: selectedDeviceId,
				code: learnedCode,
				type: signalType,
				category: commandCategory
			});

			dispatch('saved', { slug });
			open = false;
		} catch (error) {
			toast({
				title: 'Save Failed',
				description: error instanceof Error ? error.message : 'Failed to save command',
				variant: 'error'
			});
		}
	}

	function handleDialogClose() {
		open = false;
	}
</script>

<dialog
	bind:this={dialogElement}
	class="fixed z-50 bg-background border rounded-lg shadow-lg max-w-md w-full backdrop:bg-black/50 backdrop:backdrop-blur-sm p-0"
	onclose={handleDialogClose}
>
	<div class="p-6 space-y-4">
		<div class="space-y-2">
			<div class="flex items-center gap-2">
				<Radio class="h-5 w-5" />
				<h2 class="text-lg font-semibold">Learn New Command</h2>
			</div>
			<p class="text-sm text-muted-foreground">
				Point your remote at the Broadlink device and press a button to learn its code.
			</p>
		</div>

		{#if isLearning}
			<!-- Step 2: Learning in progress -->
			<div class="py-8 text-center space-y-4">
				<div class="relative mx-auto w-16 h-16">
					<div class="absolute inset-0 rounded-full border-4 border-primary/30 animate-ping"></div>
					<div class="absolute inset-0 flex items-center justify-center">
						<Radio class="h-8 w-8 text-primary animate-pulse" />
					</div>
				</div>
				<div>
					<p class="font-medium">Waiting for {signalType.toUpperCase()} signal...</p>
					<p class="text-sm text-muted-foreground mt-1">
						Point your remote at the device and press a button
					</p>
				</div>
				<Button buttonVariant="outline" onclick={handleCancelLearning}>
					Cancel
				</Button>
			</div>
		{:else if learnError}
			<!-- Error state -->
			<div class="py-6 text-center space-y-4">
				<div class="mx-auto w-12 h-12 rounded-full bg-red-100 flex items-center justify-center">
					<X class="h-6 w-6 text-red-600" />
				</div>
				<div>
					<p class="font-medium text-red-600">Learning Failed</p>
					<p class="text-sm text-muted-foreground mt-1">{learnError}</p>
				</div>
				<Button onclick={handleStartLearning}>
					Try Again
				</Button>
			</div>
		{:else if learnedCode}
			<!-- Step 3: Code learned, confirm save -->
			<div class="space-y-4">
				<div class="py-4 text-center">
					<div class="mx-auto w-12 h-12 rounded-full bg-green-100 flex items-center justify-center mb-3">
						<Check class="h-6 w-6 text-green-600" />
					</div>
					<p class="font-medium text-green-600">Code Learned Successfully</p>
				</div>

				<div class="space-y-2">
					<Label>Learned Code</Label>
					<div class="p-2 bg-muted rounded-md">
						<code class="text-xs break-all block max-h-20 overflow-y-auto">
							{learnedCode.slice(0, 100)}{learnedCode.length > 100 ? '...' : ''}
						</code>
					</div>
				</div>

				<div class="space-y-2">
					<Label for="confirm-name">Command Name</Label>
					<Input
						id="confirm-name"
						bind:value={commandName}
						placeholder="e.g., Projector Power On"
					/>
				</div>

				<div class="space-y-2">
					<Label for="confirm-category">Category</Label>
					<select
						id="confirm-category"
						bind:value={commandCategory}
						class="h-9 w-full rounded-md border border-input bg-background px-3 text-sm"
					>
						{#each COMMAND_CATEGORIES as category}
							<option value={category}>
								{category.charAt(0).toUpperCase() + category.slice(1)}
							</option>
						{/each}
					</select>
				</div>

				<div class="flex gap-2">
					<Button buttonVariant="outline" onclick={handleStartLearning} className="flex-1">
						Learn Again
					</Button>
					<Button onclick={handleSave} className="flex-1">
						Save Command
					</Button>
				</div>
			</div>
		{:else}
			<!-- Step 1: Configure (default form) -->
			<div class="space-y-4">
				<div class="space-y-2">
					<Label for="learn-device">Device</Label>
					<select
						id="learn-device"
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
					<Label>Signal Type</Label>
					<div class="flex gap-4">
						<label class="flex items-center gap-2 cursor-pointer">
							<input
								type="radio"
								bind:group={signalType}
								value="ir"
								class="h-4 w-4"
							/>
							<span class="text-sm">Infrared (IR)</span>
						</label>
						<label class="flex items-center gap-2 cursor-pointer">
							<input
								type="radio"
								bind:group={signalType}
								value="rf"
								class="h-4 w-4"
							/>
							<span class="text-sm">Radio Frequency (RF)</span>
						</label>
					</div>
				</div>

				<div class="space-y-2">
					<Label for="learn-name">Command Name</Label>
					<Input
						id="learn-name"
						bind:value={commandName}
						placeholder="e.g., Projector Power On"
					/>
				</div>

				<div class="space-y-2">
					<Label for="learn-category">Category</Label>
					<select
						id="learn-category"
						bind:value={commandCategory}
						class="h-9 w-full rounded-md border border-input bg-background px-3 text-sm"
					>
						{#each COMMAND_CATEGORIES as category}
							<option value={category}>
								{category.charAt(0).toUpperCase() + category.slice(1)}
							</option>
						{/each}
					</select>
				</div>

				<Button onclick={handleStartLearning} className="w-full">
					Start Learning
				</Button>
			</div>
		{/if}

		<div class="flex justify-end pt-4 border-t">
			<Button buttonVariant="ghost" onclick={handleClose}>
				{learnedCode ? 'Cancel' : 'Close'}
			</Button>
		</div>
	</div>
</dialog>
