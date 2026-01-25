<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import Button from '$lib/components/ui/button.svelte';
	import Input from '$lib/components/ui/input.svelte';
	import Label from '$lib/components/ui/label.svelte';
	import Textarea from '$lib/components/ui/textarea.svelte';
	import { toast } from '$lib/utils/toast';
	import { rfIrStore, rfIrDevices, defaultDevice } from '$lib/stores/rf-ir-store';
	import { COMMAND_CATEGORIES } from '$lib/types/rf-ir';
	import { FileCode, AlertCircle } from 'lucide-svelte';

	export let open = false;

	const dispatch = createEventDispatcher<{ saved: { slug: string } }>();

	let dialogElement: HTMLDialogElement;

	// Form state
	let selectedDeviceId = '';
	let signalType: 'ir' | 'rf' = 'ir';
	let commandName = '';
	let commandCategory = 'other';
	let commandCode = '';

	// Validation
	let codeError = '';

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

	// Validate hex code
	$: {
		if (commandCode) {
			const cleaned = commandCode.replace(/\s/g, '');
			if (!/^[0-9a-fA-F]*$/.test(cleaned)) {
				codeError = 'Code must be hexadecimal (0-9, a-f)';
			} else if (cleaned.length < 10) {
				codeError = 'Code seems too short';
			} else if (cleaned.length % 2 !== 0) {
				codeError = 'Code must have even number of characters';
			} else {
				codeError = '';
			}
		} else {
			codeError = '';
		}
	}

	function resetForm() {
		commandName = '';
		commandCategory = 'other';
		commandCode = '';
		signalType = 'ir';
		codeError = '';
		if ($defaultDevice) {
			selectedDeviceId = $defaultDevice.id;
		}
	}

	function handleClose() {
		open = false;
	}

	async function handleSave() {
		if (!selectedDeviceId) {
			toast({
				title: 'No Device Selected',
				description: 'Please select a device',
				variant: 'warning'
			});
			return;
		}

		if (!commandName.trim()) {
			toast({
				title: 'Name Required',
				description: 'Please enter a command name',
				variant: 'warning'
			});
			return;
		}

		const cleanedCode = commandCode.replace(/\s/g, '');
		if (!cleanedCode) {
			toast({
				title: 'Code Required',
				description: 'Please enter the IR/RF code',
				variant: 'warning'
			});
			return;
		}

		if (codeError) {
			toast({
				title: 'Invalid Code',
				description: codeError,
				variant: 'warning'
			});
			return;
		}

		try {
			const slug = await rfIrStore.addCommand({
				name: commandName.trim(),
				deviceId: selectedDeviceId,
				code: cleanedCode.toLowerCase(),
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

	function handlePaste(event: ClipboardEvent) {
		// Auto-clean pasted content
		event.preventDefault();
		const pasted = event.clipboardData?.getData('text') || '';
		// Remove common prefixes and clean up
		const cleaned = pasted
			.replace(/^0x/i, '')
			.replace(/\s+/g, '')
			.replace(/[^0-9a-fA-F]/g, '');
		commandCode = cleaned;
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
				<FileCode class="h-5 w-5" />
				<h2 class="text-lg font-semibold">Add Code Manually</h2>
			</div>
			<p class="text-sm text-muted-foreground">
				Paste an IR/RF code from another source (e.g., exported from another app).
			</p>
		</div>

		<div class="space-y-4">
			<div class="space-y-2">
				<Label for="manual-device">Device</Label>
				<select
					id="manual-device"
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
				<Label for="manual-name">Command Name</Label>
				<Input
					id="manual-name"
					bind:value={commandName}
					placeholder="e.g., Projector Power On"
				/>
			</div>

			<div class="space-y-2">
				<Label for="manual-category">Category</Label>
				<select
					id="manual-category"
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

			<div class="space-y-2">
				<Label for="manual-code">Hex Code</Label>
				<Textarea
					id="manual-code"
					bind:value={commandCode}
					placeholder="Paste hex code here (e.g., 2600500000012396...)"
					rows={4}
					className="font-mono text-xs"
					onpaste={handlePaste}
				/>
				{#if codeError}
					<div class="flex items-center gap-1.5 text-amber-600 text-xs">
						<AlertCircle class="h-3.5 w-3.5" />
						<span>{codeError}</span>
					</div>
				{/if}
				<p class="text-xs text-muted-foreground">
					Paste the raw hex code. Spaces and "0x" prefix will be automatically removed.
				</p>
			</div>
		</div>

		<div class="flex gap-2 pt-4 border-t">
			<Button buttonVariant="outline" onclick={handleClose} className="flex-1">
				Cancel
			</Button>
			<Button onclick={handleSave} disabled={!!codeError} className="flex-1">
				Save Command
			</Button>
		</div>
	</div>
</dialog>
