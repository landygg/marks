import { describe, expect, it } from 'vitest';
import {
  isComposing,
  isNewShortcut,
  isPaletteShortcut,
  isSettingsShortcut,
  isSidebarShortcut,
  paletteModeShortcut,
  profileShortcut,
} from './shortcuts';

const key = (
  key: string,
  mods: {
    metaKey?: boolean;
    ctrlKey?: boolean;
    altKey?: boolean;
    code?: string;
    repeat?: boolean;
  } = {},
) => ({
  metaKey: false,
  ctrlKey: false,
  altKey: false,
  key,
  code: '',
  ...mods,
});

describe('isNewShortcut', () => {
  it('accepts ⌘N', () => {
    expect(isNewShortcut(key('n', { metaKey: true }))).toBe(true);
  });

  it('accepts ⌘N with Shift or Caps Lock (uppercase key)', () => {
    expect(isNewShortcut(key('N', { metaKey: true }))).toBe(true);
  });

  it('leaves Ctrl+N to the text field (next line in macOS)', () => {
    expect(isNewShortcut(key('n', { ctrlKey: true }))).toBe(false);
  });

  it('rejects a plain n', () => {
    expect(isNewShortcut(key('n'))).toBe(false);
  });

  it('rejects ⌘ with another key', () => {
    expect(isNewShortcut(key('m', { metaKey: true }))).toBe(false);
  });
});

describe('palette and settings shortcuts', () => {
  it('accepts ⌘K and ⌘, and rejects the bare keys', () => {
    expect(isPaletteShortcut(key('k', { metaKey: true }))).toBe(true);
    expect(isPaletteShortcut(key('k'))).toBe(false);
    // Ctrl+K is "kill line" in macOS text fields, so it must stay with the text field.
    expect(isPaletteShortcut(key('k', { ctrlKey: true }))).toBe(false);
    expect(isSettingsShortcut(key(',', { ctrlKey: true }))).toBe(false);
    expect(isSettingsShortcut(key(',', { metaKey: true }))).toBe(true);
    expect(isSettingsShortcut(key(','))).toBe(false);
  });

  it('maps ⌘1, ⌘2 and ⌘3 to the palette modes', () => {
    expect(paletteModeShortcut(key('1', { metaKey: true }))).toBe('commands');
    expect(paletteModeShortcut(key('2', { metaKey: true }))).toBe('bookmarks');
    expect(paletteModeShortcut(key('3', { metaKey: true }))).toBe('tags');
    expect(paletteModeShortcut(key('4', { metaKey: true }))).toBeNull();
    expect(paletteModeShortcut(key('1'))).toBeNull();
    expect(paletteModeShortcut(key('1', { ctrlKey: true }))).toBeNull();
  });

  it('leaves ⌥⌘1 to the profiles', () => {
    expect(
      paletteModeShortcut(key('1', { metaKey: true, altKey: true, code: 'Digit1' })),
    ).toBeNull();
  });
});

describe('profileShortcut', () => {
  it('maps ⌥⌘1…⌥⌘9 to positions 0…8 by the physical key', () => {
    // With ⌥ held, macOS reports another character as `key`.
    expect(profileShortcut(key('¡', { metaKey: true, altKey: true, code: 'Digit1' }))).toBe(0);
    expect(profileShortcut(key('9', { metaKey: true, altKey: true, code: 'Digit9' }))).toBe(8);
  });

  it('rejects ⌘ without ⌥ (palette modes) and ⌥⌘0', () => {
    expect(profileShortcut(key('1', { metaKey: true, code: 'Digit1' }))).toBeNull();
    expect(profileShortcut(key('0', { metaKey: true, altKey: true, code: 'Digit0' }))).toBeNull();
    expect(profileShortcut(key('¡', { altKey: true, code: 'Digit1' }))).toBeNull();
  });
});

describe('isSidebarShortcut', () => {
  it('accepts ⌃⌘S', () => {
    expect(isSidebarShortcut(key('s', { metaKey: true, ctrlKey: true }))).toBe(true);
    expect(isSidebarShortcut(key('S', { metaKey: true, ctrlKey: true }))).toBe(true);
  });

  it('rejects the near misses', () => {
    expect(isSidebarShortcut(key('s', { metaKey: true }))).toBe(false);
    expect(isSidebarShortcut(key('s', { ctrlKey: true }))).toBe(false);
    expect(isSidebarShortcut(key('s', { metaKey: true, ctrlKey: true, altKey: true }))).toBe(false);
    expect(isSidebarShortcut(key('d', { metaKey: true, ctrlKey: true }))).toBe(false);
  });

  it('ignores key repeat, so holding the keys does not flap the sidebar', () => {
    expect(isSidebarShortcut(key('s', { metaKey: true, ctrlKey: true, repeat: true }))).toBe(false);
  });
});

describe('isComposing', () => {
  const press = (init: KeyboardEventInit & { keyCode?: number }) =>
    init as unknown as KeyboardEvent;

  it('is true while the browser says it composes', () => {
    expect(isComposing(press({ key: 'Enter', isComposing: true }))).toBe(true);
  });

  it('is true for keyCode 229, which Safari sends after isComposing is already false', () => {
    expect(isComposing(press({ key: 'Enter', isComposing: false, keyCode: 229 }))).toBe(true);
  });

  it('is false for a plain Enter', () => {
    expect(isComposing(press({ key: 'Enter', isComposing: false, keyCode: 13 }))).toBe(false);
  });
});

// Every handler that reads Enter, Esc or an arrow in a text field steps aside while an input method composes.
describe('key handlers in .svelte files', () => {
  const sources = import.meta.glob<string>(['./**/*.svelte', '../App.svelte'], {
    query: '?raw',
    import: 'default',
    eager: true,
  });
  // Files whose key handlers never sit on a text field, with the reason.
  const NO_TEXT_FIELD: Record<string, string> = {
    './ui/InlineConfirm.svelte': 'buttons only: Esc cancels the question',
  };
  const READS_KEYS = /ENTER_KEY|ESCAPE_KEY|'Arrow/;

  it('use isComposing', () => {
    const missing = Object.entries(sources)
      .filter(([path, source]) => READS_KEYS.test(source) && !(path in NO_TEXT_FIELD))
      .filter(([, source]) => !source.includes('isComposing('))
      .map(([path]) => path);
    expect(missing).toEqual([]);
  });

  it('list only exemptions that still read keys', () => {
    for (const path of Object.keys(NO_TEXT_FIELD))
      expect(READS_KEYS.test(sources[path])).toBe(true);
  });
});
