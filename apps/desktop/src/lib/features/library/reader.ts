// Reader configuration for `ContentView`: what bookmark content (arbitrary web pages turned into
// Markdown) may put in the DOM. The tag list, the attribute list, the URL policy and the media
// renderers are four separate layers; the CSP sits behind them.
import {
  defaultSanitizeAttributes,
  type MarkedExtension,
  type SanitizeContext,
  type SanitizeUrlFn,
  type Token,
} from '@humanspeak/svelte-markdown';
// The subpath, not `…/extensions`: the index pulls in KaTeX and Mermaid.
import { markedFootnote } from '@humanspeak/svelte-markdown/extensions/footnote';
import { callout } from './callout';

/** Text and structure only. Anything else renders its children as text, never as an element. */
export const ALLOWED_TAGS: readonly string[] = [
  ...['p', 'br', 'hr', 'div', 'span', 'b', 'strong', 'i', 'em', 'u', 's', 'del', 'ins', 'small'],
  ...['sup', 'sub', 'abbr', 'cite', 'q', 'blockquote', 'code', 'pre', 'kbd', 'samp', 'var'],
  ...['ul', 'ol', 'li', 'dl', 'dt', 'dd', 'details', 'summary', 'figure', 'figcaption'],
  ...['table', 'thead', 'tbody', 'tfoot', 'tr', 'th', 'td', 'caption'],
  ...['h1', 'h2', 'h3', 'h4', 'h5', 'h6'],
  'a',
];

/** Tags that would load something: they become a `ContentMedia` line instead. */
export const MEDIA_TAGS = ['img', 'picture', 'iframe', 'video', 'audio', 'embed'] as const;

// No `style` (it could cover the app with `position: fixed`), `class` or `id` (they could hook
// into the global styles of app.css).
export const ALLOWED_ATTRIBUTES: readonly string[] = [
  'href',
  'title',
  'alt',
  'colspan',
  'rowspan',
  'lang',
  'dir',
  'open',
  'start',
];

// Only the media renderer reads these, to know the URL to offer; it never loads it.
const MEDIA_ATTRIBUTES: readonly string[] = ['src', 'poster'];

/** The library's sanitizer (no `on*`, safe URLs) narrowed to our attribute whitelist. */
export function sanitizeAttributes(
  attributes: Record<string, string>,
  context: SanitizeContext,
  sanitizeUrl: SanitizeUrlFn,
): Record<string, string> {
  const media = (MEDIA_TAGS as readonly string[]).includes(context.tag);
  const safe = defaultSanitizeAttributes(attributes, context, sanitizeUrl);
  return Object.fromEntries(
    Object.entries(safe).filter(([key]) => {
      const name = key.toLowerCase();
      return ALLOWED_ATTRIBUTES.includes(name) || (media && MEDIA_ATTRIBUTES.includes(name));
    }),
  );
}

export type LinkAction = { kind: 'open'; url: string } | { kind: 'none' } | { kind: 'text' };

const WEB_URL = /^https?:\/\//i;
const FRAGMENT_PREFIX = '#';

/**
 * Every link leaves the app through the system opener: anything else would navigate the webview.
 * In-page anchors point into the original page, so they do nothing.
 */
export function linkAction(href: string): LinkAction {
  if (WEB_URL.test(href)) return { kind: 'open', url: href };
  if (href.startsWith(FRAGMENT_PREFIX)) return { kind: 'none' };
  return { kind: 'text' };
}

/** What a media line shows: the alt text, else the URL's domain, else `fallback`. */
export function mediaLabel(alt: string | undefined, url: string | undefined, fallback: string) {
  const text = alt?.trim();
  if (text) return text;
  return url && WEB_URL.test(url) && URL.canParse(url) ? new URL(url).hostname : fallback;
}

type Node = Token & { href?: string; tag?: string; attributes?: Record<string, string> };

const isLink = (t: Node) => t.type === 'link' || (t.type === 'html' && t.tag === 'a');
const isMedia = (t: Node) =>
  t.type === 'image' ||
  (t.type === 'html' && (MEDIA_TAGS as readonly string[]).includes(t.tag ?? ''));

// Child token lists: inline and block children, list items, table header and row cells.
function children(t: object): Node[] {
  const { tokens, items, header, rows } = t as Record<string, unknown>;
  return [tokens, items, header, (rows as unknown[][] | undefined)?.flat()]
    .filter(Array.isArray)
    .flat() as Node[];
}

function retarget(t: Node, href: string): void {
  if (isMedia(t)) {
    if (t.type === 'image') t.href = href;
    else t.attributes = { ...t.attributes, src: href };
  }
  for (const child of children(t)) retarget(child, href);
}

function hasMedia(t: Node): boolean {
  return children(t).some((child) => isMedia(child) || hasMedia(child));
}

/**
 * A link around media (a linked image, a badge) would nest the media line's button in the link
 * and open both URLs. It becomes a plain span, and its media opens the link's target instead.
 */
export function unwrapLinkedMedia(token: Token): void {
  const t = token as Node;
  if (isLink(t) && hasMedia(t)) {
    const href = t.type === 'link' ? t.href : t.attributes?.href;
    const action = linkAction(href ?? '');
    if (action.kind === 'open') retarget(t, action.url);
    Object.assign(t, { type: 'html', tag: 'span', attributes: {} });
  }
  for (const child of children(t)) unwrapLinkedMedia(child);
}

// Module constants: the component re-parses when `options` or `extensions` change identity.
export const READER_OPTIONS = { headerIds: false };

// No `markedAlert`: it only knows GitHub's five types on their own line, not Defuddle's callouts.
// The library calls `walkTokens` only on top-level tokens, so `unwrapLinkedMedia` descends itself.
export const READER_EXTENSIONS: MarkedExtension[] = [
  markedFootnote(),
  { extensions: [callout] },
  { walkTokens: unwrapLinkedMedia },
];
