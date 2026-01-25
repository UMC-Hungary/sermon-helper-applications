/**
 * Slug generation utility for URL-safe identifiers
 */

/**
 * Generate a URL-safe slug from a name, ensuring uniqueness
 * @param name - The name to convert to a slug
 * @param existingSlugs - Array of existing slugs to avoid conflicts
 * @returns A unique URL-safe slug
 */
export function generateSlug(name: string, existingSlugs: string[] = []): string {
	// Convert to lowercase, replace spaces/special chars with hyphens
	let base = name
		.toLowerCase()
		.trim()
		.normalize('NFD')
		.replace(/[\u0300-\u036f]/g, '') // Remove diacritics
		.replace(/[^a-z0-9]+/g, '-') // Replace non-alphanumeric with hyphens
		.replace(/^-+|-+$/g, '') // Remove leading/trailing hyphens
		.replace(/-+/g, '-'); // Collapse multiple hyphens

	// Handle empty result
	if (!base) {
		base = 'command';
	}

	// Ensure uniqueness
	let slug = base;
	let counter = 1;
	while (existingSlugs.includes(slug)) {
		slug = `${base}-${counter}`;
		counter++;
	}
	return slug;
}

/**
 * Validate if a string is a valid slug
 * @param slug - The string to validate
 * @returns true if valid slug format
 */
export function isValidSlug(slug: string): boolean {
	return /^[a-z0-9]+(-[a-z0-9]+)*$/.test(slug);
}
