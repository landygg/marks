import Defuddle from 'defuddle/full';
import type { DefuddleResponse } from 'defuddle/full';
import pkg from '../package.json';

export interface ExtractedMeta {
  description: string | null;
  author: string | null;
  site: string | null;
  published: string | null;
  language: string | null;
  image: string | null;
  favicon: string | null;
  word_count: number | null;
  extractor: string;
}
export interface Extracted {
  title: string | null;
  markdown: string;
  meta: ExtractedMeta;
}

export const EXTRACTOR = `defuddle@${pkg.dependencies.defuddle}`;
// Defuddle puts this text in `content` when the HTML-to-Markdown conversion fails midway.
export const PARTIAL_CONVERSION_PREFIX = 'Partial conversion completed with errors'; // i18n-allow: detection

// Defuddle's site extractors (YouTube, Reddit, X) would fetch; the page must stay offline.
const rejectAll = () => Promise.reject(new Error('marks: no network')); // i18n-allow: invariant

const text = (v: string | undefined | null): string | null => v?.trim() || null;

function absolute(value: string | undefined | null, base: string): string | null {
  const v = text(value);
  if (!v) return null;
  try {
    const u = new URL(v, base);
    return u.protocol === 'http:' || u.protocol === 'https:' ? u.href : null;
  } catch {
    return null;
  }
}

export function toExtracted(r: DefuddleResponse, url: string): Extracted {
  const content = r.content ?? '';
  return {
    title: text(r.title),
    markdown: content.startsWith(PARTIAL_CONVERSION_PREFIX) ? '' : content.trim(),
    meta: {
      description: text(r.description),
      author: text(r.author),
      site: text(r.site),
      published: text(r.published),
      language: text(r.language),
      image: absolute(r.image, url),
      favicon: absolute(r.favicon, url),
      word_count: r.wordCount ?? null,
      extractor: EXTRACTOR,
    },
  };
}

export function extractDocument(doc: Document, url: string): Extracted {
  const r = new Defuddle(doc, { url, markdown: true, useAsync: false, fetch: rejectAll }).parse();
  return toExtracted(r, url);
}

export function extractHtml(html: string, url: string): Extracted {
  return extractDocument(new DOMParser().parseFromString(html, 'text/html'), url);
}

// Input for the AI extraction: content tags only, no code, styling or embedded objects.
const DROPPED_TAGS =
  'script, style, noscript, template, svg, math, iframe, object, embed, canvas, link, meta, head';
const KEPT_ATTRIBUTES = new Set(['href', 'src', 'alt', 'title', 'colspan', 'rowspan']);
const URL_ATTRIBUTES = new Set(['href', 'src']);

// DOMParser builds an inert document: no scripts run and nothing is fetched.
export function cleanHtml(html: string, url: string): string {
  const doc = new DOMParser().parseFromString(html, 'text/html');
  doc.querySelectorAll(DROPPED_TAGS).forEach((e) => e.remove());
  const comments = doc.createTreeWalker(doc, NodeFilter.SHOW_COMMENT);
  const dead: Node[] = [];
  while (comments.nextNode()) dead.push(comments.currentNode);
  dead.forEach((c) => c.parentNode?.removeChild(c));
  for (const el of doc.body.querySelectorAll('*')) {
    for (const name of el.getAttributeNames()) {
      if (!KEPT_ATTRIBUTES.has(name)) {
        el.removeAttribute(name);
      } else if (URL_ATTRIBUTES.has(name)) {
        const abs = absolute(el.getAttribute(name), url);
        if (abs) el.setAttribute(name, abs);
        else el.removeAttribute(name);
      }
    }
  }
  // Collapsed text node by text node: whitespace inside <pre> is the code's layout. Merged first,
  // or the spaces left around each removed element would stay side by side.
  doc.body.normalize();
  const texts = doc.createTreeWalker(doc.body, NodeFilter.SHOW_TEXT);
  while (texts.nextNode()) {
    const text = texts.currentNode as Text;
    if (!text.parentElement?.closest('pre')) text.data = text.data.replace(/\s+/g, ' ');
  }
  return doc.body.innerHTML.trim();
}
