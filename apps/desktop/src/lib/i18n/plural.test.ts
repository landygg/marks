import { afterEach, describe, expect, it } from 'vitest';
import { num, plural, setLocale, t } from '.';

afterEach(() => setLocale('es'));

describe('plural', () => {
  it('treats many as other', () => {
    setLocale('es');
    expect(t.bookmarkCount(1_000_000)).toBe('1.000.000 bookmarks');
    expect(t.bookmarkCount(1)).toBe('1 bookmark');
    setLocale('en');
    expect(t.bookmarkCount(1_000_000)).toBe('1,000,000 bookmarks');
    expect(t.notices.importDone(1)).toBe('1 bookmark imported, not extracted.');
  });

  it('picks one only for the singular', () => {
    setLocale('en');
    expect(plural(1, { one: 'a', other: 'b' })).toBe('a');
    expect(plural(0, { one: 'a', other: 'b' })).toBe('b');
    expect(plural(2, { one: 'a', other: 'b' })).toBe('b');
  });
});

describe('num', () => {
  it('formats decimals with the active locale', () => {
    setLocale('es');
    expect(num(2.2)).toBe('2,2');
    setLocale('en');
    expect(num(2.2)).toBe('2.2');
  });
});
