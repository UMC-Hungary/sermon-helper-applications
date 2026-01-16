import PptxGenJS from 'pptxgenjs';
import type { BibleVerse } from '$lib/types/bible';

const THRESHOLD = 250;

interface GeneratePptxOptions {
	reference: string; // e.g., "Jn 3,16"
	verses: BibleVerse[];
	type: 'textus' | 'leckio';
}

function mapTermType(type: 'textus' | 'leckio'): string {
	return type === 'textus' ? 'Textus' : 'Lekció';
}

/**
 * Generate a PPTX presentation from Bible verses
 * Returns a Blob that can be saved or downloaded
 */
export async function generatePptx(options: GeneratePptxOptions): Promise<Blob> {
	const { reference, verses, type } = options;

	const pres = new PptxGenJS();

	// Set up 16:9 layout
	pres.defineLayout({ name: 'A3', width: 16, height: 9 });
	pres.layout = 'A3';
	pres.theme = { headFontFace: 'Helvetica Neue', bodyFontFace: 'Helvetica' };

	const textboxOpts: PptxGenJS.TextPropsOptions = {
		x: 1,
		y: 1,
		w: 14,
		h: 7,
		color: 'FFFFFF',
		fontSize: 56,
		bold: true,
		align: 'center',
		valign: 'middle',
	};

	// Combine verses into parts based on threshold
	const parts = verses
		.map((v) => v.text)
		.reduce<string[]>((acc, curr) => {
			const last = acc.length > 0 ? acc[acc.length - 1] : '';
			const isOverflow = curr.length + last.length > THRESHOLD;
			const rest = acc.slice(0, -1);

			return [...rest, ...(isOverflow ? [last, curr] : [`${last} ${curr}`])];
		}, []);

	const totalParts = parts.length;

	// Title slide
	const titleSlide = pres.addSlide();
	titleSlide.background = { color: '000000' };
	titleSlide.addText(
		[
			{ text: `${reference} (${mapTermType(type)})`, options: { breakLine: true } },
			{
				text: `${new Date().toISOString().substring(0, 10)}`,
				options: { fontSize: 42, breakLine: false },
			},
		],
		textboxOpts
	);

	// Verse slides
	parts.forEach((text, i) => {
		const slide = pres.addSlide();
		slide.background = { color: '000000' };

		const pageIndicator = totalParts > 1 ? `${i + 1}/${totalParts}` : '';
		slide.addText(
			[
				{ text: text.trim(), options: { breakLine: true } },
				{
					text: `(${reference}) ${pageIndicator}`,
					options: { fontSize: 42, breakLine: false },
				},
			],
			textboxOpts
		);
	});

	// Empty closing slide
	const lastSlide = pres.addSlide();
	lastSlide.background = { color: '000000' };

	// Generate as blob
	const output = await pres.write({ outputType: 'blob' });
	return output as Blob;
}
