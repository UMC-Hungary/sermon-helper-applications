import WebSocket from 'ws'
import type { ModuleConfig, RfIrCommand, ApiHealthResponse, PptFolder, PptFile, PptFilesResponse, PresentationStatus } from './types.js'

export class SermonHelperApi {
	private config: ModuleConfig
	private ws: WebSocket | null = null
	private wsReconnectTimer: ReturnType<typeof setTimeout> | null = null
	private wsPingTimer: ReturnType<typeof setInterval> | null = null
	private onCommandExecuted?: (slug: string, success: boolean) => void
	private onConnectionChange?: (connected: boolean) => void
	private onPptFoldersChanged?: (folders: PptFolder[]) => void
	private onPptFileOpened?: (fileName: string, success: boolean, presenterStarted: boolean) => void
	private onPresentationStatusChanged?: (status: PresentationStatus) => void

	constructor(config: ModuleConfig) {
		this.config = config
	}

	updateConfig(config: ModuleConfig): void {
		this.config = config
	}

	private get baseUrl(): string {
		return `http://${this.config.host}:${this.config.port}`
	}

	private get wsUrl(): string {
		return `ws://${this.config.host}:${this.config.port}/ws`
	}

	private get headers(): Record<string, string> {
		const headers: Record<string, string> = {
			'Content-Type': 'application/json',
		}
		if (this.config.authToken) {
			headers['Authorization'] = `Bearer ${this.config.authToken}`
		}
		return headers
	}

	async checkHealth(): Promise<boolean> {
		try {
			const response = await fetch(`${this.baseUrl}/api/v1/health`, {
				headers: this.headers,
				signal: AbortSignal.timeout(5000),
			})
			if (!response.ok) return false
			const data = (await response.json()) as ApiHealthResponse
			return data.status === 'ok'
		} catch {
			return false
		}
	}

	async getCommands(): Promise<RfIrCommand[]> {
		try {
			const response = await fetch(`${this.baseUrl}/api/v1/rfir/commands`, {
				headers: this.headers,
				signal: AbortSignal.timeout(10000),
			})
			if (!response.ok) {
				throw new Error(`HTTP ${response.status}`)
			}
			const data = await response.json() as { success: boolean; data: RfIrCommand[] | null; error: string | null }
			// API returns { success, data: [...], error }
			if (data.success && Array.isArray(data.data)) {
				return data.data
			}
			return []
		} catch (error) {
			console.error('Failed to fetch commands:', error)
			return []
		}
	}

	async executeCommand(slug: string): Promise<{ success: boolean; error?: string }> {
		try {
			const response = await fetch(`${this.baseUrl}/api/v1/rfir/commands/${encodeURIComponent(slug)}/execute`, {
				method: 'POST',
				headers: this.headers,
				signal: AbortSignal.timeout(10000),
			})
			const data = (await response.json()) as { success: boolean; error?: string }
			return data
		} catch (error) {
			return {
				success: false,
				error: error instanceof Error ? error.message : 'Unknown error',
			}
		}
	}

	// PPT API Methods

	async getPptFolders(): Promise<PptFolder[]> {
		try {
			const response = await fetch(`${this.baseUrl}/api/v1/ppt/folders`, {
				headers: this.headers,
				signal: AbortSignal.timeout(10000),
			})
			if (!response.ok) {
				throw new Error(`HTTP ${response.status}`)
			}
			const data = await response.json() as { success: boolean; data: { folders: PptFolder[] } | null }
			if (data.success && data.data?.folders) {
				return data.data.folders
			}
			return []
		} catch (error) {
			console.error('Failed to fetch PPT folders:', error)
			return []
		}
	}

	async getPptFiles(folderId: string, filter?: string): Promise<PptFilesResponse> {
		try {
			const params = new URLSearchParams({ folder_id: folderId })
			if (filter) {
				params.set('filter', filter)
			}
			const response = await fetch(`${this.baseUrl}/api/v1/ppt/files?${params.toString()}`, {
				headers: this.headers,
				signal: AbortSignal.timeout(10000),
			})
			if (!response.ok) {
				throw new Error(`HTTP ${response.status}`)
			}
			const data = await response.json() as { success: boolean; data: PptFilesResponse | null }
			if (data.success && data.data) {
				return data.data
			}
			return { files: [], total: 0, filter: null }
		} catch (error) {
			console.error('Failed to fetch PPT files:', error)
			return { files: [], total: 0, filter: null }
		}
	}

	async openPptFile(filePath: string, startPresenter = true): Promise<{ success: boolean; presenterStarted: boolean; error?: string }> {
		try {
			const response = await fetch(`${this.baseUrl}/api/v1/ppt/open`, {
				method: 'POST',
				headers: this.headers,
				body: JSON.stringify({
					filePath: filePath,
					startPresenter: startPresenter,
				}),
				signal: AbortSignal.timeout(15000), // Longer timeout for opening + presenter mode
			})
			const data = await response.json() as { success: boolean; data?: { presenter_started?: boolean }; error?: string }
			return {
				success: data.success,
				presenterStarted: data.data?.presenter_started ?? false,
				error: data.error,
			}
		} catch (error) {
			return {
				success: false,
				presenterStarted: false,
				error: error instanceof Error ? error.message : 'Unknown error',
			}
		}
	}

	// Presentation Control API Methods

	async presentationOpen(filePath: string, startPresenter = true): Promise<{ success: boolean; error?: string }> {
		try {
			const response = await fetch(`${this.baseUrl}/api/v1/presentation/open`, {
				method: 'POST',
				headers: this.headers,
				body: JSON.stringify({ filePath, startPresenter }),
				signal: AbortSignal.timeout(15000),
			})
			return (await response.json()) as { success: boolean; error?: string }
		} catch (error) {
			return { success: false, error: error instanceof Error ? error.message : 'Unknown error' }
		}
	}

	async presentationStart(fromSlide?: number): Promise<{ success: boolean; error?: string }> {
		try {
			const body: Record<string, unknown> = {}
			if (fromSlide !== undefined) body.fromSlide = fromSlide
			const response = await fetch(`${this.baseUrl}/api/v1/presentation/start`, {
				method: 'POST',
				headers: this.headers,
				body: JSON.stringify(body),
				signal: AbortSignal.timeout(10000),
			})
			return (await response.json()) as { success: boolean; error?: string }
		} catch (error) {
			return { success: false, error: error instanceof Error ? error.message : 'Unknown error' }
		}
	}

	async presentationStop(): Promise<{ success: boolean; error?: string }> {
		try {
			const response = await fetch(`${this.baseUrl}/api/v1/presentation/stop`, {
				method: 'POST',
				headers: this.headers,
				signal: AbortSignal.timeout(10000),
			})
			return (await response.json()) as { success: boolean; error?: string }
		} catch (error) {
			return { success: false, error: error instanceof Error ? error.message : 'Unknown error' }
		}
	}

	async presentationNext(): Promise<{ success: boolean; error?: string }> {
		try {
			const response = await fetch(`${this.baseUrl}/api/v1/presentation/next`, {
				method: 'POST',
				headers: this.headers,
				signal: AbortSignal.timeout(10000),
			})
			return (await response.json()) as { success: boolean; error?: string }
		} catch (error) {
			return { success: false, error: error instanceof Error ? error.message : 'Unknown error' }
		}
	}

	async presentationPrevious(): Promise<{ success: boolean; error?: string }> {
		try {
			const response = await fetch(`${this.baseUrl}/api/v1/presentation/previous`, {
				method: 'POST',
				headers: this.headers,
				signal: AbortSignal.timeout(10000),
			})
			return (await response.json()) as { success: boolean; error?: string }
		} catch (error) {
			return { success: false, error: error instanceof Error ? error.message : 'Unknown error' }
		}
	}

	async presentationGoto(slideNumber: number): Promise<{ success: boolean; error?: string }> {
		try {
			const response = await fetch(`${this.baseUrl}/api/v1/presentation/goto`, {
				method: 'POST',
				headers: this.headers,
				body: JSON.stringify({ slideNumber }),
				signal: AbortSignal.timeout(10000),
			})
			return (await response.json()) as { success: boolean; error?: string }
		} catch (error) {
			return { success: false, error: error instanceof Error ? error.message : 'Unknown error' }
		}
	}

	async presentationFirst(): Promise<{ success: boolean; error?: string }> {
		try {
			const response = await fetch(`${this.baseUrl}/api/v1/presentation/first`, {
				method: 'POST',
				headers: this.headers,
				signal: AbortSignal.timeout(10000),
			})
			return (await response.json()) as { success: boolean; error?: string }
		} catch (error) {
			return { success: false, error: error instanceof Error ? error.message : 'Unknown error' }
		}
	}

	async presentationLast(): Promise<{ success: boolean; error?: string }> {
		try {
			const response = await fetch(`${this.baseUrl}/api/v1/presentation/last`, {
				method: 'POST',
				headers: this.headers,
				signal: AbortSignal.timeout(10000),
			})
			return (await response.json()) as { success: boolean; error?: string }
		} catch (error) {
			return { success: false, error: error instanceof Error ? error.message : 'Unknown error' }
		}
	}

	async presentationBlank(): Promise<{ success: boolean; error?: string }> {
		try {
			const response = await fetch(`${this.baseUrl}/api/v1/presentation/blank`, {
				method: 'POST',
				headers: this.headers,
				signal: AbortSignal.timeout(10000),
			})
			return (await response.json()) as { success: boolean; error?: string }
		} catch (error) {
			return { success: false, error: error instanceof Error ? error.message : 'Unknown error' }
		}
	}

	async presentationUnblank(): Promise<{ success: boolean; error?: string }> {
		try {
			const response = await fetch(`${this.baseUrl}/api/v1/presentation/unblank`, {
				method: 'POST',
				headers: this.headers,
				signal: AbortSignal.timeout(10000),
			})
			return (await response.json()) as { success: boolean; error?: string }
		} catch (error) {
			return { success: false, error: error instanceof Error ? error.message : 'Unknown error' }
		}
	}

	async presentationStatus(): Promise<PresentationStatus | null> {
		try {
			const response = await fetch(`${this.baseUrl}/api/v1/presentation/status`, {
				headers: this.headers,
				signal: AbortSignal.timeout(5000),
			})
			if (!response.ok) return null
			const data = (await response.json()) as { success: boolean; data?: PresentationStatus }
			if (data.success && data.data) return data.data
			return null
		} catch {
			return null
		}
	}

	setCallbacks(callbacks: {
		onCommandExecuted?: (slug: string, success: boolean) => void
		onConnectionChange?: (connected: boolean) => void
		onPptFoldersChanged?: (folders: PptFolder[]) => void
		onPptFileOpened?: (fileName: string, success: boolean, presenterStarted: boolean) => void
		onPresentationStatusChanged?: (status: PresentationStatus) => void
	}): void {
		this.onCommandExecuted = callbacks.onCommandExecuted
		this.onConnectionChange = callbacks.onConnectionChange
		this.onPptFoldersChanged = callbacks.onPptFoldersChanged
		this.onPptFileOpened = callbacks.onPptFileOpened
		this.onPresentationStatusChanged = callbacks.onPresentationStatusChanged
	}

	connectWebSocket(): void {
		if (this.ws) {
			this.disconnectWebSocket()
		}

		try {
			const wsOptions: WebSocket.ClientOptions = {}
			if (this.config.authToken) {
				wsOptions.headers = {
					Authorization: `Bearer ${this.config.authToken}`,
				}
			}
			this.ws = new WebSocket(this.wsUrl, wsOptions)

			this.ws.on('open', () => {
				console.log('WebSocket connected')
				this.onConnectionChange?.(true)
				this.startPingInterval()
			})

			this.ws.on('close', () => {
				console.log('WebSocket disconnected')
				this.stopPingInterval()
				this.onConnectionChange?.(false)
				this.scheduleReconnect()
			})

			this.ws.on('error', (error) => {
				console.error('WebSocket error:', error)
			})

			this.ws.on('message', (data) => {
				try {
					const message = JSON.parse(data.toString()) as {
						type: string
						data?: {
							slug?: string
							success?: boolean
							folders?: PptFolder[]
							file_name?: string
							presenter_started?: boolean
							// Presentation status fields
							app?: string
							app_running?: boolean
							slideshow_active?: boolean
							current_slide?: number | null
							total_slides?: number | null
							current_slide_title?: string | null
							blanked?: boolean
						}
					}

					if (message.type === 'ping') {
						// Respond to server ping with pong
						if (this.ws && this.ws.readyState === WebSocket.OPEN) {
							this.ws.send(JSON.stringify({ type: 'pong' }))
						}
					} else if (message.type === 'rf_ir_command_executed' && message.data?.slug !== undefined) {
						this.onCommandExecuted?.(message.data.slug, message.data.success ?? false)
					} else if (message.type === 'ppt_folders_changed' && message.data?.folders) {
						this.onPptFoldersChanged?.(message.data.folders)
					} else if (message.type === 'ppt_file_opened' && message.data?.file_name !== undefined) {
						this.onPptFileOpened?.(
							message.data.file_name,
							message.data.success ?? false,
							message.data.presenter_started ?? false
						)
					} else if (message.type === 'presentation_status_changed' && message.data) {
						this.onPresentationStatusChanged?.({
							app: message.data.app ?? null,
							appRunning: message.data.app_running ?? false,
							slideshowActive: message.data.slideshow_active ?? false,
							currentSlide: message.data.current_slide ?? null,
							totalSlides: message.data.total_slides ?? null,
							currentSlideTitle: message.data.current_slide_title ?? null,
							blanked: message.data.blanked ?? false,
						})
					}
				} catch {
					// Ignore parse errors
				}
			})
		} catch (error) {
			console.error('Failed to connect WebSocket:', error)
			this.scheduleReconnect()
		}
	}

	private scheduleReconnect(): void {
		if (this.wsReconnectTimer) {
			clearTimeout(this.wsReconnectTimer)
		}
		this.wsReconnectTimer = setTimeout(() => {
			this.connectWebSocket()
		}, 5000)
	}

	private startPingInterval(): void {
		this.stopPingInterval()
		// Send ping every 15 seconds to keep connection alive
		this.wsPingTimer = setInterval(() => {
			if (this.ws && this.ws.readyState === WebSocket.OPEN) {
				try {
					this.ws.send(JSON.stringify({ type: 'ping' }))
				} catch {
					// Ignore send errors, connection will be handled by close event
				}
			}
		}, 15000)
	}

	private stopPingInterval(): void {
		if (this.wsPingTimer) {
			clearInterval(this.wsPingTimer)
			this.wsPingTimer = null
		}
	}

	disconnectWebSocket(): void {
		this.stopPingInterval()
		if (this.wsReconnectTimer) {
			clearTimeout(this.wsReconnectTimer)
			this.wsReconnectTimer = null
		}
		if (this.ws) {
			this.ws.removeAllListeners() // Prevent reconnect
			this.ws.close()
			this.ws = null
		}
	}
}
