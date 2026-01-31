<script lang="ts">
	import Card from '$lib/components/ui/card.svelte';
	import Button from '$lib/components/ui/button.svelte';
	import Input from '$lib/components/ui/input.svelte';
	import { toast } from '$lib/utils/toast';
	import { appSettingsStore } from '$lib/utils/app-settings-store';
	import type { PptFolder, PptSettings } from '$lib/types/ppt';
	import { DEFAULT_PPT_SETTINGS } from '$lib/types/ppt';
	import { FolderOpen, Trash2, Plus, FileSliders, Wand2, Download } from 'lucide-svelte';
	import { onMount } from 'svelte';
	import { open, save } from '@tauri-apps/plugin-dialog';
	import { writeTextFile } from '@tauri-apps/plugin-fs';

	let settings: PptSettings = { ...DEFAULT_PPT_SETTINGS };
	let isLoading = true;
	let isSaving = false;

	// Companion instance name - should match user's existing connection label
	let companionInstanceName = 'Sermon_Helper';

	onMount(async () => {
		try {
			const appSettingsValue = await appSettingsStore.get('pptSettings');
			if (appSettingsValue) {
				settings = { ...DEFAULT_PPT_SETTINGS, ...appSettingsValue };
			}
		} catch (error) {
			console.error('Failed to load PPT settings:', error);
		}
		isLoading = false;
	});

	async function handleAddFolder() {
		try {
			const selected = await open({
				directory: true,
				multiple: false,
				title: 'Select PPT Folder'
			});

			if (selected && typeof selected === 'string') {
				// Generate a name from the folder path
				const pathParts = selected.replace(/\\/g, '/').split('/');
				const folderName = pathParts[pathParts.length - 1] || 'Untitled';

				// Check if folder already exists
				if (settings.folders.some((f) => f.path === selected)) {
					toast({
						title: 'Folder Already Added',
						description: 'This folder is already in your list',
						variant: 'warning'
					});
					return;
				}

				const newFolder: PptFolder = {
					id: crypto.randomUUID(),
					path: selected,
					name: folderName
				};

				settings.folders = [...settings.folders, newFolder];
				await saveSettings();

				toast({
					title: 'Folder Added',
					description: `Added "${folderName}" to PPT folders`,
					variant: 'success'
				});
			}
		} catch (error) {
			console.error('Failed to add folder:', error);
			toast({
				title: 'Failed to Add Folder',
				description: error instanceof Error ? error.message : 'Unknown error',
				variant: 'error'
			});
		}
	}

	async function handleRemoveFolder(id: string) {
		const folder = settings.folders.find((f) => f.id === id);
		settings.folders = settings.folders.filter((f) => f.id !== id);
		await saveSettings();

		toast({
			title: 'Folder Removed',
			description: folder ? `Removed "${folder.name}"` : 'Folder removed',
			variant: 'success'
		});
	}

	async function handleUpdateFolderName(id: string, name: string) {
		settings.folders = settings.folders.map((f) => (f.id === id ? { ...f, name } : f));
		await saveSettings();
	}

	async function saveSettings() {
		isSaving = true;
		try {
			await appSettingsStore.set('pptSettings', settings);
			// No sync needed - discovery server reads directly from settings file
		} catch (error) {
			console.error('Failed to save PPT settings:', error);
			toast({
				title: 'Failed to Save',
				description: 'Could not save PPT settings',
				variant: 'error'
			});
		} finally {
			isSaving = false;
		}
	}

	async function saveCompanionConfig() {
		try {
			// Open save dialog
			const filePath = await save({
				title: 'Save Companion Config',
				defaultPath: 'ppt-selector-page.companionconfig',
				filters: [
					{ name: 'Companion Config', extensions: ['companionconfig'] },
					{ name: 'All Files', extensions: ['*'] }
				]
			});

			if (!filePath) return; // User cancelled

			// Generate the config content
			const configContent = generateCompanionConfig();

			// Write the file
			await writeTextFile(filePath, configContent);

			toast({
				title: 'Config Saved',
				description: 'Import this file in Companion via Buttons → Import',
				variant: 'success'
			});
		} catch (error) {
			toast({
				title: 'Failed to Save',
				description: error instanceof Error ? error.message : String(error),
				variant: 'error'
			});
		}
	}

	function generateCompanionConfig(): string {
		// Use the user-specified instance name
		const instanceLabel = companionInstanceName.trim() || 'Sermon_Helper';
		// Generate a unique connection ID for this export
		const connectionId = 'ppt-selector-connection';

		const config = {
			version: 9,
			type: 'full',
			pages: {
				'1': {
					id: crypto.randomUUID(),
					name: 'PPT Selector',
					controls: {
						// Row 0: Slot buttons
						'0': Object.fromEntries(
							[1, 2, 3, 4, 5].map((slot, col) => [
								col.toString(),
								createSlotButton(slot, connectionId, instanceLabel)
							])
						),
						// Row 1: Digits 1-5
						'1': Object.fromEntries(
							[1, 2, 3, 4, 5].map((digit, col) => [
								col.toString(),
								createDigitButton(digit.toString(), connectionId, instanceLabel)
							])
						),
						// Row 2: Digits 6-0 and controls
						'2': {
							'0': createDigitButton('6', connectionId, instanceLabel),
							'1': createDigitButton('7', connectionId, instanceLabel),
							'2': createDigitButton('8', connectionId, instanceLabel),
							'3': createDigitButton('9', connectionId, instanceLabel),
							'4': createDigitButton('0', connectionId, instanceLabel)
						},
						// Row 3: Control buttons
						'3': {
							'0': createActionButton('⌫', 'ppt_backspace', {}, connectionId, 15680580),
							'1': createActionButton('CLR', 'ppt_clear', {}, connectionId, 16096779, '18'),
							'2': createActionButton('↻', 'ppt_refresh', {}, connectionId, 7041664),
							'3': createDisplayButton(`Filter:\\n$(${instanceLabel}:ppt_filter)`, 3289650),
							'4': createDisplayButton(`$(${instanceLabel}:ppt_match_count)\\nfiles`, 3289650)
						}
					},
					gridSize: { minColumn: 0, maxColumn: 7, minRow: 0, maxRow: 3 }
				}
			},
			triggers: {},
			custom_variables: {},
			instances: {
				[connectionId]: {
					instance_type: 'sermon-helper',
					label: instanceLabel,
					enabled: true,
					sortOrder: 0,
					config: {
						host: '127.0.0.1',
						port: 8765,
						authToken: '',
						useAutoDiscovery: false,
						pollInterval: 30000
					},
					lastUpgradeIndex: -1
				}
			},
			surfaces: {},
			surfaceGroups: {}
		};

		return JSON.stringify(config, null, 2);
	}

	function createDigitButton(digit: string, connectionId: string, instanceLabel: string) {
		return {
			type: 'button',
			style: {
				text: digit,
				textExpression: false,
				size: '44',
				png64: null,
				alignment: 'center:center',
				pngalignment: 'center:center',
				color: 16777215,
				bgcolor: 3900150,
				show_topbar: 'default'
			},
			options: { stepProgression: 'auto', stepExpression: '', rotaryActions: false },
			feedbacks: [
				{
					type: 'feedback',
					id: crypto.randomUUID(),
					connectionId,
					definitionId: 'connection_status',
					options: {},
					isInverted: true,
					style: { bgcolor: 6579300 },
					upgradeIndex: -1
				}
			],
			steps: {
				'0': {
					action_sets: {
						down: [
							{
								type: 'action',
								id: crypto.randomUUID(),
								connectionId,
								definitionId: 'ppt_digit',
								options: { digit },
								upgradeIndex: -1
							}
						],
						up: []
					},
					options: { runWhileHeld: [] }
				}
			},
			localVariables: []
		};
	}

	function createActionButton(
		text: string,
		definitionId: string,
		options: Record<string, unknown>,
		connectionId: string,
		bgcolor: number,
		size = '44'
	) {
		return {
			type: 'button',
			style: {
				text,
				textExpression: false,
				size,
				png64: null,
				alignment: 'center:center',
				pngalignment: 'center:center',
				color: 16777215,
				bgcolor,
				show_topbar: 'default'
			},
			options: { stepProgression: 'auto', stepExpression: '', rotaryActions: false },
			feedbacks: [],
			steps: {
				'0': {
					action_sets: {
						down: [
							{
								type: 'action',
								id: crypto.randomUUID(),
								connectionId,
								definitionId,
								options,
								upgradeIndex: -1
							}
						],
						up: []
					},
					options: { runWhileHeld: [] }
				}
			},
			localVariables: []
		};
	}

	function createDisplayButton(text: string, bgcolor: number) {
		return {
			type: 'button',
			style: {
				text,
				textExpression: false,
				size: '14',
				png64: null,
				alignment: 'center:center',
				pngalignment: 'center:center',
				color: 16777215,
				bgcolor,
				show_topbar: 'default'
			},
			options: { stepProgression: 'auto', stepExpression: '', rotaryActions: false },
			feedbacks: [],
			steps: {
				'0': {
					action_sets: { down: [], up: [] },
					options: { runWhileHeld: [] }
				}
			},
			localVariables: []
		};
	}

	function createSlotButton(slot: number, connectionId: string, instanceLabel: string) {
		return {
			type: 'button',
			style: {
				text: `$(${instanceLabel}:ppt_slot_${slot}_name)`,
				textExpression: false,
				size: '14',
				png64: null,
				alignment: 'center:center',
				pngalignment: 'center:center',
				color: 16777215,
				bgcolor: 6579300,
				show_topbar: 'default'
			},
			options: { stepProgression: 'auto', stepExpression: '', rotaryActions: false },
			feedbacks: [
				{
					type: 'feedback',
					id: crypto.randomUUID(),
					connectionId,
					definitionId: 'ppt_slot_has_file',
					options: { slot: slot.toString() },
					isInverted: false,
					style: { bgcolor: 38400 },
					upgradeIndex: -1
				},
				{
					type: 'feedback',
					id: crypto.randomUUID(),
					connectionId,
					definitionId: 'connection_status',
					options: {},
					isInverted: true,
					style: { bgcolor: 5263440 },
					upgradeIndex: -1
				}
			],
			steps: {
				'0': {
					action_sets: {
						down: [
							{
								type: 'action',
								id: crypto.randomUUID(),
								connectionId,
								definitionId: 'ppt_select_slot',
								options: { slot: slot.toString(), startPresenter: true },
								upgradeIndex: -1
							}
						],
						up: []
					},
					options: { runWhileHeld: [] }
				}
			},
			localVariables: []
		};
	}
</script>

<Card>
	<svelte:fragment slot="title">
		<FileSliders class="h-5 w-5" />
		PPT Folder Configuration
	</svelte:fragment>

	<svelte:fragment slot="description">
		Configure folders containing PowerPoint files for Companion integration
	</svelte:fragment>

	<svelte:fragment slot="content">
		<div class="space-y-4">
			{#if isLoading}
				<div class="text-center py-8 text-muted-foreground">Loading...</div>
			{:else}
				<!-- Folder List -->
				{#if settings.folders.length === 0}
					<div class="rounded-lg border border-dashed p-8 text-center">
						<FolderOpen class="h-12 w-12 mx-auto text-muted-foreground/50 mb-3" />
						<p class="text-muted-foreground">No folders configured</p>
						<p class="text-xs text-muted-foreground mt-1">
							Add folders containing your PowerPoint presentations
						</p>
					</div>
				{:else}
					<div class="space-y-3">
						{#each settings.folders as folder (folder.id)}
							<div class="rounded-lg border bg-card p-4 space-y-2">
								<div class="flex items-start justify-between gap-3">
									<div class="flex-1 min-w-0">
										<Input
											type="text"
											value={folder.name}
											oninput={(e: Event & { currentTarget: HTMLInputElement }) =>
												handleUpdateFolderName(folder.id, e.currentTarget.value)}
											placeholder="Folder name"
											className="font-medium"
										/>
									</div>
									<Button
										buttonVariant="ghost"
										onclick={() => handleRemoveFolder(folder.id)}
										className="h-9 w-9 p-0 text-destructive hover:text-destructive hover:bg-destructive/10"
									>
										<Trash2 class="h-4 w-4" />
									</Button>
								</div>
								<div class="flex items-center gap-2 text-xs text-muted-foreground">
									<FolderOpen class="h-3 w-3 flex-shrink-0" />
									<span class="truncate" title={folder.path}>{folder.path}</span>
								</div>
							</div>
						{/each}
					</div>
				{/if}

				<!-- Add Folder Button -->
				<Button onclick={handleAddFolder} className="w-full" disabled={isSaving}>
					<Plus class="mr-2 h-4 w-4" />
					Add Folder
				</Button>

				<!-- Companion Integration -->
				<div class="rounded-lg border p-4 space-y-4">
					<h4 class="font-medium text-sm flex items-center gap-2">
						<Wand2 class="h-4 w-4" />
						Companion Button Setup
					</h4>

					<p class="text-xs text-muted-foreground">
						Export PPT selector buttons config for Bitfocus Companion.
					</p>

					<!-- Instance Name -->
					<div class="space-y-1">
						<label for="companion-instance" class="text-xs font-medium">
							Connection Label
						</label>
						<Input
							id="companion-instance"
							type="text"
							bind:value={companionInstanceName}
							placeholder="Sermon_Helper"
							className="h-8 text-sm"
						/>
						<p class="text-xs text-muted-foreground">
							Must match your existing connection name in Companion
						</p>
					</div>

					<!-- Action Buttons -->
					<div class="space-y-3">
						<Button onclick={saveCompanionConfig} className="w-full">
							<Download class="mr-2 h-4 w-4" />
							Save Config File
						</Button>

						<div class="text-xs text-muted-foreground space-y-2">
							<p class="font-medium">How to import:</p>
							<ol class="list-decimal list-inside space-y-1">
								<li>Enter your Companion connection label above</li>
								<li>Click <strong>Save Config File</strong></li>
								<li>In Companion → <strong>Buttons</strong> → <strong>Import</strong></li>
								<li>Select the saved file</li>
							</ol>
						</div>
					</div>
				</div>

				<!-- Info -->
				<div class="rounded-lg bg-muted/50 p-4 text-sm space-y-2">
					<p class="font-medium">How it works:</p>
					<ul class="list-disc list-inside text-muted-foreground text-xs space-y-1">
						<li>Add folders containing .ppt, .pptx, or .odp files</li>
						<li>Folders are available to Companion via the discovery server</li>
						<li>Use numeric prefixes (e.g., "01-Welcome.pptx") for easy filtering</li>
						<li>Files are opened in presenter mode when selected from Companion</li>
					</ul>
				</div>
			{/if}
		</div>
	</svelte:fragment>
</Card>
