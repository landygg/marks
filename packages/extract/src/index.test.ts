import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it, vi } from 'vitest';
import type { DefuddleResponse } from 'defuddle/full';
import {
  EXTRACTOR,
  PARTIAL_CONVERSION_PREFIX,
  cleanHtml,
  extractHtml,
  toExtracted,
} from './index';

const fixture = (name: string) =>
  readFileSync(resolve('fixtures', name), 'utf8');
const base = {
  title: 'T',
  content: 'body',
  author: 'A',
  site: 'S',
  published: '',
  language: 'es',
  description: 'D',
  image: '',
  favicon: '',
  wordCount: 1,
} as unknown as DefuddleResponse;

describe('extractHtml', () => {
  it('article_keeps_footnotes_code_and_table', () => {
    const r = extractHtml(fixture('article.html'), 'https://example.com/blog/p');
    expect(r.markdown).toContain('[^1]');
    expect(r.markdown).toContain('```rust');
    expect(r.markdown).toContain('| Tipo |');
    expect(r.title).toBe('Propiedad en Rust');
    expect(r.meta.author).toBe('Ana Pérez');
    expect(r.meta.site).toBeTruthy();
    expect(r.meta.published).toBeTruthy();
    expect(r.meta.language).toBe('es');
  });

  it('page_without_article_keeps_metadata', () => {
    const r = extractHtml(fixture('no-article.html'), 'https://example.com/x');
    expect(r.markdown.length).toBeLessThan(200);
    expect(r.meta.description).not.toBeNull();
  });

  it('relative_links_become_absolute', () => {
    const r = extractHtml(fixture('article.html'), 'https://example.com/blog/p');
    expect(r.markdown).toContain('[propiedad](https://example.com/docs/ownership)');
  });

  it('unsafe_html_never_reaches_markdown', () => {
    const md = extractHtml(fixture('unsafe.html'), 'https://example.com/u').markdown;
    for (const bad of ['javascript:', 'onclick', '<script', 'alert(']) {
      expect(md).not.toContain(bad);
    }
  });

  it('never_fetches', () => {
    const spy = vi.spyOn(globalThis, 'fetch');
    for (const url of [
      'https://www.youtube.com/watch?v=x',
      'https://www.reddit.com/r/x/comments/y',
      'https://x.com/a/status/1',
    ]) {
      extractHtml(fixture('article.html'), url);
    }
    expect(spy).not.toHaveBeenCalled();
  });

  it('contract_fixture_matches', () => {
    const r = extractHtml(fixture('contract.html'), 'https://example.com/contrato');
    expect(r).toEqual(JSON.parse(fixture('extracted.json')));
  });
});

describe('toExtracted', () => {
  it('meta_urls_resolve_against_the_page', () => {
    const url = 'https://example.com/a/b';
    const f = (favicon: string, image: string) => toExtracted({ ...base, favicon, image }, url).meta;
    expect(f('/favicon.ico', '//cdn.example.com/i.png')).toMatchObject({
      favicon: 'https://example.com/favicon.ico',
      image: 'https://cdn.example.com/i.png',
    });
    expect(f('notaurl::', '').favicon).toBeNull();
  });

  it('empty_strings_become_null', () => {
    expect(toExtracted({ ...base, author: '  ' }, 'https://example.com/').meta.author).toBeNull();
  });

  it('partial_conversion_becomes_empty', () => {
    const r = toExtracted({ ...base, content: `${PARTIAL_CONVERSION_PREFIX}: boom` }, 'https://example.com/');
    expect(r.markdown).toBe('');
  });

  it('extractor_names_the_pinned_defuddle', () => {
    const pkg = JSON.parse(readFileSync(resolve('node_modules/defuddle/package.json'), 'utf8'));
    expect(EXTRACTOR).toBe('defuddle@0.19.6');
    expect(EXTRACTOR).toBe(`defuddle@${pkg.version}`);
  });
});

describe('cleanHtml', () => {
  const url = 'https://example.com/blog/p';
  const clean = () => cleanHtml(fixture('dirty.html'), url);
  const parse = (html: string) => new DOMParser().parseFromString(html, 'text/html').body;

  it('clean_html_drops_scripts_styles_svg_and_comments', () => {
    const html = clean();
    for (const bad of [
      '<script', '<style', '<noscript', '<template', '<svg', '<math', '<iframe',
      '<object', '<embed', '<canvas', '<link', '<meta', '<!--', 'tracker', 'plantilla',
    ]) {
      expect(html).not.toContain(bad);
    }
    expect(html).toContain('Página sucia');
    expect(html).toContain('celda');
  });

  it('clean_html_keeps_only_whitelisted_attributes', () => {
    const body = parse(clean());
    const names = new Set([...body.querySelectorAll('*')].flatMap((e) => e.getAttributeNames()));
    expect([...names].sort()).toEqual(['alt', 'colspan', 'href', 'rowspan', 'src', 'title']);
    expect(body.querySelector('h1')?.getAttribute('title')).toBe('Título');
    expect(body.querySelector('td')?.getAttribute('colspan')).toBe('2');
  });

  it('clean_html_absolutizes_and_drops_non_http_links', () => {
    const body = parse(clean());
    const hrefs = [...body.querySelectorAll('a')].map((a) => a.getAttribute('href'));
    expect(hrefs).toEqual([
      'https://example.com/inicio',
      'https://example.com/doc/otra',
      'https://cdn.example.com/x',
      null,
      null,
    ]);
    const srcs = [...body.querySelectorAll('img')].map((i) => i.getAttribute('src'));
    expect(srcs).toEqual(['https://example.com/blog/img/foto.png', null]);
    expect(clean()).not.toMatch(/javascript:|data:/);
  });

  it('clean_html_collapses_whitespace_outside_pre', () => {
    const outsidePre = clean().replace(/<pre>[\s\S]*?<\/pre>/g, '<pre></pre>');
    expect(outsidePre).not.toMatch(/\s{2,}/);
  });

  it('clean_html_keeps_whitespace_inside_pre', () => {
    const pre = parse(clean()).querySelector('pre');
    expect(pre?.textContent).toBe('fn main() {\n    println!("hola");\n}');
  });

  it('clean_html_never_fetches', () => {
    const spy = vi.spyOn(globalThis, 'fetch');
    clean();
    expect(spy).not.toHaveBeenCalled();
  });
});
