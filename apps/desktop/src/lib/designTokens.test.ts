import { parseCss } from 'svelte/compiler';
import { describe, expect, it } from 'vitest';
// DESIGN.md is the source of the design. These checks keep the three things that drift first in
// step with it: the tokens of app.css, the rule that only lib/ui draws controls, and the rule
// that a style takes its values from tokens. They read the CSS as a tree, not as text.
import design from '../../../../DESIGN.md?raw';
import css from '../app.css?raw';

const frontMatter = design.split('---')[1];

/** Every `.svelte` file of the app, by path relative to `src/lib`. */
const sources: Record<string, string> = {
  ...import.meta.glob<string>('./**/*.svelte', { query: '?raw', import: 'default', eager: true }),
  ...import.meta.glob<string>('../App.svelte', { query: '?raw', import: 'default', eager: true }),
};
const UI = './ui/';
const uiFiles = Object.keys(sources).filter((path) => path.startsWith(UI));
const screens = Object.keys(sources).filter((path) => !path.startsWith(UI));

/** A top-level block of the front matter (`colors`, `typography`…), without its key line. */
function block(key: string): string {
  const match = frontMatter.match(new RegExp(`^${key}:\\n((?: .*\\n)+)`, 'm'));
  if (!match) throw new Error(`no ${key} block in DESIGN.md`);
  return match[1];
}

/** `  name: value` pairs of a flat block. */
const flat = (text: string): Record<string, string> =>
  Object.fromEntries(
    [...text.matchAll(/^ {2}([\w-]+): "?([^"\n]+)"?$/gm)].map((m) => [m[1], m[2]]),
  );

/** `  name:` followed by its `    key: value` lines. */
function nested(text: string): Record<string, Record<string, string>> {
  const result: Record<string, Record<string, string>> = {};
  for (const [, name, body] of text.matchAll(/^ {2}([\w-]+):\n((?: {4}.*\n)+)/gm))
    result[name] = Object.fromEntries(
      [...body.matchAll(/^ {4}(\w+): "?([^"\n]+)"?$/gm)].map((m) => [m[1], m[2]]),
    );
  return result;
}

interface Declaration {
  selector: string;
  property: string;
  value: string;
}
interface Node {
  type: string;
  start: number;
  end: number;
  property?: string;
  value?: string;
  prelude?: { start: number; end: number } | string;
  block?: { children: Node[] } | null;
}

/** Every declaration of a stylesheet with the selector of its rule, at-rules included. */
function declarations(text: string): Declaration[] {
  const found: Declaration[] = [];
  const walk = (nodes: Node[], selector: string): void => {
    for (const node of nodes) {
      if (node.type === 'Declaration')
        found.push({ selector, property: node.property ?? '', value: node.value ?? '' });
      else if (node.block) {
        const own =
          typeof node.prelude === 'object' ? text.slice(node.prelude.start, node.prelude.end) : '';
        walk(node.block.children, node.type === 'Rule' ? own : selector);
      }
    }
  };
  walk(parseCss(text).children as Node[], '');
  return found;
}

const sheet = declarations(css);
/** The variables of `:root`: the one place where values are written. */
const variables = Object.fromEntries(
  sheet
    .filter((d) => d.selector === ':root' && d.property.startsWith('--'))
    .map((d) => [d.property, d.value]),
);

/** WCAG 2 contrast ratio of two `#rrggbb` colours. */
function contrast(a: string, b: string): number {
  const luminance = (hex: string) => {
    const [r, g, b] = [1, 3, 5].map((i) => {
      const c = parseInt(hex.slice(i, i + 2), 16) / 255;
      return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
    });
    return 0.2126 * r + 0.7152 * g + 0.0722 * b;
  };
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
}

/** Spacing tokens whose variable is not `--space-<name>`. `null`: the value lives outside the CSS. */
const SIZES: Record<string, string | null> = {
  'rail-width': '--rail',
  'sidebar-width': '--sidebar',
  'list-width': '--list',
  'detail-min-width': '--detail-min',
  'content-max-width': '--content',
  'content-measure': '--measure',
  'control-height': '--control',
  'topbar-height': '--topbar',
  'palette-width': '--palette',
  'palette-top': '--palette-top',
  'palette-search-height': '--palette-search',
  'row-height': '--row',
  'command-item-height': '--command-item',
  'command-hit-height': '--command-hit',
  'setting-row-min-height': '--setting-row',
  'field-width': '--field',
  'icon-size': '--icon',
  indicator: '--indicator',
  'panel-gap': '--gap',
  'resize-hit-width': '--resize-hit',
  // features/layout/layout.ts (checked by layout.test.ts).
  'sidebar-min-width': null,
  'sidebar-max-width': null,
  'list-min-width': null,
  'list-max-width': null,
  'window-controls-width': '--window-controls',
  'icon-small-size': '--icon-small',
  'icon-stroke-width': '--icon-stroke',
  'status-dot-size': '--dot',
  'palette-list-max-height': '--palette-list',
  'textarea-min-height': '--textarea',
  'dialog-width': '--dialog',
  'tag-height': '--tag',
  'tag-field-width': '--tag-field',
  'notification-panel-width': '--notification-panel',
  'notification-panel-max-height': '--notification-panel-max',
  'status-item-max-width': '--status-item-max',
  // tauri.conf.json (`minWidth`).
  'window-min-width': null,
};
/** The variable of each colour token; a themed one is `light-dark(<-light token>, <token>)`. */
const COLOURS: Record<string, string> = {
  neutral: '--bg',
  surface: '--surface',
  'on-surface': '--fg',
  secondary: '--muted',
  outline: '--border',
  'field-outline': '--field-border',
  indicator: '--indicator-color',
  error: '--danger',
  success: '--ok',
  warning: '--warn',
  primary: '--accent',
  'primary-hover': '--accent-hover',
  'on-primary': '--on-accent',
};

describe('the tokens of app.css are those of DESIGN.md', () => {
  const colours = flat(block('colors'));

  it('colours, with both themes', () => {
    expect(Object.keys(colours).length).toBeGreaterThan(10);
    for (const [name, value] of Object.entries(colours)) {
      if (name.endsWith('-light')) continue;
      const light = colours[`${name}-light`];
      expect(variables[COLOURS[name]], name).toBe(light ? `light-dark(${light}, ${value})` : value);
    }
    // Every -light token belongs to a colour that exists.
    for (const name of Object.keys(colours))
      expect(colours, name).toHaveProperty(name.replace(/-light$/, ''));
  });

  it('the focus ring is the accent on dark and ink on light, where yellow has no contrast', () => {
    expect(variables['--focus']).toBe(
      `light-dark(${colours['on-surface-light']}, ${colours.primary})`,
    );
  });

  it('the thin indicator is the accent on dark and ink on light, like the focus ring', () => {
    expect(variables['--indicator-color']).toBe(
      `light-dark(${colours['on-surface-light']}, ${colours.primary})`,
    );
  });

  it('field borders reach 3:1 and the light warning 4.5:1 on the backgrounds they sit on', () => {
    for (const theme of ['', '-light']) {
      for (const ground of ['neutral', 'surface']) {
        const pair = `field-outline${theme} on ${ground}${theme}`;
        expect(
          contrast(colours[`field-outline${theme}`], colours[`${ground}${theme}`]),
          pair,
        ).toBeGreaterThanOrEqual(3);
      }
    }
    for (const ground of ['neutral-light', 'surface-light'])
      expect(contrast(colours['warning-light'], colours[ground]), ground).toBeGreaterThanOrEqual(
        4.5,
      );
  });

  it('typography, with its tracking', () => {
    const types = Object.entries(nested(block('typography')));
    expect(types.length).toBeGreaterThanOrEqual(9);
    for (const [name, type] of types) {
      const family = type.fontFamily.startsWith('ui-monospace') ? 'var(--mono)' : 'var(--sans)';
      expect(variables[`--type-${name}`], name).toBe(
        `${type.fontWeight} ${type.fontSize}/${type.lineHeight} ${family}`,
      );
      // Tracking is not part of the `font` shorthand: it has a variable of its own.
      if (type.letterSpacing)
        expect(variables[`--tracking-${name}`], name).toBe(type.letterSpacing);
    }
  });

  it('rounded and spacing', () => {
    for (const [name, value] of Object.entries(flat(block('rounded'))))
      if (value !== '0px') expect(variables[`--radius-${name}`], name).toBe(value);
    const sizes = Object.entries(flat(block('spacing')));
    expect(sizes.length).toBeGreaterThanOrEqual(Object.keys(SIZES).length);
    for (const [name, value] of sizes) {
      const variable = name in SIZES ? SIZES[name] : `--space-${name}`;
      if (variable !== null) expect(variables[variable], name).toBe(value);
    }
  });

  it('and app.css declares no variable that is not a token', () => {
    const types = nested(block('typography'));
    const known = new Set([
      ...Object.values(COLOURS),
      ...['--focus', '--sans', '--mono'],
      ...Object.keys(types).map((name) => `--type-${name}`),
      ...Object.keys(types).map((name) => `--tracking-${name}`),
      ...Object.keys(flat(block('rounded'))).map((name) => `--radius-${name}`),
      ...Object.keys(flat(block('spacing'))).map((name) => SIZES[name] ?? `--space-${name}`),
    ]);
    expect(Object.keys(variables).filter((name) => !known.has(name))).toEqual([]);
  });
});

describe('components', () => {
  it('thin 2px indicators take the indicator colour, ink on the light theme, not the yellow', () => {
    // A hover hint on the resize handle, not a marker of state: it stays yellow.
    const HOVER_HINTS = ['./ui/Resizer.svelte'];
    const off: string[] = [];
    for (const path of uiFiles.filter((p) => !HOVER_HINTS.includes(p))) {
      const rules = new Map<string, Declaration[]>();
      const style = sources[path].match(/<style[^>]*>([\s\S]*)<\/style>/)?.[1] ?? '';
      for (const d of declarations(style))
        rules.set(d.selector, [...(rules.get(d.selector) ?? []), d]);
      for (const [selector, declared] of rules) {
        const thin = declared.some((d) => d.value.includes('var(--indicator)'));
        const yellow = declared.some((d) => d.value.includes('var(--accent)'));
        if (thin && yellow) off.push(`${path}: ${selector}`);
      }
    }
    expect(off).toEqual([]);
  });

  it('only lib/ui draws controls: no screen has a raw button or field', () => {
    expect(screens).toContain('../App.svelte');
    for (const path of screens)
      expect(sources[path], path).not.toMatch(/<(button|input|select|textarea)\b/);
  });

  it('DESIGN.md names every lib/ui component, and every one it names exists', () => {
    const named = new Set([...design.matchAll(/`ui\/(\w+\.svelte)`/g)].map((m) => `${UI}${m[1]}`));
    expect(uiFiles.length).toBeGreaterThan(10);
    expect([...named].sort()).toEqual([...uiFiles].sort());
  });

  it('a style takes its values from tokens: no colour, font or length written by hand', () => {
    const styles = Object.entries(sources).map(([path, source]): [string, Declaration[]] => [
      path,
      declarations(source.match(/<style[^>]*>([\s\S]*)<\/style>/)?.[1] ?? ''),
    ]);
    styles.push(['app.css', sheet.filter((d) => d.selector !== ':root')]);

    const COLOUR = /#[0-9a-f]{3,8}\b|\b(rgba?|hsla?|hwb|oklch|oklab|lab|lch)\(/i;
    const FONT = /^(font|font-size|font-family|font-weight|font-style|line-height)$/;
    // With a number: the 1px hairline, fractions, percentages, the viewport, one line, times.
    const LENGTH = /^(1px|\d+fr|\d+%|100vw|100vh|1lh|\d+m?s)$/;
    const off: string[] = [];
    let seen = 0;
    for (const [path, declared] of styles)
      for (const { selector, property, value } of declared) {
        seen += 1;
        const where = `${path}: ${selector} { ${property}: ${value} }`;
        if (property.startsWith('--')) off.push(`${where} (a variable outside :root)`);
        if (COLOUR.test(value)) off.push(`${where} (colour)`);
        if (FONT.test(property) && !/^(inherit|var\(--type-[\w-]+\))$/.test(value))
          off.push(`${where} (font)`);
        // The focus ring is the one 2px line of the design.
        const lengths = value
          .replace('2px solid var(--focus)', '')
          .replace(/var\(--[\w-]+\)/g, '')
          .toLowerCase()
          .match(/\d*\.?\d+[a-z%]+/g);
        if (lengths?.some((length) => !LENGTH.test(length))) off.push(`${where} (length)`);
      }
    expect(seen).toBeGreaterThan(300);
    expect(off).toEqual([]);
  });
});
