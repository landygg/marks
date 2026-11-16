/// <reference types="vitest/config" />
import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

// Tauri expects a fixed port (tauri.conf.json `devUrl`) and must not hide Rust errors.
const DEV_PORT = 1420;

export default defineConfig(({ mode }) => ({
  plugins: [svelte()],
  clearScreen: false,
  server: {
    port: DEV_PORT,
    strictPort: true,
    host: '127.0.0.1',
    watch: { ignored: ['**/src-tauri/**'] },
  },
  // Second page for the hidden `parser` window (src-tauri/src/parser_window.rs).
  build: { rollupOptions: { input: { main: 'index.html', extract: 'extract.html' } } },
  // Fixed zone so date tests do not depend on the machine running them.
  // `css: true` so designTokens.test.ts can read app.css (vitest blanks stylesheets otherwise).
  test: { env: { TZ: 'UTC' }, css: true },
  // Component tests `mount`, which needs Svelte's client build; the app build keeps the defaults.
  resolve: mode === 'test' ? { conditions: ['browser'] } : {},
}));
