import { writable, derived } from 'svelte/store';
import type { BibleVerse, BibleTranslation, LegacySuggestion } from '$lib/types/bible';

export interface BibleTabState {
  query: string;
  translation: BibleTranslation;
  verses: BibleVerse[];
  label: string;
  loading: boolean;
  error: string | null;
}

export interface BibleEditorState {
  textus: BibleTabState;
  leckio: BibleTabState;
  suggestions: LegacySuggestion[];
  showSuggestions: boolean;
  activeSuggestionTab: 'textus' | 'leckio' | null;
}

const DEFAULT_TAB_STATE: BibleTabState = {
  query: '',
  translation: 'RUF_v2',
  verses: [],
  label: '',
  loading: false,
  error: null,
};

const DEFAULT_STATE: BibleEditorState = {
  textus: { ...DEFAULT_TAB_STATE },
  leckio: { ...DEFAULT_TAB_STATE },
  suggestions: [],
  showSuggestions: false,
  activeSuggestionTab: null,
};

function createBibleStore() {
  const { subscribe, set, update } = writable<BibleEditorState>(DEFAULT_STATE);

  return {
    subscribe,

    // Update query for a tab
    setQuery: (tab: 'textus' | 'leckio', query: string) => {
      update(state => ({
        ...state,
        [tab]: { ...state[tab], query },
      }));
    },

    // Update translation for a tab
    setTranslation: (tab: 'textus' | 'leckio', translation: BibleTranslation) => {
      update(state => ({
        ...state,
        [tab]: { ...state[tab], translation },
      }));
    },

    // Set loading state
    setLoading: (tab: 'textus' | 'leckio', loading: boolean) => {
      update(state => ({
        ...state,
        [tab]: { ...state[tab], loading, error: loading ? null : state[tab].error },
      }));
    },

    // Set verses after successful fetch
    setVerses: (tab: 'textus' | 'leckio', verses: BibleVerse[], label: string) => {
      update(state => ({
        ...state,
        [tab]: { ...state[tab], verses, label, loading: false, error: null },
      }));
    },

    // Set error
    setError: (tab: 'textus' | 'leckio', error: string) => {
      update(state => ({
        ...state,
        [tab]: { ...state[tab], error, loading: false },
      }));
    },

    // Update a specific verse's text
    updateVerse: (tab: 'textus' | 'leckio', verseIndex: number, text: string) => {
      update(state => ({
        ...state,
        [tab]: {
          ...state[tab],
          verses: state[tab].verses.map((v, i) =>
            i === verseIndex ? { ...v, text } : v
          ),
        },
      }));
    },

    // Toggle verse editing mode
    toggleEditing: (tab: 'textus' | 'leckio', verseIndex: number) => {
      update(state => ({
        ...state,
        [tab]: {
          ...state[tab],
          verses: state[tab].verses.map((v, i) =>
            i === verseIndex ? { ...v, editing: !v.editing } : v
          ),
        },
      }));
    },

    // Set all verses to non-editing mode
    finishAllEditing: (tab: 'textus' | 'leckio') => {
      update(state => ({
        ...state,
        [tab]: {
          ...state[tab],
          verses: state[tab].verses.map(v => ({ ...v, editing: false })),
        },
      }));
    },

    // Set suggestions for autocomplete (legacy translations)
    setSuggestions: (suggestions: LegacySuggestion[], tab: 'textus' | 'leckio') => {
      update(state => ({
        ...state,
        suggestions,
        showSuggestions: suggestions.length > 0,
        activeSuggestionTab: suggestions.length > 0 ? tab : null,
      }));
    },

    // Clear suggestions
    clearSuggestions: () => {
      update(state => ({
        ...state,
        suggestions: [],
        showSuggestions: false,
        activeSuggestionTab: null,
      }));
    },

    // Clear verses for a tab
    clearVerses: (tab: 'textus' | 'leckio') => {
      update(state => ({
        ...state,
        [tab]: {
          ...state[tab],
          verses: [],
          label: '',
          error: null,
        },
      }));
    },

    // Reset a tab to defaults
    resetTab: (tab: 'textus' | 'leckio') => {
      update(state => ({
        ...state,
        [tab]: { ...DEFAULT_TAB_STATE },
      }));
    },

    // Reset entire store to defaults
    reset: () => set(DEFAULT_STATE),
  };
}

export const bibleStore = createBibleStore();

// Derived store for current sermon references (can be used in sidebar)
export const currentSermonRefs = derived(bibleStore, $bible => ({
  textus: $bible.textus.label || $bible.textus.query || '',
  leckio: $bible.leckio.label || $bible.leckio.query || '',
  hasTextus: $bible.textus.verses.length > 0,
  hasLeckio: $bible.leckio.verses.length > 0,
}));

// Derived store for loading state
export const bibleLoading = derived(bibleStore, $bible => ({
  textus: $bible.textus.loading,
  leckio: $bible.leckio.loading,
  any: $bible.textus.loading || $bible.leckio.loading,
}));

// Derived store for errors
export const bibleErrors = derived(bibleStore, $bible => ({
  textus: $bible.textus.error,
  leckio: $bible.leckio.error,
  hasAny: !!$bible.textus.error || !!$bible.leckio.error,
}));
