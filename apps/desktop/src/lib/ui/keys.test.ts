import { describe, expect, it } from 'vitest';
import { shortcutKeys } from './keys';

describe('shortcutKeys', () => {
  it('splits each leading modifier into its own key', () => {
    expect(shortcutKeys('⌘K')).toEqual(['⌘', 'K']);
    expect(shortcutKeys('⌥⌘2')).toEqual(['⌥', '⌘', '2']);
    expect(shortcutKeys('⌃⌥⇧⌘P')).toEqual(['⌃', '⌥', '⇧', '⌘', 'P']);
  });

  it('keeps a punctuation key and a named key whole', () => {
    expect(shortcutKeys('⌘,')).toEqual(['⌘', ',']);
    expect(shortcutKeys('esc')).toEqual(['esc']);
    expect(shortcutKeys('↵')).toEqual(['↵']);
  });

  it('returns a lone modifier as one key', () => {
    expect(shortcutKeys('⌘')).toEqual(['⌘']);
  });
});
