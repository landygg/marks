import { defaultSanitizeUrl } from '@humanspeak/svelte-markdown';
import { describe, expect, it } from 'vitest';
import { callout } from './callout';
import { ALLOWED_TAGS, MEDIA_TAGS, linkAction, mediaLabel, sanitizeAttributes } from './reader';

describe('linkAction', () => {
  it('opens http and https links', () => {
    expect(linkAction('https://example.com/a')).toEqual({
      kind: 'open',
      url: 'https://example.com/a',
    });
    expect(linkAction('HTTP://example.com')).toEqual({ kind: 'open', url: 'HTTP://example.com' });
  });

  it('does nothing for an in-page fragment', () => {
    expect(linkAction('#section')).toEqual({ kind: 'none' });
  });

  it('shows mailto, tel, relative and empty hrefs as text', () => {
    for (const href of ['mailto:a@b.c', 'tel:+34600000000', '/path', 'page.html', '']) {
      expect(linkAction(href), href).toEqual({ kind: 'text' });
    }
  });
});

describe('sanitizeAttributes', () => {
  const sanitize = (attrs: Record<string, string>, tag: string) =>
    sanitizeAttributes(attrs, { type: 'html', tag }, defaultSanitizeUrl);

  it('keeps only the whitelisted attributes', () => {
    expect(
      sanitize(
        {
          href: 'https://example.com',
          title: 't',
          style: 'position: fixed',
          class: 'x',
          id: 'y',
          onclick: 'alert(1)',
          onToggle: 'alert(1)',
          srcset: 'https://example.com/a.png 2x',
          target: '_top',
        },
        'a',
      ),
    ).toEqual({ href: 'https://example.com', title: 't' });
  });

  it('keeps table, language and list attributes', () => {
    const attrs = { colspan: '2', rowspan: '3', lang: 'es', dir: 'rtl', open: '', start: '4' };
    expect(sanitize(attrs, 'td')).toEqual(attrs);
  });

  it('passes src and poster only on media tags', () => {
    const attrs = { src: 'https://example.com/v.mp4', poster: 'https://example.com/p.png' };
    for (const tag of MEDIA_TAGS) expect(sanitize(attrs, tag), tag).toEqual(attrs);
    expect(sanitize(attrs, 'div')).toEqual({});
  });

  it('still runs URLs through the library sanitizer', () => {
    expect(sanitize({ href: 'javascript:alert(1)', src: 'data:x' }, 'img')).toEqual({});
  });
});

describe('ALLOWED_TAGS', () => {
  it('has no tag that loads or submits anything', () => {
    for (const tag of [...MEDIA_TAGS, 'form', 'svg', 'math', 'object', 'script', 'style', 'base'])
      expect(ALLOWED_TAGS, tag).not.toContain(tag);
  });
});

describe('mediaLabel', () => {
  it('prefers the alt text, then the domain, then the fallback', () => {
    expect(mediaLabel(' Gato ', 'https://img.example.com/g.png', 'Imagen')).toBe('Gato');
    expect(mediaLabel('', 'https://img.example.com/g.png', 'Imagen')).toBe('img.example.com');
    expect(mediaLabel(undefined, '/rel.png', 'Imagen')).toBe('Imagen');
  });
});

describe('callout', () => {
  type Lexer = { blockTokens: (src: string, tokens: unknown[]) => unknown[] };
  // A stand-in lexer: records the body it is asked to tokenize.
  function tokenize(src: string) {
    const bodies: string[] = [];
    const lexer: Lexer = {
      blockTokens: (body, tokens) => {
        bodies.push(body);
        return tokens;
      },
    };
    const tokenizer = callout.tokenizer as unknown as (
      this: { lexer: Lexer },
      src: string,
    ) => unknown;
    return { token: tokenizer.call({ lexer }, src) as Record<string, unknown>, bodies };
  }

  it('reads the type and the title on the same line', () => {
    const { token, bodies } = tokenize('> [!note] Note\n> Body *text*\n\nAfter');
    expect(token).toMatchObject({
      type: 'callout',
      kind: 'note',
      title: 'Note',
      raw: '> [!note] Note\n> Body *text*\n',
    });
    expect(bodies).toEqual(['Body *text*']);
  });

  it('accepts any type, the fold mark and no title', () => {
    expect(tokenize('> [!warning]- Cuidado\n> x').token).toMatchObject({
      kind: 'warning',
      title: 'Cuidado',
    });
    expect(tokenize('> [!INFO]+\n> x').token).toMatchObject({ kind: 'info', title: '' });
  });

  it('leaves plain quotes alone', () => {
    expect(tokenize('> just a quote').token).toBeUndefined();
  });
});
