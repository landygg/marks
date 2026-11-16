// @vitest-environment jsdom
// Security boundary of the reader: bookmark content comes from arbitrary web pages.
import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { t } from '../../strings';
import ContentView from './ContentView.svelte';
import { openLink } from './libraryState.svelte';

vi.mock('./libraryState.svelte', () => ({ openLink: vi.fn() }));

let target: HTMLElement;
let view: ReturnType<typeof mount> | null = null;

function render(content: string): HTMLElement {
  view = mount(ContentView, { target, props: { content } });
  flushSync();
  return target.querySelector('article') ?? target;
}

beforeEach(() => {
  target = document.createElement('div');
  document.body.append(target);
  vi.mocked(openLink).mockClear();
});

afterEach(() => {
  if (view) unmount(view);
  view = null;
  target.remove();
});

describe('ContentView never lets through', () => {
  it.each([
    ['script', 'a\n\n<script>alert(1)</script>'],
    ['img', '<img src="https://example.com/x.png" onerror="alert(1)">'],
    ['img', '![x](https://example.com/x.png)'],
    ['iframe', '<iframe src="https://www.youtube.com/embed/x"></iframe>'],
    ['video', '<video src="https://example.com/v.mp4" poster="https://example.com/p.png"></video>'],
    ['audio', '<audio src="https://example.com/a.mp3"></audio>'],
    ['embed', '<embed src="https://example.com/x.swf">'],
    ['picture', '<picture><source srcset="https://example.com/a.webp"><img src="a.png"></picture>'],
    ['form', '<form action="https://evil.example"><input name="q"><button>Go</button></form>'],
    ['svg', '<svg><a href="javascript:alert(1)"><text>x</text></a></svg>'],
    ['math', '<math><mi>x</mi></math>'],
    ['base', '<base href="https://evil.example/">'],
    ['meta', '<meta http-equiv="refresh" content="0;url=https://evil.example">'],
    ['object', '<object data="https://example.com/x.pdf"></object>'],
    ['style', '<style>body { display: none }</style>'],
  ])('<%s>', (tag, source) => {
    const article = render(source);
    expect(article.querySelector(tag)).toBeNull();
    expect(article.querySelector('[src],[srcset],[poster],[action],[data]')).toBeNull();
  });

  it('event handlers, style, class and id attributes', () => {
    const article = render(
      '<p onclick="alert(1)" style="position: fixed" class="app" id="main">a</p>\n\n' +
        '<details ontoggle="alert(1)" open><summary>s</summary>b</details>\n\n# Título',
    );
    for (const el of article.querySelectorAll('*'))
      for (const { name } of el.attributes) expect(name, el.tagName).not.toMatch(/^on|^style$/);
    expect(article.querySelector('p')?.textContent).toBe('a');
    expect(article.querySelector('[class="app"],[id]')).toBeNull();
  });

  it.each([
    'javascript:alert(1)',
    'JaVaScRiPt:alert(1)',
    'java\tscript:alert(1)',
    'data:text/html,<script>alert(1)</script>',
    'vbscript:msgbox(1)',
  ])('a %s link', (href) => {
    const article = render(`[x](${href}) <a href="${href}">y</a>`);
    for (const a of article.querySelectorAll('a')) a.click();
    expect(article.querySelector('a')).toBeNull();
    expect(openLink).not.toHaveBeenCalled();
    expect(article.textContent).toContain('x');
  });
});

describe('ContentView renders', () => {
  it('http(s) links, Markdown and HTML, through openLink', () => {
    const article = render('[md](https://example.com/a) <a href="http://example.com/b">html</a>');
    const links = [...article.querySelectorAll('a')];
    expect(links.map((a) => a.getAttribute('href'))).toEqual([
      'https://example.com/a',
      'http://example.com/b',
    ]);
    for (const a of links) a.click();
    expect(vi.mocked(openLink).mock.calls).toEqual([
      ['https://example.com/a'],
      ['http://example.com/b'],
    ]);
  });

  it('fragments, mailto and relative links as plain text', () => {
    const article = render('[frag](#x) [mail](mailto:a@b.c) [rel](/path)');
    expect(article.querySelector('a')).toBeNull();
    expect(article.textContent).toContain('frag mail rel');
  });

  it('media as one line that opens the URL', () => {
    const article = render('![Gato](https://img.example.com/g.png)');
    expect(article.textContent).toContain('Gato');
    const open = article.querySelector('button');
    expect(open?.getAttribute('aria-label')).toBe(t.mediaOpenNamed('Gato'));
    open?.click();
    expect(openLink).toHaveBeenCalledWith('https://img.example.com/g.png');
  });

  it.each([
    ['Markdown', '[![Gato](https://e.example/i.png)](https://e.example/page)'],
    ['HTML', '<a href="https://e.example/page"><img src="https://e.example/i.png" alt="Gato"></a>'],
  ])('a linked image (%s) as one media line that opens the link once', (_, source) => {
    const article = render(source);
    expect(article.querySelector('a')).toBeNull();
    article.querySelector('button')?.click();
    expect(vi.mocked(openLink).mock.calls).toEqual([['https://e.example/page']]);
  });

  it('a linked image whose link cannot open, opening the image', () => {
    render('[![Gato](https://e.example/i.png)](#frag)').querySelector('button')?.click();
    expect(vi.mocked(openLink).mock.calls).toEqual([['https://e.example/i.png']]);
  });

  it('no button inside a link anywhere', () => {
    const article = render(
      'Ver [![a](https://e.example/a.png) y texto](https://e.example/p) y ' +
        '<a href="https://e.example/q"><span><video src="https://e.example/v.mp4"></video></span></a>\n\n' +
        '| c |\n|---|\n| [![b](https://e.example/b.png)](https://e.example/r) |',
    );
    expect(article.querySelectorAll('button')).toHaveLength(3);
    expect(article.querySelector('a button')).toBeNull();
  });

  it('footnotes with their label and live links', () => {
    const article = render('Texto[^1].\n\n[^1]: Ver [fuente](https://example.com/f).');
    const notes = article.querySelector('.footnotes');
    expect(notes?.textContent).toContain('1');
    expect(notes?.textContent).toContain('Ver fuente.');
    notes?.querySelector('a')?.click();
    expect(openLink).toHaveBeenCalledWith('https://example.com/f');
    expect(article.querySelector('[id],[class="footnote-ref"]')).toBeNull();
  });

  it('footnotes separated by blank lines, as Defuddle writes them, one line under the text', () => {
    const article = render('a[^1] b[^2]\n\n[^1]: Uno.\n\n[^2]: Dos.');
    expect([...article.querySelectorAll('.footnotes p')].map((p) => p.textContent)).toEqual([
      '1 Uno.',
      '2 Dos.',
    ]);
  });

  it('callouts as a quote with their type', () => {
    const article = render('> [!info] Info\n> Cuerpo **fuerte**');
    const quote = article.querySelector('blockquote');
    expect(quote?.querySelector('.section-label')?.textContent).toBe('info');
    expect(quote?.textContent).toContain('Info');
    expect([...(quote?.querySelectorAll('strong') ?? [])].map((s) => s.textContent)).toContain(
      'fuerte',
    );
  });

  it('code spans keeping their backticks', () => {
    expect(render('``a`b``').querySelector('code')?.textContent).toBe('a`b');
  });

  it('code blocks with their language as text and no class', () => {
    const article = render('```rust\nfn main() {}\n```');
    expect(article.querySelector('pre code')?.textContent).toBe('fn main() {}');
    expect(article.querySelector('pre')?.textContent).toContain('rust');
    expect(article.querySelector('code[class]')).toBeNull();
  });

  it('task lists with a glyph and no field', () => {
    const article = render('- [x] hecha\n- [ ] pendiente');
    expect(article.querySelector('input')).toBeNull();
    const glyphs = [...article.querySelectorAll('[role="img"]')];
    expect(glyphs.map((g) => [g.textContent, g.getAttribute('aria-label')])).toEqual([
      ['☑', t.taskDone],
      ['☐', t.taskOpen],
    ]);
  });

  it('highlights and formulas as literal text', () => {
    const article = render('$x$ y ==m==');
    expect(article.textContent).toContain('$x$ y ==m==');
    expect(article.querySelector('mark')).toBeNull();
  });

  it('tags without a library component', () => {
    const article = render(
      '<figure><figcaption>Pie</figcaption></figure>\n\n<p><ins>i</ins> <q>q</q></p>\n\n' +
        '<table><caption>Cap</caption><tr><td colspan="2">c</td></tr></table>',
    );
    for (const tag of ['figure', 'figcaption', 'ins', 'q', 'caption'])
      expect(article.querySelector(tag), tag).not.toBeNull();
    expect(article.querySelector('td')?.getAttribute('colspan')).toBe('2');
  });

  it('the empty state without content', () => {
    view = mount(ContentView, { target, props: { content: null } });
    flushSync();
    expect(target.textContent).toContain(t.noContent);
  });
});

describe('ContentView survives', () => {
  it('nesting too deep to render, showing the source as plain text', () => {
    const source = `${'> '.repeat(300)}hondo <b>x</b>`;
    expect(() => render(source)).not.toThrow();
    expect(target.textContent).toContain(t.contentUnrenderable);
    expect(target.querySelector('article')?.textContent).toBe(source);
    expect(target.querySelector('blockquote, b')).toBeNull();
  });
});
