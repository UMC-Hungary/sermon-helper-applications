import tseslint from 'typescript-eslint';
import svelte from 'eslint-plugin-svelte';
import svelteParser from 'svelte-eslint-parser';
import globals from 'globals';
import prettier from 'eslint-config-prettier';

export default tseslint.config(
  { ignores: ['build/', '.svelte-kit/', 'node_modules/', 'src-tauri/', 'companion/dist/'] },

  // TypeScript files
  {
    files: ['**/*.ts'],
    extends: tseslint.configs.recommended,
    languageOptions: {
      globals: { ...globals.browser, ...globals.node },
    },
    rules: {
      '@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_' }],
      '@typescript-eslint/no-explicit-any': 'error',
      '@typescript-eslint/consistent-type-imports': ['error', { prefer: 'type-imports' }],
      '@typescript-eslint/no-import-type-side-effects': 'error',
    },
  },

  // Svelte files — recommended rules
  ...svelte.configs['flat/recommended'],
  // Disable rules that incorrectly flag plain <a href> and goto() navigation
  { rules: { 'svelte/no-navigation-without-resolve': 'off' } },

  // Svelte files — TypeScript parser + extra rules
  {
    files: ['**/*.svelte'],
    languageOptions: {
      parser: svelteParser,
      parserOptions: { parser: tseslint.parser },
      globals: { ...globals.browser },
    },
    plugins: { '@typescript-eslint': tseslint.plugin },
    rules: {
      '@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_' }],
      '@typescript-eslint/no-explicit-any': 'error',
    },
  },

  // ── Core client boundary ────────────────────────────────────────────────────
  // A rendering UI reaches the core only through $lib/core-client. Tauri IPC,
  // raw fetch and raw WebSocket live inside the SDK so a UI stays portable
  // across desktop, browser and remote-client hosting.
  {
    files: ['src/**/*.{ts,svelte}'],
    // The SDK and the modules it re-exports are what own the transports.
    ignores: [
      'src/lib/core-client/**',
      'src/lib/api/**',
      'src/lib/ws/**',
      'src/lib/host/**',
      'src/lib/utils/bible-api.ts',
    ],
    rules: {
      'no-restricted-imports': [
        'error',
        {
          patterns: [
            {
              group: ['@tauri-apps/*', 'tauri-plugin-*'],
              message:
                'UI code must not use Tauri directly. Use $lib/core-client (host capabilities are feature-detected there).',
            },
            {
              group: ['$lib/api/*', '$lib/ws/*', '$lib/host/*'],
              message: 'Import from $lib/core-client instead of reaching into its layers.',
            },
          ],
        },
      ],
      'no-restricted-globals': [
        'error',
        {
          name: 'fetch',
          message: 'Use the core client SDK ($lib/core-client) instead of raw fetch.',
        },
        {
          name: 'WebSocket',
          message: 'Use connectWs/connectPresenterWs from $lib/core-client instead.',
        },
      ],
      'no-restricted-syntax': [
        'error',
        {
          selector: "NewExpression[callee.name='WebSocket']",
          message: 'Use connectWs/connectPresenterWs from $lib/core-client instead.',
        },
        {
          selector: "CallExpression[callee.name='fetch']",
          message: 'Use the core client SDK ($lib/core-client) instead of raw fetch.',
        },
      ],
    },
  },

  // Prettier last — disables formatting rules that conflict
  prettier,
);
