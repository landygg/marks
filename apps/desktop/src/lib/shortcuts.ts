import type { PaletteMode } from './features/palette/commands';

const NEW_KEY = 'n';
const PALETTE_KEY = 'k';
const SETTINGS_KEY = ',';
const SIDEBAR_KEY = 's';
/** `KeyboardEvent.key` values: identifiers, not text. */
export const ENTER_KEY = 'Enter'; // i18n-allow: identifier
export const ESCAPE_KEY = 'Escape'; // i18n-allow: identifier
const MODE_KEYS: Record<string, PaletteMode> = { '1': 'commands', '2': 'bookmarks', '3': 'tags' };

// Physical keys of ⌥⌘1…⌥⌘9, in order.
const PROFILE_CODES = [
  'Digit1',
  'Digit2',
  'Digit3',
  'Digit4',
  'Digit5',
  'Digit6',
  'Digit7',
  'Digit8',
  'Digit9',
];

/** How many profiles have a ⌥⌘ shortcut. */
export const PROFILE_SHORTCUT_COUNT = PROFILE_CODES.length;

type Keys = {
  metaKey: boolean;
  ctrlKey: boolean;
  altKey: boolean;
  key: string;
  code: string;
  repeat?: boolean;
};

// Every shortcut takes ⌘ only (⌃⌘S aside, which is not a text-editing key): Ctrl+N, Ctrl+K and friends are text-editing keys in macOS fields.

/** ⌘N: opens the add dialog. */
export function isNewShortcut(e: Keys): boolean {
  return e.metaKey && e.key.toLowerCase() === NEW_KEY;
}

/** ⌘K: opens the command palette. */
export function isPaletteShortcut(e: Keys): boolean {
  return e.metaKey && e.key.toLowerCase() === PALETTE_KEY;
}

/** ⌘, : opens Settings. */
export function isSettingsShortcut(e: Keys): boolean {
  return e.metaKey && e.key === SETTINGS_KEY;
}

/** ⌃⌘S: shows or hides the sidebar; a held key does not repeat it. */
export function isSidebarShortcut(e: Keys): boolean {
  return e.metaKey && e.ctrlKey && !e.altKey && !e.repeat && e.key.toLowerCase() === SIDEBAR_KEY;
}

/** ⌘1, ⌘2, ⌘3 inside the palette: its mode, or `null` for any other key (⌥⌘ digits are profiles). */
export function paletteModeShortcut(e: Keys): PaletteMode | null {
  return e.metaKey && !e.altKey ? (MODE_KEYS[e.key] ?? null) : null;
}

/** ⌥⌘1…⌥⌘9 -> 0…8; reads `code` ("Digit1"), because with ⌥ held `key` is another character. */
export function profileShortcut(e: Keys): number | null {
  if (!e.metaKey || !e.altKey) return null;
  const position = PROFILE_CODES.indexOf(e.code);
  return position === -1 ? null : position;
}
