import { execFileSync } from 'node:child_process';
import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { parse } from 'svelte/compiler';
import ts from 'typescript';
import { describe, expect, it } from 'vitest';

// Guard: no user-facing text outside the locale files. Same rule as
// crates/i18n/tests/no_user_literals.rs; the catalogs are the only place for text.

const ROOT = fileURLToPath(new URL('../../../', import.meta.url));
const SCAN_DIRS = ['apps/desktop/src', 'apps/extension', 'packages/extract/src'];
/** The app's own pages live next to `src`, not in it. */
const EXTRA_FILES = ['apps/desktop/index.html', 'apps/desktop/extract.html'];
const SKIP_DIRS = new Set(['node_modules', 'dist', 'fixtures']);
const CATALOGS = new Set(['apps/desktop/src/lib/i18n/es.ts', 'apps/desktop/src/lib/i18n/en.ts']);
const EXTENSIONS = /\.(ts|js|svelte|html|swift)$/;
const TESTS = /\.test\.(ts|js)$/;
/** Attributes and props the user reads; `class`, `style`, `href`, `data-*`… never are. */
const USER_ATTRS = new Set([
  'title',
  'alt',
  'placeholder',
  'label',
  'aria-label',
  'aria-description',
  'aria-valuetext',
  'aria-roledescription',
]);
/** Never looked at, not even their expressions. */
const STYLE_ATTRS = new Set(['class', 'style']);
const STYLE_DIRECTIVES = new Set(['ClassDirective', 'StyleDirective']);
const SQL_WORDS = new Set([
  'SELECT',
  'INSERT',
  'UPDATE',
  'DELETE',
  'CREATE',
  'DROP',
  'ALTER',
  'PRAGMA',
  'WITH',
]);
const MARKER = 'i18n-allow:';
const FORBIDDEN_REASON = 'temporal';
/** Bundles whose name the system shows, with the folder of their `<locale>.lproj`. */
const BUNDLES = [
  { plist: 'apps/desktop/src-tauri/Info.plist', lproj: 'apps/desktop/src-tauri/infoplist' },
  { plist: 'apps/extension/safari/Info.plist', lproj: 'apps/extension/safari' },
];
const BUNDLE_KEYS = ['CFBundleDisplayName', 'CFBundleName'];

/** Drops the `{…}` placeholders, nested ones included. */
function stripBraces(s: string): string {
  let depth = 0;
  let out = '';
  for (const c of s) {
    if (c === '{') depth++;
    else if (c === '}' && depth > 0) depth--;
    else if (depth === 0) out += c;
  }
  return out;
}

function isUserText(raw: string): boolean {
  const s = stripBraces(raw).trim();
  if (SQL_WORDS.has(s.split(/\s+/)[0])) return false;
  return (
    /(?!\p{ASCII})\p{Alphabetic}/u.test(s) ||
    /\p{Alphabetic}\s+\p{Alphabetic}/u.test(s) ||
    /^\p{P}*\p{Uppercase}\p{Lowercase}+\p{P}*$/u.test(s)
  );
}

/** Markup text and user-visible attribute values: any letter is text, not only words. */
const isMarkupText = (raw: string) => /\p{Alphabetic}/u.test(stripBraces(raw));

/** The reason of an `i18n-allow:` marker on the line, `''` if missing, `undefined` if none. */
function allowReason(line: string): string | undefined {
  const at = line.indexOf(MARKER);
  if (at < 0) return undefined;
  return line
    .slice(at + MARKER.length)
    .trim()
    .replace(/(-->|\*\/)$/, '')
    .trim();
}

const isValidReason = (reason: string | undefined) =>
  reason !== undefined && reason !== '' && reason !== FORBIDDEN_REASON;

function files(dir: string): string[] {
  return readdirSync(join(ROOT, dir), { withFileTypes: true }).flatMap((entry) => {
    const path = `${dir}/${entry.name}`;
    if (entry.isDirectory()) return SKIP_DIRS.has(entry.name) ? [] : files(path);
    return EXTENSIONS.test(path) && !TESTS.test(path) && !CATALOGS.has(path) ? [path] : [];
  });
}

/** One file's text with what each scanner needs: lines, offset → line, and the hits. */
class Source {
  readonly lines: string[];
  readonly hits: string[] = [];
  /** Lines whose marker let some literal through. */
  readonly used = new Set<number>();
  private readonly starts: number[] = [0];

  constructor(
    readonly path: string,
    readonly text: string,
  ) {
    this.lines = text.split('\n');
    for (let i = 0; i < text.length; i++) if (text[i] === '\n') this.starts.push(i + 1);
  }

  /** 1-based line of a character offset. */
  line(offset: number): number {
    let lo = 0;
    let hi = this.starts.length - 1;
    while (lo < hi) {
      const mid = (lo + hi + 1) >> 1;
      if (this.starts[mid] <= offset) lo = mid;
      else hi = mid - 1;
    }
    return lo + 1;
  }

  private allowed(line: number): boolean {
    return isValidReason(allowReason(this.lines[line - 1] ?? ''));
  }

  /**
   * `decl`: line of the `const`/`let` that declares the literal, whose marker also counts.
   * `markup`: text the user reads where it stands (`isMarkupText`).
   */
  check(value: string, line: number, decl?: number, markup = false): void {
    if (!(markup ? isMarkupText(value) : isUserText(value))) return;
    const marked = [line, decl].find((l) => l !== undefined && this.allowed(l));
    if (marked !== undefined) this.used.add(marked);
    else this.hits.push(`${this.path}:${line}: ${JSON.stringify(value.trim())}`);
  }

  /** Markers without a reason, or that let nothing through; and how many markers there are. */
  markers(): { count: number; problems: string[] } {
    const problems: string[] = [];
    let count = 0;
    this.lines.forEach((line, i) => {
      const reason = allowReason(line);
      if (reason === undefined) return;
      count++;
      if (!isValidReason(reason))
        problems.push(`${this.path}:${i + 1}: i18n-allow without a reason`);
      else if (!this.used.has(i + 1))
        problems.push(`${this.path}:${i + 1}: i18n-allow that lets nothing through`);
    });
    return { count, problems };
  }
}

const isConsoleCall = (node: ts.Node) =>
  ts.isCallExpression(node) &&
  ts.isPropertyAccessExpression(node.expression) &&
  ts.isIdentifier(node.expression.expression) &&
  node.expression.expression.text === 'console';

/** `.ts`/`.js`, or a Svelte `<script>` starting at character `offset` of the file. */
function scanScript(src: Source, code: string, offset = 0): void {
  const kind = src.path.endsWith('.js') ? ts.ScriptKind.JS : ts.ScriptKind.TS;
  const file = ts.createSourceFile(src.path, code, ts.ScriptTarget.Latest, true, kind);
  const lineOf = (node: ts.Node) => src.line(offset + node.getStart(file));
  const declOf = (node: ts.Node) => {
    for (let n = node.parent; n; n = n.parent) if (ts.isVariableStatement(n)) return lineOf(n);
  };
  const visit = (node: ts.Node): void => {
    if (isConsoleCall(node)) return;
    if (ts.isStringLiteral(node) || ts.isNoSubstitutionTemplateLiteral(node)) {
      src.check(node.text, lineOf(node), declOf(node));
    } else if (ts.isTemplateExpression(node)) {
      for (const part of [node.head, ...node.templateSpans.map((s) => s.literal)])
        src.check(part.text, lineOf(part), declOf(node));
    }
    ts.forEachChild(node, visit);
  };
  visit(file);
}

/** A node of Svelte's modern AST, read loosely: only `type`, offsets and children matter. */
type SvelteNode = { type?: string; start: number; [key: string]: unknown };

function scanSvelte(src: Source): void {
  const ast = parse(src.text, { modern: true });
  for (const script of [ast.instance, ast.module]) {
    if (!script) continue;
    const { start, end } = script.content as unknown as SvelteNode & { end: number };
    scanScript(src, src.text.slice(start, end), start);
  }
  const text = (node: SvelteNode & { data: string; raw: string }, markup: boolean) => {
    const lead = node.raw.length - node.raw.trimStart().length;
    src.check(node.data, src.line(node.start + lead), undefined, markup);
  };
  /**
   * `prop`: inside the attributes of a component, whose static props are all read.
   * `markup`: a `Text` here is read as it stands (markup, or a user-visible attribute).
   */
  const walk = (value: unknown, prop = false, markup = true): void => {
    if (Array.isArray(value)) return value.forEach((child) => walk(child, prop, markup));
    if (!value || typeof value !== 'object') return;
    const node = value as SvelteNode;
    if (node.type === 'Text')
      return text(node as SvelteNode & { data: string; raw: string }, markup);
    if (node.type === 'Attribute') {
      const name = String(node.name);
      if (STYLE_ATTRS.has(name)) return;
      // Static text only where the user reads it; expressions (handlers, values…) always.
      const shown = USER_ATTRS.has(name) || name.endsWith('Label');
      const parts = Array.isArray(node.value) ? node.value : [node.value];
      for (const part of parts)
        if (prop || shown || (part as SvelteNode)?.type !== 'Text') walk(part, false, shown);
      return;
    }
    if (node.type && STYLE_DIRECTIVES.has(node.type)) return;
    if (node.type === 'Literal' && typeof node.value === 'string')
      return src.check(node.value, src.line(node.start));
    if (node.type === 'TemplateElement')
      return src.check(String((node.value as { cooked: string }).cooked), src.line(node.start));
    for (const [key, child] of Object.entries(node))
      if (key !== 'metadata') walk(child, key === 'attributes' && node.type === 'Component');
  };
  walk(ast.fragment);
}

/** Blanks a match but keeps its newlines, so the lines after it do not move. */
const blank = (match: string) => match.replace(/[^\n]/g, ' ');

function scanHtml(src: Source): void {
  const text = src.text
    .replace(/<!--[\s\S]*?-->/g, blank)
    .replace(/<(script|style)\b[\s\S]*?<\/\1>/g, blank);
  // Prettier moves a trailing HTML comment to its own line, so a marker on the line above counts.
  const check = (value: string, line: number) => src.check(value, line, line - 1, true);
  for (const m of text.matchAll(/>([^<]+)</g)) check(m[1], src.line(m.index + 1));
  for (const m of text.matchAll(/\s([\w-]+)="([^"]*)"/g))
    if (USER_ATTRS.has(m[1])) check(m[2], src.line(m.index));
}

/** `"…"` literals outside `//` comment lines, with each `\(…)` interpolation as a placeholder. */
function scanSwift(src: Source): void {
  src.lines.forEach((line, i) => {
    if (line.trim().startsWith('//')) return;
    for (const m of line.matchAll(/"((?:[^"\\]|\\.)*)"/g))
      src.check(m[1].replace(/\\\((?:[^()]|\([^()]*\))*\)/g, '{}'), i + 1);
  });
}

function scanText(path: string, text: string): Source {
  const src = new Source(path, text);
  if (path.endsWith('.svelte')) scanSvelte(src);
  else if (path.endsWith('.html')) scanHtml(src);
  else if (path.endsWith('.swift')) scanSwift(src);
  else scanScript(src, src.text);
  return src;
}

const scan = (path: string) => scanText(path, readFileSync(join(ROOT, path), 'utf8'));

const plist = (path: string) =>
  JSON.parse(
    execFileSync('plutil', ['-convert', 'json', '-o', '-', join(ROOT, path)], { encoding: 'utf8' }),
  );

describe('i18n guard', () => {
  it('tells user text from identifiers', () => {
    for (const s of [
      'perfil no encontrado',
      'Ajustes',
      'Añadir URL…',
      'Open',
      'Resultado:',
      '¿Seguro?',
    ])
      expect(isUserText(s), s).toBe(true);
    for (const s of [
      'library-changed',
      'pending_embed',
      'HTML',
      'main',
      '⌘N',
      '{n}',
      'SELECT id FROM t',
      '/v1/ping',
    ])
      expect(isUserText(s), s).toBe(false);
  });

  it('needs a reason on every marker', () => {
    expect(allowReason('x // i18n-allow: formato')).toBe('formato');
    expect(allowReason('<!-- i18n-allow: formato -->')).toBe('formato');
    expect(isValidReason(allowReason('x // i18n-allow:'))).toBe(false);
    expect(isValidReason(allowReason('x // i18n-allow: temporal'))).toBe(false);
    expect(allowReason('x // nothing')).toBeUndefined();
  });

  it('reads handlers, directives and component props in Svelte markup', () => {
    const hits = (markup: string) => scanText('probe.svelte', markup).hits.length;
    expect(hits(`<button onclick={() => report('No se pudo guardar')}></button>`)).toBe(1);
    expect(hits(`<div use:tip={'Texto'}></div>`)).toBe(1);
    expect(hits(`<InlineConfirm question="¿Borrar?" />`)).toBe(1);
    expect(
      hits(`<div class="Hola mundo" style="Uno dos" class:Activo={x} data-x="Hola mundo"></div>`),
    ).toBe(0);
  });

  it('takes any word in markup text and user-visible attributes as user text', () => {
    const hits = (path: string, markup: string) => scanText(path, markup).hits.length;
    expect(hits('probe.svelte', '<p>listo</p>')).toBe(1);
    expect(hits('probe.svelte', '<button title="failed"></button>')).toBe(1);
    expect(hits('probe.svelte', '<input placeholder="url" />')).toBe(1);
    expect(hits('probe.svelte', '<Field ariaLabel="url" />')).toBe(1);
    expect(hits('probe.html', '<p>listo</p><input placeholder="url" />')).toBe(2);
    expect(hits('probe.html', '<!-- i18n-allow: nombre propio -->\n<title>marks</title>')).toBe(0);
    // Elsewhere the general rule still holds: a lone identifier is not text.
    expect(hits('probe.svelte', '<p>{n} · {m}</p><Confirm tone="neutral" />')).toBe(0);
  });

  it('reads the fixed text around Swift interpolations', () => {
    expect(scanText('probe.swift', 'let s = "No se pudo abrir \\(path)"').hits).toHaveLength(1);
  });

  it('flags markers that let nothing through', () => {
    const src = scanText('probe.ts', "const a = 'id'; // i18n-allow: formato\n");
    expect(src.markers().problems).toHaveLength(1);
  });

  it('finds no user text outside the catalogs', () => {
    const sources = [...SCAN_DIRS.flatMap(files), ...EXTRA_FILES].map(scan);
    const hits = sources.flatMap((src) => src.hits);
    let markers = 0;
    for (const src of sources) {
      const { count, problems } = src.markers();
      markers += count;
      hits.push(...problems);
    }
    // stderr, not console: vitest's agent reporter hides the console of passing tests.
    process.stderr.write(`i18n-allow: ${markers}\n`);
    expect(hits).toEqual([]);
  });

  it.skipIf(process.platform !== 'darwin')('names every bundle in each of its locales', () => {
    for (const { plist: path, lproj } of BUNDLES) {
      const locales: string[] = plist(path).CFBundleLocalizations;
      expect(locales.length, path).toBeGreaterThan(0);
      for (const locale of locales) {
        const strings = plist(`${lproj}/${locale}.lproj/InfoPlist.strings`);
        for (const key of BUNDLE_KEYS)
          expect(strings[key], `${lproj} ${locale} ${key}`).toBeTruthy();
      }
    }
  });
});
