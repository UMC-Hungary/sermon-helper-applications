import tseslint from 'typescript-eslint';
import svelte from 'eslint-plugin-svelte';
import svelteParser from 'svelte-eslint-parser';
import globals from 'globals';
import prettier from 'eslint-config-prettier';

export default tseslint.config(
  { ignores: ['build/', '.svelte-kit/', 'node_modules/', 'src-tauri/'] },

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

  // Prettier last — disables formatting rules that conflict
  prettier,
);
