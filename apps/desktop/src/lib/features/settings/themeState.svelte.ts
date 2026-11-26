// The chosen theme. It is a preference of this window, not library data, so it lives in the
// webview's localStorage instead of the database.
import { getCurrentWindow } from '@tauri-apps/api/window';
import { report } from '../library/libraryState.svelte';
import { STORAGE_KEY, applyToDocument, storedTheme, type Theme } from './theme';

// `main.ts` already applied it to the document, before the first frame.
export const appearance = $state({ theme: storedTheme() });

/** Applies the theme to the page and to the window, which makes native menus match. */
export function applyTheme(): void {
  const { theme } = appearance;
  applyToDocument(theme);
  getCurrentWindow()
    .setTheme(theme === 'system' ? null : theme)
    .catch((err) => report(err, { profile: null }));
}

export function setTheme(theme: Theme): void {
  appearance.theme = theme;
  try {
    localStorage.setItem(STORAGE_KEY, theme);
  } catch {
    // Storage unavailable: the choice still applies until the app closes.
  }
  applyTheme();
}
