import WebSocket from 'ws'
import type { ModuleConfig, RfIrCommand, ApiHealthResponse, PptFolder, PptFile, PptFilesResponse } from './types.js'

export class SermonHelperApi {
	private config: ModuleConfig
	private ws: WebSocket | null = null
	private wsReconnectTimer: ReturnType<typeof setTimeout> | null = null
	private wsPingTimer: ReturnType<typeof setInterval> | null = null
	private onCommandExecuted?: (slug: string, success: boolean) => void
	private onConnectionChange?: (connected: boolean) => void
	private onPptFoldersChanged?: (folders: PptFolder[]) => void
	private onPptFileOpened?: (fileName: string, success: boolean, presenterStarted: boolean) => void

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

	setCallbacks(callbacks: {
		onCommandExecuted?: (slug: string, success: boolean) => void
		onConnectionChange?: (connected: boolean) => void
		onPptFoldersChanged?: (folders: PptFolder[]) => void
		onPptFileOpened?: (fileName: string, success: boolean, presenterStarted: boolean) => void
	}): void {
		this.onCommandExecuted = callbacks.onCommandExecuted
		this.onConnectionChange = callbacks.onConnectionChange
		this.onPptFoldersChanged = callbacks.onPptFoldersChanged
		this.onPptFileOpened = callbacks.onPptFileOpened
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
