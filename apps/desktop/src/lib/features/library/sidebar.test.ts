import { describe, expect, it } from 'vitest';
import { ALL, matches, sameSelection, sidebarGroups, validSelection } from './sidebar';

const groups = [
  { id: 1, name: 'Datos', tags: ['fts5', 'sqlite', 'huérfana'] },
  { id: 2, name: 'Vacío', tags: [] },
];

describe('sidebarGroups', () => {
  it('lists user groups, then the ungrouped tags', () => {
    expect(sidebarGroups(['búsqueda', 'fts5', 'sqlite'], groups, 'Sin grupo')).toEqual([
      { id: 1, name: 'Datos', tags: ['fts5', 'sqlite'] }, // "huérfana" is on no bookmark
      { id: 2, name: 'Vacío', tags: [] }, // empty groups stay as drop targets
      { id: null, name: 'Sin grupo', tags: ['búsqueda'] },
    ]);
  });
});

describe('validSelection', () => {
  it('falls back to ALL when the selected tag or category is gone', () => {
    expect(validSelection({ kind: 'tag', value: 'x' }, [], ['sqlite'])).toEqual(ALL);
    expect(validSelection({ kind: 'category', value: 'Rust' }, [], ['Rust'])).toEqual(ALL);
  });

  it('keeps a selection that still exists', () => {
    const rust = { kind: 'category', value: 'Rust' } as const;
    expect(validSelection(rust, ['Rust'], [])).toEqual(rust);
    expect(validSelection(ALL, [], [])).toEqual(ALL);
  });
});

describe('matches', () => {
  const bookmark = { category: 'Rust', tags: ['sqlite'] };

  it('matches by selection', () => {
    expect(matches(ALL, bookmark)).toBe(true);
    expect(matches({ kind: 'tag', value: 'sqlite' }, bookmark)).toBe(true);
    expect(matches({ kind: 'category', value: 'Rust' }, bookmark)).toBe(true);
    expect(matches({ kind: 'category', value: 'Diseño' }, bookmark)).toBe(false);
    expect(matches({ kind: 'tag', value: 'Rust' }, bookmark)).toBe(false);
  });
});

describe('sameSelection', () => {
  it('compares kind and value', () => {
    expect(sameSelection(ALL, { kind: 'all' })).toBe(true);
    expect(sameSelection({ kind: 'tag', value: 'a' }, { kind: 'tag', value: 'a' })).toBe(true);
    expect(sameSelection({ kind: 'tag', value: 'a' }, { kind: 'category', value: 'a' })).toBe(
      false,
    );
  });
});
