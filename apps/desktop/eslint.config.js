// Flat config from the eslint-plugin-svelte guide (TypeScript variant). Formatting is Prettier's:
// `svelte.configs.prettier` turns off the plugin's style rules that would fight it.
import js from '@eslint/js';
import svelte from 'eslint-plugin-svelte';
import { defineConfig, globalIgnores } from 'eslint/config';
import globals from 'globals';
import ts from 'typescript-eslint';
import svelteConfig from './svelte.config.js';

export default defineConfig(
  globalIgnores(['dist/', 'src-tauri/', '.svelte-check/']),
  js.configs.recommended,
  ts.configs.recommended,
  svelte.configs.recommended,
  svelte.configs.prettier,
  {
    languageOptions: {
      globals: globals.browser,
      parserOptions: { extraFileExtensions: ['.svelte'] },
    },
  },
  {
    files: ['**/*.svelte', '**/*.svelte.ts'],
    languageOptions: {
      parserOptions: { projectService: true, parser: ts.parser, svelteConfig },
    },
  },
  {
    files: ['*.config.{js,ts}'],
    languageOptions: { globals: globals.node },
  },
);
