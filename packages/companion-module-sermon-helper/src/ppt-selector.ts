import type { PptFolder, PptFile } from './types.js'
import type { SermonHelperApi } from './api.js'

export interface PptSelectorState {
	currentFilter: string
	matchingFiles: PptFile[]
	selectedFolderId: string | null
	folders: PptFolder[]
	lastOpenedFile: string | null
}

export class PptSelector {
	private api: SermonHelperApi
	private state: PptSelectorState
	private onStateChange?: (state: PptSelectorState) => void

	constructor(api: SermonHelperApi) {
		this.api = api
		this.state = {
			currentFilter: '',
			matchingFiles: [],
			selectedFolderId: null,
			folders: [],
			lastOpenedFile: null,
		}
	}

	getState(): PptSelectorState {
		return { ...this.state }
	}

	setOnStateChange(callback: (state: PptSelectorState) => void): void {
		this.onStateChange = callback
	}

	private notifyChange(): void {
		this.onStateChange?.(this.getState())
	}

	/**
	 * Refresh folders from the API
	 */
	async refreshFolders(): Promise<void> {
		const folders = await this.api.getPptFolders()
		this.state.folders = folders

		// If no folder is selected and we have folders, select the first one
		if (!this.state.selectedFolderId && folders.length > 0) {
			this.state.selectedFolderId = folders[0].id
			await this.refreshFiles()
		}

		this.notifyChange()
	}

	/**
	 * Update folders from WebSocket event
	 */
	updateFolders(folders: PptFolder[]): void {
		this.state.folders = folders

		// If selected folder was removed, clear selection
		if (this.state.selectedFolderId && !folders.find((f) => f.id === this.state.selectedFolderId)) {
			this.state.selectedFolderId = folders.length > 0 ? folders[0].id : null
			this.state.matchingFiles = []
			this.state.currentFilter = ''
		}

		this.notifyChange()
	}

	/**
	 * Refresh files from the selected folder
	 */
	async refreshFiles(): Promise<void> {
		if (!this.state.selectedFolderId) {
			this.state.matchingFiles = []
			this.notifyChange()
			return
		}

		const response = await this.api.getPptFiles(
			this.state.selectedFolderId,
			this.state.currentFilter || undefined
		)
		this.state.matchingFiles = response.files
		this.notifyChange()
	}

	/**
	 * Select a folder by ID
	 */
	async selectFolder(folderId: string): Promise<void> {
		const folder = this.state.folders.find((f) => f.id === folderId)
		if (!folder) return

		this.state.selectedFolderId = folderId
		this.state.currentFilter = ''
		await this.refreshFiles()
	}

	/**
	 * Append a digit to the filter
	 */
	async appendDigit(digit: number): Promise<void> {
		if (digit < 0 || digit > 9) return

		this.state.currentFilter += digit.toString()
		await this.refreshFiles()
	}

	/**
	 * Remove the last digit from the filter (backspace)
	 */
	async backspace(): Promise<void> {
		if (this.state.currentFilter.length === 0) return

		this.state.currentFilter = this.state.currentFilter.slice(0, -1)
		await this.refreshFiles()
	}

	/**
	 * Clear the filter completely
	 */
	async clearFilter(): Promise<void> {
		if (this.state.currentFilter === '') return

		this.state.currentFilter = ''
		await this.refreshFiles()
	}

	/**
	 * Get the file at a specific slot (0-indexed)
	 */
	getFileAtSlot(slot: number): PptFile | null {
		if (slot < 0 || slot >= this.state.matchingFiles.length) {
			return null
		}
		return this.state.matchingFiles[slot]
	}

	/**
	 * Open the file at a specific slot
	 */
	async openFileAtSlot(slot: number, startPresenter = true): Promise<{ success: boolean; error?: string }> {
		const file = this.getFileAtSlot(slot)
		if (!file) {
			return { success: false, error: 'No file at this slot' }
		}

		const result = await this.api.openPptFile(file.path, startPresenter)

		if (result.success) {
			this.state.lastOpenedFile = file.name
		}

		// Always reset filter after a selection attempt — the user made their choice
		this.state.currentFilter = ''
		await this.refreshFiles()

		return result
	}

	/**
	 * Get the display name for a slot (truncated if needed)
	 */
	getSlotDisplayName(slot: number, maxLength = 15): string {
		const file = this.getFileAtSlot(slot)
		if (!file) return ''

		// Remove extension for display
		const nameWithoutExt = file.name.replace(/\.(pptx?|odp)$/i, '')

		if (nameWithoutExt.length <= maxLength) {
			return nameWithoutExt
		}

		return nameWithoutExt.slice(0, maxLength - 1) + '…'
	}

	/**
	 * Get the selected folder name
	 */
	getSelectedFolderName(): string {
		if (!this.state.selectedFolderId) return ''
		const folder = this.state.folders.find((f) => f.id === this.state.selectedFolderId)
		return folder?.name || ''
	}

	/**
	 * Get folder choices for action dropdown
	 */
	getFolderChoices(): Array<{ id: string; label: string }> {
		return this.state.folders.map((f) => ({
			id: f.id,
			label: f.name,
		}))
	}
}
