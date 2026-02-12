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

	// Presentation Control Presets
	presets['presentation_open'] = {
		type: 'button',
		category: 'Presentation Control',
		name: 'Open Presentation',
		style: {
			text: '📂 Open',
			size: 'auto',
			color: combineRgb(255, 255, 255),
			bgcolor: combineRgb(100, 65, 165), // Purple
		},
		steps: [
			{
				down: [
					{
						actionId: 'presentation_open',
						options: { filePath: '', startPresenter: true },
					},
				],
				up: [],
			},
		],
		feedbacks: [
			{
				feedbackId: 'connection_status',
				options: {},
				style: { bgcolor: combineRgb(100, 100, 100) },
				isInverted: true,
			},
		],
	}

	presets['presentation_on_off'] = {
		type: 'button',
		category: 'Presentation Control',
		name: 'Presentation ON/OFF',
		style: {
			text: 'Presentation\\nOFF',
			size: '14',
			color: combineRgb(255, 255, 255),
			bgcolor: combineRgb(100, 0, 0), // Red = OFF
		},
		steps: [
			{
				down: [],
				up: [],
			},
		],
		feedbacks: [
			{
				feedbackId: 'slideshow_active',
				options: {},
				style: {
					text: 'Presentation\\nON',
					bgcolor: combineRgb(0, 150, 0), // Green = ON
				},
			},
			{
				feedbackId: 'connection_status',
				options: {},
				style: { bgcolor: combineRgb(80, 80, 80) },
				isInverted: true,
			},
		],
	}

	presets['presentation_start'] = {
		type: 'button',
		category: 'Presentation Control',
		name: 'Start Slideshow',
		style: {
			text: '▶ Play',
			size: 'auto',
			color: combineRgb(255, 255, 255),
			bgcolor: combineRgb(0, 150, 0), // Green
		},
		steps: [
			{
				down: [
					{
						actionId: 'presentation_start',
						options: { fromSlide: 0 },
					},
				],
				up: [],
			},
		],
		feedbacks: [
			{
				feedbackId: 'connection_status',
				options: {},
				style: { bgcolor: combineRgb(100, 100, 100) },
				isInverted: true,
			},
		],
	}

	presets['presentation_stop'] = {
		type: 'button',
		category: 'Presentation Control',
		name: 'Stop Slideshow',
		style: {
			text: '⏹ Stop',
			size: 'auto',
			color: combineRgb(255, 255, 255),
			bgcolor: combineRgb(153, 0, 0), // Dark Red
		},
		steps: [
			{
				down: [
					{
						actionId: 'presentation_stop',
						options: {},
					},
				],
				up: [],
			},
		],
		feedbacks: [
			{
				feedbackId: 'slideshow_active',
				options: {},
				style: { bgcolor: combineRgb(200, 0, 0) },
			},
			{
				feedbackId: 'connection_status',
				options: {},
				style: { bgcolor: combineRgb(100, 100, 100) },
				isInverted: true,
			},
		],
	}

	presets['presentation_prev'] = {
		type: 'button',
		category: 'Presentation Control',
		name: 'Previous Slide',
		style: {
			text: '◀ Prev',
			size: 'auto',
			color: combineRgb(255, 255, 255),
			bgcolor: combineRgb(59, 130, 246), // Blue
		},
		steps: [
			{
				down: [
					{
						actionId: 'presentation_previous',
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
				style: { bgcolor: combineRgb(100, 100, 100) },
				isInverted: true,
			},
		],
	}

	presets['presentation_next'] = {
		type: 'button',
		category: 'Presentation Control',
		name: 'Next Slide',
		style: {
			text: '▶ Next',
			size: 'auto',
			color: combineRgb(255, 255, 255),
			bgcolor: combineRgb(59, 130, 246), // Blue
		},
		steps: [
			{
				down: [
					{
						actionId: 'presentation_next',
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
				style: { bgcolor: combineRgb(100, 100, 100) },
				isInverted: true,
			},
		],
	}

	presets['presentation_first'] = {
		type: 'button',
		category: 'Presentation Control',
		name: 'First Slide',
		style: {
			text: '⏮ First',
			size: 'auto',
			color: combineRgb(255, 255, 255),
			bgcolor: combineRgb(59, 130, 246), // Blue
		},
		steps: [
			{
				down: [
					{
						actionId: 'presentation_first',
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
				style: { bgcolor: combineRgb(100, 100, 100) },
				isInverted: true,
			},
		],
	}

	presets['presentation_last'] = {
		type: 'button',
		category: 'Presentation Control',
		name: 'Last Slide',
		style: {
			text: '⏭ Last',
			size: 'auto',
			color: combineRgb(255, 255, 255),
			bgcolor: combineRgb(59, 130, 246), // Blue
		},
		steps: [
			{
				down: [
					{
						actionId: 'presentation_last',
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
				style: { bgcolor: combineRgb(100, 100, 100) },
				isInverted: true,
			},
		],
	}

	presets['presentation_blank'] = {
		type: 'button',
		category: 'Presentation Control',
		name: 'Toggle Blank Screen',
		style: {
			text: '■ Blank',
			size: 'auto',
			color: combineRgb(255, 255, 255),
			bgcolor: combineRgb(50, 50, 50), // Dark Gray
		},
		steps: [
			{
				down: [
					{
						actionId: 'presentation_blank_toggle',
						options: {},
					},
				],
				up: [],
			},
		],
		feedbacks: [
			{
				feedbackId: 'presentation_blanked',
				options: {},
				style: { bgcolor: combineRgb(0, 0, 0) },
			},
			{
				feedbackId: 'connection_status',
				options: {},
				style: { bgcolor: combineRgb(100, 100, 100) },
				isInverted: true,
			},
		],
	}

	presets['presentation_goto'] = {
		type: 'button',
		category: 'Presentation Control',
		name: 'Go to Slide',
		style: {
			text: 'Go To\\nSlide',
			size: 'auto',
			color: combineRgb(255, 255, 255),
			bgcolor: combineRgb(100, 65, 165), // Purple
		},
		steps: [
			{
				down: [
					{
						actionId: 'presentation_goto',
						options: { slideNumber: 1 },
					},
				],
				up: [],
			},
		],
		feedbacks: [
			{
				feedbackId: 'connection_status',
				options: {},
				style: { bgcolor: combineRgb(100, 100, 100) },
				isInverted: true,
			},
		],
	}

	// Presentation Status Display
	presets['presentation_status'] = {
		type: 'button',
		category: 'Presentation Control',
		name: 'Slide Status',
		style: {
			text: '$(sermon-helper:ppt_current_slide)/$(sermon-helper:ppt_total_slides)',
			size: '18',
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
				feedbackId: 'slideshow_active',
				options: {},
				style: { bgcolor: combineRgb(0, 100, 0) },
			},
		],
	}

	return presets
}
