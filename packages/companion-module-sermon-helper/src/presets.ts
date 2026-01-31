import { combineRgb, type CompanionPresetDefinitions } from '@companion-module/base'
import type { ModuleInstance } from './main.js'
import { CATEGORY_COLORS } from './types.js'

export function GetPresets(instance: ModuleInstance): CompanionPresetDefinitions {
	const presets: CompanionPresetDefinitions = {}

	// Generate a preset for each command
	for (const cmd of instance.commands) {
		const categoryColor = CATEGORY_COLORS[cmd.category] || CATEGORY_COLORS['other']

		// Convert hex color to RGB components
		const r = (categoryColor >> 16) & 0xff
		const g = (categoryColor >> 8) & 0xff
		const b = categoryColor & 0xff

		presets[`cmd_${cmd.slug}`] = {
			type: 'button',
			category: cmd.category.charAt(0).toUpperCase() + cmd.category.slice(1),
			name: cmd.name,
			style: {
				text: cmd.name,
				size: 'auto',
				color: combineRgb(255, 255, 255),
				bgcolor: combineRgb(r, g, b),
			},
			steps: [
				{
					down: [
						{
							actionId: 'execute_command',
							options: {
								slug: cmd.slug,
							},
						},
					],
					up: [],
				},
			],
			feedbacks: [
				{
					feedbackId: 'connection_status',
					options: {},
					style: {
						bgcolor: combineRgb(100, 100, 100),
					},
					isInverted: true,
				},
			],
		}
	}

	// Add a status preset
	presets['status'] = {
		type: 'button',
		category: 'Status',
		name: 'Connection Status',
		style: {
			text: 'Sermon\\nHelper',
			size: 'auto',
			color: combineRgb(255, 255, 255),
			bgcolor: combineRgb(100, 0, 0),
		},
		steps: [
			{
				down: [
					{
						actionId: 'refresh_commands',
						options: {},
					},
				],
				up: [],
			},
		],
		feedbacks: [
			{
				feedbackId: 'connection_status',
				options: {},
				style: {
					bgcolor: combineRgb(0, 100, 0),
				},
			},
		],
	}

	// PPT Selector Presets - Digit Buttons
	for (let digit = 0; digit <= 9; digit++) {
		presets[`ppt_digit_${digit}`] = {
			type: 'button',
			category: 'PPT Selector',
			name: `PPT: Digit ${digit}`,
			style: {
				text: digit.toString(),
				size: '44',
				color: combineRgb(255, 255, 255),
				bgcolor: combineRgb(59, 130, 246), // Blue
			},
			steps: [
				{
					down: [
						{
							actionId: 'ppt_digit',
							options: {
								digit: digit.toString(),
							},
						},
					],
					up: [],
				},
			],
			feedbacks: [
				{
					feedbackId: 'connection_status',
					options: {},
					style: {
						bgcolor: combineRgb(100, 100, 100),
					},
					isInverted: true,
				},
			],
		}
	}

	// PPT Selector - Control Buttons
	presets['ppt_backspace'] = {
		type: 'button',
		category: 'PPT Selector',
		name: 'PPT: Backspace',
		style: {
			text: '⌫',
			size: '44',
			color: combineRgb(255, 255, 255),
			bgcolor: combineRgb(239, 68, 68), // Red
		},
		steps: [
			{
				down: [
					{
						actionId: 'ppt_backspace',
						options: {},
					},
				],
				up: [],
			},
		],
		feedbacks: [],
	}

	presets['ppt_clear'] = {
		type: 'button',
		category: 'PPT Selector',
		name: 'PPT: Clear',
		style: {
			text: 'CLR',
			size: '18',
			color: combineRgb(255, 255, 255),
			bgcolor: combineRgb(245, 158, 11), // Amber
		},
		steps: [
			{
				down: [
					{
						actionId: 'ppt_clear',
						options: {},
					},
				],
				up: [],
			},
		],
		feedbacks: [
			{
				feedbackId: 'ppt_filter_active',
				options: {},
				style: {
					bgcolor: combineRgb(220, 120, 0),
				},
			},
		],
	}

	presets['ppt_refresh'] = {
		type: 'button',
		category: 'PPT Selector',
		name: 'PPT: Refresh',
		style: {
			text: '↻',
			size: '44',
			color: combineRgb(255, 255, 255),
			bgcolor: combineRgb(107, 114, 128), // Gray
		},
		steps: [
			{
				down: [
					{
						actionId: 'ppt_refresh',
						options: {},
					},
				],
				up: [],
			},
		],
		feedbacks: [],
	}

	// PPT Selector - Display Slot Buttons
	for (let slot = 1; slot <= 5; slot++) {
		presets[`ppt_slot_${slot}`] = {
			type: 'button',
			category: 'PPT Selector',
			name: `PPT: Slot ${slot}`,
			style: {
				text: `$(sermon-helper:ppt_slot_${slot}_name)`,
				size: '14',
				color: combineRgb(255, 255, 255),
				bgcolor: combineRgb(100, 100, 100), // Gray when empty
			},
			steps: [
				{
					down: [
						{
							actionId: 'ppt_select_slot',
							options: {
								slot: slot.toString(),
								startPresenter: true,
							},
						},
					],
					up: [],
				},
			],
			feedbacks: [
				{
					feedbackId: 'ppt_slot_has_file',
					options: {
						slot: slot.toString(),
					},
					style: {
						bgcolor: combineRgb(0, 150, 0), // Green when file present
					},
				},
				{
					feedbackId: 'connection_status',
					options: {},
					style: {
						bgcolor: combineRgb(80, 80, 80),
					},
					isInverted: true,
				},
			],
		}
	}

	// PPT Selector - Filter Status Display
	presets['ppt_filter_status'] = {
		type: 'button',
		category: 'PPT Selector',
		name: 'PPT: Filter Status',
		style: {
			text: 'Filter:\\n$(sermon-helper:ppt_filter)\\n$(sermon-helper:ppt_match_count) files',
			size: '14',
			color: combineRgb(255, 255, 255),
			bgcolor: combineRgb(50, 50, 50),
		},
		steps: [
			{
				down: [],
				up: [],
			},
		],
		feedbacks: [
			{
				feedbackId: 'ppt_filter_active',
				options: {},
				style: {
					bgcolor: combineRgb(100, 50, 0),
				},
			},
		],
	}

	// PPT Selector - Current Folder Display
	presets['ppt_folder_display'] = {
		type: 'button',
		category: 'PPT Selector',
		name: 'PPT: Current Folder',
		style: {
			text: '📁\\n$(sermon-helper:ppt_folder_name)',
			size: '14',
			color: combineRgb(255, 255, 255),
			bgcolor: combineRgb(70, 70, 70),
		},
		steps: [
			{
				down: [],
				up: [],
			},
		],
		feedbacks: [],
	}

	return presets
}
