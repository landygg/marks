import { describe, expect, it } from 'vitest';
import { THEMES, parseTheme } from './theme';

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
