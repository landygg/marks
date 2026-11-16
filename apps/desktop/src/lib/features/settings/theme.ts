// The appearance setting: follow the system or pin one theme.
export const THEMES = ['system', 'light', 'dark'] as const;
export type Theme = (typeof THEMES)[number];

/** The stored value if it names a theme, else `"system"`. */
export function parseTheme(value: string | null): Theme {
  return THEMES.find((theme) => theme === value) ?? 'system';
}
