import { afterEach, describe, expect, it } from 'vitest';
import { domainOf, formatDate, formatTime, parseTags, truncate } from './format';
import { setLocale } from './i18n';

describe('domainOf', () => {
  it('drops the scheme, path and www. prefix', () => {
    expect(domainOf('https://www.example.com/a')).toBe('example.com');
  });

  it('returns the input unchanged when it is not a URL', () => {
    expect(domainOf('not a url')).toBe('not a url');
  });
});

describe('formatDate', () => {
  it('formats as day, short month without dot, year in Spanish', () => {
    expect(formatDate('2026-11-09T12:00:00Z')).toBe('9 nov 2026');
  });

  it('uses the Intl output as is in English', () => {
    setLocale('en');
    expect(formatDate('2026-11-09T12:00:00Z')).toBe('Nov 9, 2026');
  });

  afterEach(() => setLocale('es'));
});

describe('formatTime', () => {
  it('format_time_is_local_24h', () => {
    expect(formatTime(new Date(2026, 10, 13, 9, 5).getTime())).toBe('09:05');
    expect(formatTime(new Date(2026, 10, 13, 21, 30).getTime())).toBe('21:30');
  });
});

describe('parseTags', () => {
  it('splits on commas, trims, lowercases and drops duplicates', () => {
    expect(parseTags('Rust, rust ,a,b')).toEqual(['rust', 'a', 'b']);
  });

  it('returns no tags for blank input', () => {
    expect(parseTags('  ,  ')).toEqual([]);
  });

  it('keeps spaces inside a tag, collapsing repeated whitespace', () => {
    expect(parseTags('Machine   Learning, rust')).toEqual(['machine learning', 'rust']);
  });
});

describe('truncate', () => {
  it('keeps text within the limit unchanged', () => {
    expect(truncate('abc', 3)).toBe('abc');
  });

  it('cuts longer text to the limit and appends an ellipsis', () => {
    expect(truncate('abcdef', 3)).toBe('abc…');
  });

  it('trims trailing whitespace before the ellipsis', () => {
    expect(truncate('ab   cdef', 4)).toBe('ab…');
  });
});
