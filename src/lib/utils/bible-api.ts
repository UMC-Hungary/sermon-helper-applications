import { apiFetch } from '$lib/api/client.js';
import { BiblePassageSchema, BibleSuggestionsSchema } from '$lib/schemas/bible.js';
import type { BibleVerse, BibleTranslation } from '$lib/types/bible';
import type { BibleSuggestion } from '$lib/schemas/bible.js';

/**
 * Bible lookups go through the core, which talks to the upstream APIs. No Tauri
 * IPC and no CORS proxy: the same path works in the desktop app, a browser and
 * a remote client.
 */
class BibleApiService {
  async fetchVerses(
    reference: string,
    translation: BibleTranslation,
  ): Promise<{ verses: BibleVerse[]; label: string }> {
    const query = new URLSearchParams({ reference, translation });
    const passage = await apiFetch(`/api/bible/verses?${query}`, BiblePassageSchema);

    return {
      label: passage.label,
      verses: passage.verses.map((v) => ({ ...v, editing: false })),
    };
  }

  async fetchSuggestions(term: string): Promise<BibleSuggestion[]> {
    if (term.length < 2) return [];
    const query = new URLSearchParams({ term });
    return apiFetch(`/api/bible/suggest?${query}`, BibleSuggestionsSchema);
  }
}

export const bibleApi = new BibleApiService();
