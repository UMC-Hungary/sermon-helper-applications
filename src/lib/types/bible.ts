// V2 API types (nyiregyhazimetodista.hu)
export interface V2Verse {
  chapter: number;
  verse: number;
  text: string;
}

export interface V2ParsedRef {
  book: string;
  book_id: number;
  chapter_from: number;
  chapter_to: number;
  verse_from: number | null;
  verse_to: number | null;
}

export interface V2SuggestApiResponse {
  label: string;
  link: string;
  hungarian_label: string;
  parsed_refs: V2ParsedRef[];
  verses: V2Verse[];
  verses_as_text: string[];
}

// Legacy API types (szentiras.eu)
export interface LegacyLocation {
  gepi: string;
  szep: string;
}

export interface LegacyNote {
  position: number | null;
  text: string;
}

export interface LegacyVerse {
  szoveg: string;
  jegyzetek: LegacyNote[];
  hely: LegacyLocation;
}

export interface LegacyTranslation {
  nev: string;
  rov: string;
}

export interface LegacyAnswer {
  versek: LegacyVerse[];
  forditas: LegacyTranslation;
}

export interface LegacySearchQuery {
  feladat: string;
  hivatkozas: string;
  forma: string;
}

export interface LegacySearchResponse {
  keres: LegacySearchQuery;
  valasz: LegacyAnswer;
}

export interface LegacySuggestion {
  cat: string;
  label: string;
  link: string;
}

// Unified internal types for the app
export interface BibleVerse {
  chapter: number;
  verse: number;
  text: string;
  editing?: boolean;
}

export interface BibleReference {
  label: string;
  verses: BibleVerse[];
  translation: string;
}

// Translation types
export type V2Translation = 'UF_v2' | 'RUF_v2';
export type LegacyTranslationType = 'RUF' | 'KG' | 'KNB' | 'SZIT' | 'BD' | 'STL';
export type BibleTranslation = V2Translation | LegacyTranslationType;

export interface TranslationOption {
  code: BibleTranslation;
  name: string;
  fullName: string;
  type: 'v2' | 'legacy';
}

export const TRANSLATIONS: TranslationOption[] = [
  // V2 translations (immediate results, no suggestion selection needed)
  { code: 'UF_v2', name: 'UF (v2)', fullName: 'Magyar Bibliatársulat Újfordítású Biblia (UF) (v2)', type: 'v2' },
  { code: 'RUF_v2', name: 'RUF (v2)', fullName: 'Magyar Bibliatársulat Újfordítású Biblia (RUF) (v2)', type: 'v2' },
  // Legacy translations (require suggestion selection)
  { code: 'RUF', name: 'RUF', fullName: 'Magyar Bibliatársulat Újfordítású Bibliája 2014 (RUF)', type: 'legacy' },
  { code: 'KG', name: 'KG', fullName: 'Károli Gáspár revideált fordítása (KG)', type: 'legacy' },
  { code: 'KNB', name: 'KNB', fullName: 'Káldi-Neovulgáta (KNB)', type: 'legacy' },
  { code: 'SZIT', name: 'SZIT', fullName: 'Szent István Társulati Biblia (SZIT)', type: 'legacy' },
  { code: 'BD', name: 'BD', fullName: 'Békés-Dalos Újszövetségi Szentírás (BD)', type: 'legacy' },
  { code: 'STL', name: 'STL', fullName: 'Simon Tamás László Újszövetség-fordítása (STL)', type: 'legacy' },
];

// Helper function to check if a translation is V2
export function isV2Translation(translation: BibleTranslation): boolean {
  return translation.endsWith('_v2');
}

// Get translation by code
export function getTranslation(code: BibleTranslation): TranslationOption | undefined {
  return TRANSLATIONS.find(t => t.code === code);
}

// Get V2 API translation code (without _v2 suffix)
export function getV2TranslationCode(translation: V2Translation): string {
  return translation.replace('_v2', '');
}
