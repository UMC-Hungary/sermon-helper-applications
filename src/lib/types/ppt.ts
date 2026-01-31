/**
 * PPT folder configuration for Companion integration
 */
export interface PptFolder {
	id: string;
	path: string;
	name: string;
}

/**
 * PPT settings stored in app settings
 */
export interface PptSettings {
	folders: PptFolder[];
}

/**
 * Default PPT settings
 */
export const DEFAULT_PPT_SETTINGS: PptSettings = {
	folders: [],
};
