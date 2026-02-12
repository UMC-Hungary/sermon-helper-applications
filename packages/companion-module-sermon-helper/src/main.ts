import {
	InstanceBase,
	InstanceStatus,
	runEntrypoint,
	type SomeCompanionConfigField,
} from '@companion-module/base'
import { GetConfigFields, GetDefaultConfig } from './config.js'
import { GetActions } from './actions.js'
import { GetFeedbacks } from './feedbacks.js'
import { GetPresets } from './presets.js'
import { GetVariableDefinitions, GetDefaultVariableValues } from './variables.js'
import { SermonHelperApi } from './api.js'
import { PptSelector } from './ppt-selector.js'
import type { ModuleConfig, RfIrCommand, PresentationStatus } from './types.js'

export class ModuleInstance extends InstanceBase<ModuleConfig> {
	public config: ModuleConfig = GetDefaultConfig()
	public api: SermonHelperApi
	public commands: RfIrCommand[] = []
	public isConnected = false
	public pptSelector: PptSelector
	public presentationStatus: PresentationStatus | null = null

	private pollTimer: ReturnType<typeof setInterval> | null = null
	private presentationPollTimer: ReturnType<typeof setInterval> | null = null

	constructor(internal: unknown) {
		super(internal as ConstructorParameters<typeof InstanceBase>[0])
		this.api = new SermonHelperApi(this.config)
		this.pptSelector = new PptSelector(this.api)
	}

	async init(config: ModuleConfig): Promise<void> {
		this.config = config
		this.api.updateConfig(config)

		// Set up PptSelector state change callback
		this.pptSelector.setOnStateChange((state) => {
			this.setVariableValues({
				ppt_filter: state.currentFilter,
				ppt_match_count: state.matchingFiles.length.toString(),
				ppt_folder_name: this.pptSelector.getSelectedFolderName(),
				ppt_slot_1_name: this.pptSelector.getSlotDisplayName(0),
				ppt_slot_2_name: this.pptSelector.getSlotDisplayName(1),
				ppt_slot_3_name: this.pptSelector.getSlotDisplayName(2),
				ppt_slot_4_name: this.pptSelector.getSlotDisplayName(3),
				ppt_slot_5_name: this.pptSelector.getSlotDisplayName(4),
			})
			this.checkFeedbacks('ppt_slot_has_file', 'ppt_filter_active')
		})

		this.api.setCallbacks({
			onCommandExecuted: (slug, success) => {
				this.log('debug', `Command executed via WS: ${slug} (${success ? 'success' : 'failed'})`)
				if (success) {
					const cmd = this.commands.find((c) => c.slug === slug)
					if (cmd) {
						this.setVariableValues({ last_command: cmd.name })
					}
				}
				this.checkFeedbacks('command_available')
			},
			onConnectionChange: (connected) => {
				this.isConnected = connected
				this.updateStatus(connected ? InstanceStatus.Ok : InstanceStatus.Disconnected)
				this.setVariableValues({
					connection_status: connected ? 'Connected' : 'Disconnected',
				})
				this.checkFeedbacks('connection_status')
			},
			onPptFoldersChanged: (folders) => {
				this.log('debug', `PPT folders changed via WS: ${folders.length} folders`)
				this.pptSelector.updateFolders(folders)
				this.updateDefinitions() // Refresh folder dropdown in actions
			},
			onPptFileOpened: (fileName, success, presenterStarted) => {
				this.log('info', `PPT file opened: ${fileName} (success: ${success}, presenter: ${presenterStarted})`)
				if (success) {
					this.setVariableValues({ ppt_last_opened: fileName })
				}
			},
			onPresentationStatusChanged: (status) => {
				this.log('debug', `Presentation status: ${status.app} slideshow=${status.slideshowActive} slide=${status.currentSlide}/${status.totalSlides}`)
				this.presentationStatus = status
				this.setVariableValues({
					ppt_current_slide: status.currentSlide?.toString() ?? '-',
					ppt_total_slides: status.totalSlides?.toString() ?? '-',
					ppt_slideshow_active: status.slideshowActive ? 'ON' : 'OFF',
					ppt_app: status.app ?? 'None',
					ppt_blanked: status.blanked ? 'YES' : 'NO',
				})
				this.checkFeedbacks('slideshow_active', 'presentation_blanked')
			},
		})

		// Initial setup
		this.updateStatus(InstanceStatus.Connecting)

		// Check connection and load initial data
		await this.checkConnection()
		await this.refreshCommands()

		// Load PPT folders
		await this.pptSelector.refreshFolders()

		// Connect WebSocket for real-time updates
		this.api.connectWebSocket()

		// Fetch initial presentation status
		await this.refreshPresentationStatus()

		// Start polling for command updates and presentation status
		this.startPolling()
		this.startPresentationPolling()

		// Initialize definitions
		this.updateDefinitions()
	}

	async destroy(): Promise<void> {
		this.stopPolling()
		this.stopPresentationPolling()
		this.api.disconnectWebSocket()
	}

	async configUpdated(config: ModuleConfig): Promise<void> {
		const hostChanged = this.config.host !== config.host || this.config.port !== config.port
		this.config = config
		this.api.updateConfig(config)

		if (hostChanged) {
			this.api.disconnectWebSocket()
			this.updateStatus(InstanceStatus.Connecting)
			await this.checkConnection()
			await this.refreshCommands()
			await this.pptSelector.refreshFolders()
			this.api.connectWebSocket()
			await this.refreshPresentationStatus()
		}

		// Restart polling with new interval
		this.stopPolling()
		this.stopPresentationPolling()
		this.startPolling()
		this.startPresentationPolling()

		this.updateDefinitions()
	}

	getConfigFields(): SomeCompanionConfigField[] {
		return GetConfigFields()
	}

	private async checkConnection(): Promise<void> {
		// Only check HTTP health if WebSocket is not connected
		// WebSocket onConnectionChange callback is the primary source of truth
		if (!this.isConnected) {
			const healthy = await this.api.checkHealth()
			if (healthy) {
				// HTTP is healthy but WebSocket is not connected - try reconnecting WebSocket
				this.log('debug', 'HTTP healthy but WebSocket disconnected, triggering reconnect')
			}
		}
	}

	public async refreshCommands(): Promise<void> {
		const commands = await this.api.getCommands()
		this.commands = commands

		this.setVariableValues({
			command_count: commands.length,
		})

		// Update all definitions with new command lists
		this.updateDefinitions()

		this.log('info', `Loaded ${commands.length} commands`)
	}

	private updateDefinitions(): void {
		this.setVariableDefinitions(GetVariableDefinitions(this))
		this.setVariableValues(GetDefaultVariableValues(this))
		this.setActionDefinitions(GetActions(this))
		this.setFeedbackDefinitions(GetFeedbacks(this))
		this.setPresetDefinitions(GetPresets(this))
	}

	private startPolling(): void {
		if (this.pollTimer) {
			clearInterval(this.pollTimer)
		}

		this.pollTimer = setInterval(async () => {
			await this.checkConnection()
			if (this.isConnected) {
				await this.refreshCommands()
			}
		}, this.config.pollInterval)
	}

	private stopPolling(): void {
		if (this.pollTimer) {
			clearInterval(this.pollTimer)
			this.pollTimer = null
		}
	}

	public async refreshPresentationStatus(): Promise<void> {
		const status = await this.api.presentationStatus()
		if (status) {
			this.presentationStatus = status
			this.setVariableValues({
				ppt_current_slide: status.currentSlide?.toString() ?? '-',
				ppt_total_slides: status.totalSlides?.toString() ?? '-',
				ppt_slideshow_active: status.slideshowActive ? 'ON' : 'OFF',
				ppt_app: status.app ?? 'None',
				ppt_blanked: status.blanked ? 'YES' : 'NO',
			})
			this.checkFeedbacks('slideshow_active', 'presentation_blanked')
		}
	}

	private startPresentationPolling(): void {
		if (this.presentationPollTimer) {
			clearInterval(this.presentationPollTimer)
		}
		// Poll presentation status every 2 seconds for responsive slide tracking
		this.presentationPollTimer = setInterval(async () => {
			if (this.isConnected) {
				await this.refreshPresentationStatus()
			}
		}, 2000)
	}

	private stopPresentationPolling(): void {
		if (this.presentationPollTimer) {
			clearInterval(this.presentationPollTimer)
			this.presentationPollTimer = null
		}
	}
}

runEntrypoint(ModuleInstance, [])
