import type { CompanionActionDefinitions, CompanionActionEvent } from '@companion-module/base'
import type { ModuleInstance } from './main.js'

export function GetActions(instance: ModuleInstance): CompanionActionDefinitions {
	const commandChoices = instance.commands.map((cmd) => ({
		id: cmd.slug,
		label: `${cmd.name} (${cmd.category})`,
	}))

	return {
		execute_command: {
			name: 'Execute RF/IR Command',
			description: 'Execute a saved RF/IR command by slug',
			options: [
				{
					type: 'dropdown',
					id: 'slug',
					label: 'Command',
					default: commandChoices[0]?.id || '',
					choices: commandChoices,
					allowCustom: true,
					tooltip: 'Select the command to execute',
				},
			],
			callback: async (action: CompanionActionEvent) => {
				const slug = action.options['slug'] as string
				if (!slug) {
					instance.log('warn', 'No command slug specified')
					return
				}

				instance.log('debug', `Executing command: ${slug}`)
				const result = await instance.api.executeCommand(slug)

				if (result.success) {
					instance.log('info', `Command executed: ${slug}`)
					instance.setVariableValues({
						last_command: instance.commands.find((c) => c.slug === slug)?.name || slug,
					})
				} else {
					instance.log('error', `Command failed: ${slug} - ${result.error}`)
				}
			},
		},

		execute_by_category: {
			name: 'Execute Command by Category',
			description: 'Execute a command filtered by category',
			options: [
				{
					type: 'dropdown',
					id: 'category',
					label: 'Category',
					default: 'projector',
					choices: [
						{ id: 'projector', label: 'Projector' },
						{ id: 'screen', label: 'Screen' },
						{ id: 'hvac', label: 'HVAC' },
						{ id: 'lighting', label: 'Lighting' },
						{ id: 'audio', label: 'Audio' },
						{ id: 'other', label: 'Other' },
					],
				},
				{
					type: 'dropdown',
					id: 'slug',
					label: 'Command',
					default: '',
					choices: commandChoices,
					allowCustom: true,
					tooltip: 'Select the command to execute',
				},
			],
			callback: async (action: CompanionActionEvent) => {
				const slug = action.options['slug'] as string
				if (!slug) {
					instance.log('warn', 'No command slug specified')
					return
				}

				instance.log('debug', `Executing command: ${slug}`)
				const result = await instance.api.executeCommand(slug)

				if (result.success) {
					instance.log('info', `Command executed: ${slug}`)
					instance.setVariableValues({
						last_command: instance.commands.find((c) => c.slug === slug)?.name || slug,
					})
				} else {
					instance.log('error', `Command failed: ${slug} - ${result.error}`)
				}
			},
		},

		refresh_commands: {
			name: 'Refresh Command List',
			description: 'Manually refresh the list of available commands from the server',
			options: [],
			callback: async () => {
				instance.log('info', 'Refreshing command list...')
				await instance.refreshCommands()
			},
		},

		// PPT Selector Actions
		ppt_digit: {
			name: 'PPT: Type Digit',
			description: 'Append a digit to the PPT file filter',
			options: [
				{
					type: 'dropdown',
					id: 'digit',
					label: 'Digit',
					default: '0',
					choices: [
						{ id: '0', label: '0' },
						{ id: '1', label: '1' },
						{ id: '2', label: '2' },
						{ id: '3', label: '3' },
						{ id: '4', label: '4' },
						{ id: '5', label: '5' },
						{ id: '6', label: '6' },
						{ id: '7', label: '7' },
						{ id: '8', label: '8' },
						{ id: '9', label: '9' },
					],
				},
			],
			callback: async (action: CompanionActionEvent) => {
				const digit = parseInt(action.options['digit'] as string, 10)
				if (!isNaN(digit)) {
					instance.log('debug', `PPT: Adding digit ${digit}`)
					await instance.pptSelector.appendDigit(digit)
				}
			},
		},

		ppt_backspace: {
			name: 'PPT: Backspace',
			description: 'Remove the last digit from the PPT file filter',
			options: [],
			callback: async () => {
				instance.log('debug', 'PPT: Backspace')
				await instance.pptSelector.backspace()
			},
		},

		ppt_clear: {
			name: 'PPT: Clear Filter',
			description: 'Clear the PPT file filter completely',
			options: [],
			callback: async () => {
				instance.log('debug', 'PPT: Clear filter')
				await instance.pptSelector.clearFilter()
			},
		},

		ppt_select_slot: {
			name: 'PPT: Select File',
			description: 'Open the PPT file at a specific display slot',
			options: [
				{
					type: 'dropdown',
					id: 'slot',
					label: 'Slot',
					default: '1',
					choices: [
						{ id: '1', label: 'Slot 1' },
						{ id: '2', label: 'Slot 2' },
						{ id: '3', label: 'Slot 3' },
						{ id: '4', label: 'Slot 4' },
						{ id: '5', label: 'Slot 5' },
					],
				},
				{
					type: 'checkbox',
					id: 'startPresenter',
					label: 'Start Presenter Mode',
					default: true,
				},
			],
			callback: async (action: CompanionActionEvent) => {
				const slot = parseInt(action.options['slot'] as string, 10) - 1 // Convert to 0-indexed
				const startPresenter = action.options['startPresenter'] as boolean

				const file = instance.pptSelector.getFileAtSlot(slot)
				if (!file) {
					instance.log('warn', `PPT: No file at slot ${slot + 1}`)
					return
				}

				instance.log('info', `PPT: Opening file at slot ${slot + 1}: ${file.name}`)
				const result = await instance.pptSelector.openFileAtSlot(slot, startPresenter)

				if (!result.success) {
					instance.log('error', `PPT: Failed to open file: ${result.error}`)
				}
			},
		},

		ppt_select_folder: {
			name: 'PPT: Select Folder',
			description: 'Switch to a different PPT folder',
			options: [
				{
					type: 'dropdown',
					id: 'folderId',
					label: 'Folder',
					default: instance.pptSelector.getFolderChoices()[0]?.id || '',
					choices: instance.pptSelector.getFolderChoices(),
					allowCustom: false,
				},
			],
			callback: async (action: CompanionActionEvent) => {
				const folderId = action.options['folderId'] as string
				if (!folderId) {
					instance.log('warn', 'PPT: No folder selected')
					return
				}

				instance.log('debug', `PPT: Selecting folder ${folderId}`)
				await instance.pptSelector.selectFolder(folderId)
			},
		},

		ppt_refresh: {
			name: 'PPT: Refresh Files',
			description: 'Refresh the list of PPT files from the current folder',
			options: [],
			callback: async () => {
				instance.log('info', 'PPT: Refreshing files...')
				await instance.pptSelector.refreshFiles()
			},
		},
	}
}
