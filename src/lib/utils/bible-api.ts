import { invoke } from '@tauri-apps/api/core';
import { getApiConfig, buildProxiedUrl } from '$lib/config/bible-api';
import type {
  V2SuggestApiResponse,
  LegacySearchResponse,
  LegacySuggestion,
  BibleVerse,
  BibleTranslation,
} from '$lib/types/bible';
import { isV2Translation, getV2TranslationCode } from '$lib/types/bible';

// Check if running in Tauri environment
const isTauriApp = (): boolean => {
  return typeof window !== 'undefined' &&
         // @ts-ignore - Tauri internal property
         typeof (window as any).__TAURI_INTERNALS__ !== 'undefined';
};

/**
 * Build URL for browser mode, optionally using CORS proxy
 */
const buildBrowserUrl = (url: string, useCorsProxy: boolean, proxyUrl?: string): string => {
  if (useCorsProxy && proxyUrl) {
    return buildProxiedUrl(url, proxyUrl);
  }
  return url;
};

class BibleApiService {
  private config = getApiConfig();

  /**
   * Fetch verses using V2 API (immediate results, no suggestion needed)
   */
  async fetchV2(reference: string, translation: string): Promise<{ verses: BibleVerse[]; label: string }> {
    // V2 API expects translation without _v2 suffix
    const apiTranslation = getV2TranslationCode(translation as any);

    if (isTauriApp()) {
      const response = await invoke<V2SuggestApiResponse>('fetch_bible_v2', {
        reference,
        translation: apiTranslation,
        apiUrl: this.config.v2ApiUrl,
      });

      return {
        verses: response.verses.map(v => ({
          chapter: v.chapter,
          verse: v.verse,
          text: v.text,
          editing: false,
        })),
        label: response.hungarian_label || reference,
      };
    } else {
      const url = `${this.config.v2ApiUrl}/suggest/${encodeURIComponent(reference)}/${apiTranslation}`;

      const response = await fetch(url);

      if (!response.ok) {
        throw new Error(`API error: ${response.status}`);
      }

      const data: V2SuggestApiResponse = await response.json();

      return {
        verses: data.verses.map(v => ({
          chapter: v.chapter,
          verse: v.verse,
          text: v.text,
          editing: false,
        })),
        label: data.hungarian_label || reference,
      };
    }
  }

  /**
   * Fetch suggestions for legacy API (autocomplete)
   */
  async fetchSuggestions(term: string): Promise<LegacySuggestion[]> {
    if (!term || term.length < 2) {
      return [];
    }

    if (isTauriApp()) {
      return invoke<LegacySuggestion[]>('fetch_bible_suggestions', {
        term,
        apiUrl: this.config.legacyApiUrl,
      });
    } else {
      // Browser fallback with CORS proxy
      const originalUrl = `${this.config.legacyApiUrl}/kereses/suggest?term=${encodeURIComponent(term)}`;
      const url = buildBrowserUrl(originalUrl, this.config.useCorsProxy, this.config.browserProxyUrl);

      const response = await fetch(url);

      if (!response.ok) {
        throw new Error(`API error: ${response.status}`);
      }

      const suggestions: LegacySuggestion[] = await response.json();
      return suggestions.filter(s => s.cat === 'ref');
    }
  }

  /**
   * Fetch verses using Legacy API
   */
  async fetchLegacy(reference: string, translation: string): Promise<{ verses: BibleVerse[]; label: string }> {
    if (isTauriApp()) {
      const response = await invoke<LegacySearchResponse>('fetch_bible_legacy', {
        reference,
        translation,
        apiUrl: this.config.legacyApiUrl,
      });

      return {
        verses: this.transformLegacyVerses(response.valasz.versek),
        label: response.keres.hivatkozas,
      };
    } else {
      // Strip leading slash if present and encode only spaces (preserve commas for Hungarian notation)
      const cleanRef = reference.startsWith('/') ? reference.slice(1) : reference;
      const encodedRef = cleanRef.replace(/ /g, '%20');
      const url = `${this.config.legacyApiUrl}/api/idezet/${encodedRef}/${translation}`;
      const response = await fetch(url);

      if (!response.ok) {
        throw new Error(`API error: ${response.status}`);
      }

      const data: LegacySearchResponse = await response.json();

      return {
        verses: this.transformLegacyVerses(data.valasz.versek),
        label: data.keres.hivatkozas,
      };
    }
  }

  /**
   * Transform legacy verse format to unified BibleVerse format
   */
  private transformLegacyVerses(versek: LegacySearchResponse['valasz']['versek']): BibleVerse[] {
    return versek.map((v, index) => {
      // Parse verse location from "gepi" format
      // Format: book_id (3 digits) + chapter (3 digits) + verse (3 digits)
      // e.g., "001001016" -> book 1, chapter 1, verse 16
      const gepi = v.hely.gepi;
      let chapter = 1;
      let verse = index + 1;

      if (gepi && gepi.length >= 6) {
        // Extract last 6 characters for chapter and verse
        const chapterStr = gepi.slice(-6, -3);
        const verseStr = gepi.slice(-3);
        chapter = parseInt(chapterStr, 10) || 1;
        verse = parseInt(verseStr, 10) || index + 1;
      }

      return {
        chapter,
        verse,
        text: this.cleanHtml(v.szoveg),
        editing: false,
      };
    });
  }

  /**
   * Clean HTML tags from verse text
   */
  private cleanHtml(html: string): string {
    if (!html) return '';
    // Remove HTML tags
    return html.replace(/<[^>]*>/g, '').trim();
  }

  /**
   * Unified fetch method that auto-detects translation type
   */
  async fetchVerses(reference: string, translation: BibleTranslation): Promise<{ verses: BibleVerse[]; label: string }> {
    if (isV2Translation(translation)) {
      return this.fetchV2(reference, translation);
    } else {
      return this.fetchLegacy(reference, translation);
    }
  }

  /**
   * Update API configuration (for settings)
   */
  updateConfig(config: Partial<{
    v2ApiUrl: string;
    legacyApiUrl: string;
    browserProxyUrl: string;
    useCorsProxy: boolean;
  }>) {
    if (config.v2ApiUrl) {
      this.config.v2ApiUrl = config.v2ApiUrl;
    }
    if (config.legacyApiUrl) {
      this.config.legacyApiUrl = config.legacyApiUrl;
    }
    if (config.browserProxyUrl !== undefined) {
      this.config.browserProxyUrl = config.browserProxyUrl;
    }
    if (config.useCorsProxy !== undefined) {
      this.config.useCorsProxy = config.useCorsProxy;
    }
  }

  /**
   * Check if running in Tauri or browser mode
   */
  isTauri(): boolean {
    return isTauriApp();
  }

  /**
   * Get current configuration (for debugging)
   */
  getConfig() {
    return { ...this.config };
  }
}

// Export singleton instance
export const bibleApi = new BibleApiService();
