// The appearance setting: follow the system or pin one theme.
export const THEMES = ['system', 'light', 'dark'] as const;
export type Theme = (typeof THEMES)[number];

/** The stored value if it names a theme, else `"system"`. */
export function parseTheme(value: string | null): Theme {
  return THEMES.find((theme) => theme === value) ?? 'system';
}

/** Where the chosen theme is kept: the webview's localStorage (a preference of this window). */
export const STORAGE_KEY = 'marks.theme';

/** The stored theme; `"system"` if none is valid or storage fails. */
export function storedTheme(storage?: Pick<Storage, 'getItem'>): Theme {
  try {
    return parseTheme((storage ?? localStorage).getItem(STORAGE_KEY));
  } catch {
    return 'system';
  }
}

/** `data-theme` drives the CSS (`color-scheme` in app.css); none follows the system. */
export function applyToDocument(theme: Theme): void {
  if (theme === 'system') delete document.documentElement.dataset.theme;
  else document.documentElement.dataset.theme = theme;
}
