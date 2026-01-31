export interface ModuleConfig {
	host: string
	port: number
	authToken: string
	useAutoDiscovery: boolean
	pollInterval: number
}

export interface RfIrCommand {
	id: string
	name: string
	slug: string
	deviceId: string
	code: string
	type: 'ir' | 'rf'
	category: 'projector' | 'screen' | 'hvac' | 'lighting' | 'audio' | 'other'
}

export interface BroadlinkDevice {
	id: string
	name: string
	type: string
	model: string
	host: string
	mac: string
}

export interface ApiHealthResponse {
	status: string
	service: string
	version: string
}

export interface ApiCommandsResponse {
	commands: RfIrCommand[]
}

export interface ApiExecuteResponse {
	success: boolean
	error?: string
}

export interface WsMessage {
	type: string
	data?: unknown
}

// PPT Selector Types
export interface PptFolder {
	id: string
	path: string
	name: string
}

export interface PptFile {
	id: string
	name: string
	path: string
	folderId: string
}

export interface PptFilesResponse {
	files: PptFile[]
	total: number
	filter: string | null
}

export interface PptFoldersResponse {
	folders: PptFolder[]
}

export interface PptOpenResponse {
	success: boolean
	fileName: string
	presenterStarted: boolean
}

export interface DiscoveredServer {
	host: string
	port: number
	name: string
}

export const CATEGORY_COLORS: Record<string, number> = {
	projector: 0xdd614a, // Rosy Copper
	screen: 0xf48668, // Coral Glow
	hvac: 0xf4a698, // Powder Blush
	lighting: 0xc5c392, // Dry Sage
	audio: 0x73a580, // Muted Teal
	other: 0x808080, // Gray
}
