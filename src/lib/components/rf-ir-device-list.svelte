<script lang="ts">
	import Button from '$lib/components/ui/button.svelte';
	import Input from '$lib/components/ui/input.svelte';
	import Label from '$lib/components/ui/label.svelte';
	import { toast } from '$lib/utils/toast';
	import { rfIrStore, rfIrDevices, defaultDevice } from '$lib/stores/rf-ir-store';
	import { broadlinkService } from '$lib/utils/broadlink-service';
	import type { BroadlinkDevice } from '$lib/types/rf-ir';
	import {
		Cpu,
		Trash2,
		Star,
		StarOff,
		Plus,
		ChevronDown,
		ChevronUp,
		Loader2,
		Check,
		X
	} from 'lucide-svelte';

	let showAddForm = false;
	let isTestingDevice: string | null = null;
	let deviceTestResults: Record<string, boolean | null> = {};

	// Form state for adding device manually
	let newDeviceName = '';
	let newDeviceHost = '';
	let newDeviceMac = '';
	let newDeviceType = '0x5f36'; // Default RM4 Pro type
	let newDeviceModel = 'RM4 Pro';

	const deviceModels = [
		{ type: '0x5f36', model: 'RM4 Pro' },
		{ type: '0x6026', model: 'RM4 Mini' },
		{ type: '0x51da', model: 'RM4 Mini' },
		{ type: '0x610e', model: 'RM4 Mini' },
		{ type: '0x62bc', model: 'RM4 C Mini' },
		{ type: '0x27a9', model: 'RM Pro+' },
		{ type: '0x2712', model: 'RM Pro+' }
	];

	function formatLastSeen(timestamp: number): string {
		const diff = Date.now() - timestamp;
		if (diff < 60000) return 'Just now';
		if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
		if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
		return new Date(timestamp).toLocaleDateString();
	}

	async function handleAddDevice() {
		if (!newDeviceName.trim() || !newDeviceHost.trim() || !newDeviceMac.trim()) {
			toast({
				title: 'Missing Fields',
				description: 'Please fill in all required fields',
				variant: 'warning'
			});
			return;
		}

		// Validate MAC address format
		const macRegex = /^([0-9a-fA-F]{2}:){5}[0-9a-fA-F]{2}$/;
		if (!macRegex.test(newDeviceMac)) {
			toast({
				title: 'Invalid MAC Address',
				description: 'MAC address should be in format aa:bb:cc:dd:ee:ff',
				variant: 'warning'
			});
			return;
		}

		// Validate IP address format
		const ipRegex = /^(\d{1,3}\.){3}\d{1,3}$/;
		if (!ipRegex.test(newDeviceHost)) {
			toast({
				title: 'Invalid IP Address',
				description: 'Please enter a valid IP address',
				variant: 'warning'
			});
			return;
		}

		try {
			await rfIrStore.addDevice({
				name: newDeviceName.trim(),
				host: newDeviceHost.trim(),
				mac: newDeviceMac.trim().toLowerCase(),
				type: newDeviceType,
				model: newDeviceModel
			});

			// Reset form
			newDeviceName = '';
			newDeviceHost = '';
			newDeviceMac = '';
			showAddForm = false;

			toast({
				title: 'Device Added',
				description: 'Broadlink device has been added',
				variant: 'success'
			});
		} catch (error) {
			toast({
				title: 'Failed to Add Device',
				description: error instanceof Error ? error.message : 'Unknown error',
				variant: 'error'
			});
		}
	}

	async function handleRemoveDevice(device: BroadlinkDevice) {
		if (!confirm(`Remove device "${device.name}"? This will also remove all associated commands.`)) {
			return;
		}

		try {
			await rfIrStore.removeDevice(device.id);
			toast({
				title: 'Device Removed',
				description: `${device.name} has been removed`,
				variant: 'success'
			});
		} catch (error) {
			toast({
				title: 'Failed to Remove Device',
				description: error instanceof Error ? error.message : 'Unknown error',
				variant: 'error'
			});
		}
	}

	async function handleSetDefault(device: BroadlinkDevice) {
		try {
			await rfIrStore.setDefaultDevice(device.id);
		} catch (error) {
			toast({
				title: 'Failed to Set Default',
				description: error instanceof Error ? error.message : 'Unknown error',
				variant: 'error'
			});
		}
	}

	async function handleTestDevice(device: BroadlinkDevice) {
		isTestingDevice = device.id;
		deviceTestResults[device.id] = null;

		try {
			const result = await broadlinkService.testDevice(device.id);
			deviceTestResults[device.id] = result;

			if (result) {
				toast({
					title: 'Device Online',
					description: `${device.name} is responding`,
					variant: 'success'
				});
			} else {
				toast({
					title: 'Device Offline',
					description: `${device.name} is not responding`,
					variant: 'warning'
				});
			}
		} catch (error) {
			deviceTestResults[device.id] = false;
			toast({
				title: 'Test Failed',
				description: error instanceof Error ? error.message : 'Unknown error',
				variant: 'error'
			});
		} finally {
			isTestingDevice = null;
		}
	}

	function handleModelChange(event: Event) {
		const target = event.target as HTMLSelectElement;
		const selected = deviceModels.find((m) => m.type === target.value);
		if (selected) {
			newDeviceType = selected.type;
			newDeviceModel = selected.model;
		}
	}
</script>

<div class="rounded-lg border p-4 space-y-4">
	<div class="flex items-center justify-between">
		<div class="flex items-center gap-2">
			<Cpu class="h-4 w-4" />
			<h4 class="font-medium text-sm">Devices ({$rfIrDevices.length})</h4>
		</div>
		<Button
			buttonVariant="ghost"
			buttonSize="sm"
			onclick={() => (showAddForm = !showAddForm)}
		>
			{#if showAddForm}
				<ChevronUp class="h-4 w-4" />
			{:else}
				<Plus class="mr-1 h-3 w-3" />
				Add
			{/if}
		</Button>
	</div>

	<!-- Add Device Form -->
	{#if showAddForm}
		<div class="rounded-lg bg-muted/50 p-3 space-y-3">
			<div class="grid grid-cols-2 gap-3">
				<div class="space-y-1.5">
					<Label for="device-name" class="text-xs">Name</Label>
					<Input
						id="device-name"
						bind:value={newDeviceName}
						placeholder="Living Room RM4"
						class="h-8 text-sm"
					/>
				</div>
				<div class="space-y-1.5">
					<Label for="device-model" class="text-xs">Model</Label>
					<select
						id="device-model"
						value={newDeviceType}
						onchange={handleModelChange}
						class="h-8 w-full rounded-md border border-input bg-background px-3 text-sm"
					>
						{#each deviceModels as model}
							<option value={model.type}>{model.model}</option>
						{/each}
					</select>
				</div>
			</div>
			<div class="grid grid-cols-2 gap-3">
				<div class="space-y-1.5">
					<Label for="device-host" class="text-xs">IP Address</Label>
					<Input
						id="device-host"
						bind:value={newDeviceHost}
						placeholder="192.168.1.100"
						class="h-8 text-sm font-mono"
					/>
				</div>
				<div class="space-y-1.5">
					<Label for="device-mac" class="text-xs">MAC Address</Label>
					<Input
						id="device-mac"
						bind:value={newDeviceMac}
						placeholder="aa:bb:cc:dd:ee:ff"
						class="h-8 text-sm font-mono"
					/>
				</div>
			</div>
			<div class="flex gap-2">
				<Button
					buttonVariant="outline"
					buttonSize="sm"
					onclick={() => (showAddForm = false)}
					className="flex-1"
				>
					Cancel
				</Button>
				<Button buttonSize="sm" onclick={handleAddDevice} className="flex-1">
					Add Device
				</Button>
			</div>
		</div>
	{/if}

	<!-- Device List -->
	{#if $rfIrDevices.length === 0}
		<p class="text-sm text-muted-foreground text-center py-4">
			No devices configured. Add a device manually or use discovery.
		</p>
	{:else}
		<div class="space-y-2">
			{#each $rfIrDevices as device (device.id)}
				<div
					class="flex items-center justify-between p-3 rounded-lg bg-muted/50 hover:bg-muted transition-colors"
				>
					<div class="flex items-center gap-3 min-w-0">
						<button
							class="flex-shrink-0 text-muted-foreground hover:text-yellow-500 transition-colors"
							onclick={() => handleSetDefault(device)}
							title={device.isDefault ? 'Default device' : 'Set as default'}
						>
							{#if device.isDefault}
								<Star class="h-4 w-4 text-yellow-500 fill-yellow-500" />
							{:else}
								<StarOff class="h-4 w-4" />
							{/if}
						</button>
						<div class="min-w-0">
							<div class="flex items-center gap-2">
								<span class="font-medium text-sm truncate">{device.name}</span>
								<span class="text-xs text-muted-foreground">({device.model})</span>
								{#if deviceTestResults[device.id] === true}
									<Check class="h-3 w-3 text-green-600" />
								{:else if deviceTestResults[device.id] === false}
									<X class="h-3 w-3 text-red-600" />
								{/if}
							</div>
							<div class="flex items-center gap-2 text-xs text-muted-foreground">
								<code>{device.host}</code>
								<span>-</span>
								<span>{formatLastSeen(device.lastSeen)}</span>
							</div>
						</div>
					</div>
					<div class="flex items-center gap-1 flex-shrink-0">
						<Button
							buttonVariant="ghost"
							buttonSize="sm"
							onclick={() => handleTestDevice(device)}
							disabled={isTestingDevice === device.id}
							className="h-7 px-2"
						>
							{#if isTestingDevice === device.id}
								<Loader2 class="h-3 w-3 animate-spin" />
							{:else}
								Test
							{/if}
						</Button>
						<Button
							buttonVariant="ghost"
							buttonSize="sm"
							onclick={() => handleRemoveDevice(device)}
							className="h-7 px-2 text-red-600 hover:text-red-700 hover:bg-red-100"
						>
							<Trash2 class="h-3 w-3" />
						</Button>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>
