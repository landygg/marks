import { describe, expect, it } from 'vitest';
import { THEMES, parseTheme, storedTheme } from './theme';

describe('parseTheme', () => {
  it('keeps a known theme', () => {
    for (const theme of THEMES) expect(parseTheme(theme)).toBe(theme);
  });

  it('falls back to the system theme for anything else', () => {
    expect(parseTheme(null)).toBe('system');
    expect(parseTheme('')).toBe('system');
    expect(parseTheme('sepia')).toBe('system');
  });
});

describe('storedTheme', () => {
  const storage = (value: string | null) => ({ getItem: () => value });

  it('reads a stored theme', () => {
    expect(storedTheme(storage('light'))).toBe('light');
    expect(storedTheme(storage('dark'))).toBe('dark');
  });

  it('falls back to the system theme when nothing valid is stored or storage fails', () => {
    expect(storedTheme(storage(null))).toBe('system');
    expect(storedTheme(storage('x'))).toBe('system');
    const failing = {
      getItem: () => {
        throw new Error('denied');
      },
    };
    expect(storedTheme(failing)).toBe('system');
  });
});
