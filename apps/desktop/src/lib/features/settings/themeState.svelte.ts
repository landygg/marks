// The chosen theme. It is a preference of this window, not library data, so it lives in the
// webview's localStorage instead of the database.
import { getCurrentWindow } from '@tauri-apps/api/window';
import { report } from '../library/libraryState.svelte';
import { parseTheme, type Theme } from './theme';

const STORAGE_KEY = 'marks.theme';

function stored(): Theme {
  try {
    return parseTheme(localStorage.getItem(STORAGE_KEY));
  } catch {
    return 'system';
  }
}

export const appearance = $state({ theme: stored() });

/** `data-theme` drives the CSS (`color-scheme` in app.css). */
function applyToDocument(theme: Theme): void {
  if (theme === 'system') delete document.documentElement.dataset.theme;
  else document.documentElement.dataset.theme = theme;
}

// At import time, before the first render, so a pinned theme does not flash the system one.
applyToDocument(appearance.theme);

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
