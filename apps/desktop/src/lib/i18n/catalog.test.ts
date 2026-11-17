import { describe, expect, it } from 'vitest';
import { en } from './en';
import { es } from './es';
import { pseudo } from './pseudo';

/** Every `.ts` and `.svelte` file of the app outside the catalogs and the tests. */
const sources = Object.entries({
  ...import.meta.glob<string>('../../**/*.{ts,svelte}', {
    query: '?raw',
    import: 'default',
    eager: true,
  }),
})
  .filter(([path]) => !path.includes('/i18n/') && !path.endsWith('.test.ts'))
  .map(([, text]) => text)
  .join('\n');

/** Dotted paths of the leaves: strings, functions and arrays (a list of steps is one message). */
function leaves(value: object, prefix = ''): string[] {
  return Object.entries(value).flatMap(([key, child]) =>
    typeof child === 'object' && !Array.isArray(child)
      ? leaves(child, `${prefix}${key}.`)
      : [`${prefix}${key}`],
  );
}

const escape = (path: string) => path.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');

describe('catalogs', () => {
  it('es and en have the same keys', () => {
    expect(leaves(en)).toEqual(leaves(es));
  });

  it('pseudo keeps the shape of en', () => {
    expect(leaves(pseudo(en))).toEqual(leaves(en));
    expect(pseudo(en).save).toMatch(/^⟦.+⟧$/);
    expect(pseudo(en).bookmarkCount(2)).toMatch(/^⟦.+⟧$/);
  });

  it('every message is used', () => {
    const unused = leaves(es).filter((path) => {
      const parent = path.split('.').slice(0, -1).join('.');
      const direct = new RegExp(`\\bt\\.${escape(path)}(?![\\w$])`);
      const indexed = parent !== '' && new RegExp(`\\bt\\.${escape(parent)}\\[`).test(sources);
      return !direct.test(sources) && !indexed;
    });
    expect(unused).toEqual([]);
  });
});
