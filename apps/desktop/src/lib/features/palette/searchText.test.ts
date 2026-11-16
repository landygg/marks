import { describe, expect, it } from 'vitest';
import { excerpt, highlight, moveIndex, pointerMoved, rowAfterPointerLeaves } from './searchText';

describe('highlight', () => {
  it('marks every occurrence, ignoring case', () => {
    expect(highlight('SQLite y sqlite-vec', 'sqlite')).toEqual([
      { text: 'SQLite', match: true },
      { text: ' y ', match: false },
      { text: 'sqlite', match: true },
      { text: '-vec', match: false },
    ]);
  });

  it('treats the query as literal text', () => {
    expect(highlight('c++ vs rust', 'c++')[0]).toEqual({ text: 'c++', match: true });
    expect(highlight('a (b) [c', '( [')).toEqual([
      { text: 'a ', match: false },
      { text: '(', match: true },
      { text: 'b) ', match: false },
      { text: '[', match: true },
      { text: 'c', match: false },
    ]);
    expect(highlight('x[a]', '[a')).toEqual([
      { text: 'x', match: false },
      { text: '[a', match: true },
      { text: ']', match: false },
    ]);
  });

  it('returns the text whole for a blank query or no match', () => {
    expect(highlight('texto', '   ')).toEqual([{ text: 'texto', match: false }]);
    expect(highlight('texto', 'zzz')).toEqual([{ text: 'texto', match: false }]);
    expect(highlight('', 'a')).toEqual([]);
  });
});

describe('excerpt', () => {
  it('cuts around the first match', () => {
    const text = 'a'.repeat(200) + ' FTS5 ' + 'b'.repeat(200);
    const cut = excerpt(text, 'fts5', 60);
    expect(cut).toContain('FTS5');
    expect(cut.length).toBeLessThanOrEqual(62); // 60 + two ellipses
    expect(cut.startsWith('…')).toBe(true);
    expect(cut.endsWith('…')).toBe(true);
  });

  it('keeps short text and starts at the beginning without a match', () => {
    expect(excerpt('corto', 'x', 60)).toBe('corto');
    expect(excerpt('x'.repeat(100), 'zzz', 10)).toBe('x'.repeat(10) + '…');
  });
});

describe('moveIndex', () => {
  it('wraps the selection and survives an empty list', () => {
    expect(moveIndex(0, -1, 3)).toBe(2);
    expect(moveIndex(2, 1, 3)).toBe(0);
    expect(moveIndex(0, 1, 0)).toBe(0);
  });
});

describe('pointerMoved', () => {
  it('is false for the first move after opening, which only shows where the pointer rests', () => {
    expect(pointerMoved(null, { x: 10, y: 20 })).toBe(false);
  });

  it('is false when the list scrolled under a resting pointer', () => {
    expect(pointerMoved({ x: 10, y: 20 }, { x: 10, y: 20 })).toBe(false);
  });

  it('is true when the pointer went somewhere else', () => {
    expect(pointerMoved({ x: 10, y: 20 }, { x: 10, y: 21 })).toBe(true);
    expect(pointerMoved({ x: 10, y: 20 }, { x: 11, y: 20 })).toBe(true);
  });
});

describe('rowAfterPointerLeaves', () => {
  it('gives back the row chosen before the pointer crossed others on its way to the footer', () => {
    // Chosen "Add bookmark" (0), crossed down to the last row (8), left for "Run".
    expect(rowAfterPointerLeaves(8, 0, 9)).toBe(0);
  });

  it('keeps the current row when the pointer never took over', () => {
    expect(rowAfterPointerLeaves(3, null, 9)).toBe(3);
  });

  it('keeps the current row when the chosen one is gone (the list got shorter)', () => {
    expect(rowAfterPointerLeaves(1, 5, 2)).toBe(1);
  });
});
