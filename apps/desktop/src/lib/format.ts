import { locale } from './i18n';

const WWW_PREFIX = 'www.';
// Same separator as `TAG_SEPARATOR` in crates/core/src/store.rs.
const TAG_SEPARATOR = ',';
const TIME_OPTIONS: Intl.DateTimeFormatOptions = {
  hour: '2-digit',
  minute: '2-digit',
  hour12: false,
};

/** Local date in the active locale (`"9 nov 2026"`, `"Nov 9, 2026"`); see `date` in `i18n/format.ts`. */
export { date as formatDate } from './i18n';

/** Host without `www.`; the input itself when it is not a valid URL. */
export function domainOf(url: string): string {
  try {
    const host = new URL(url).hostname;
    return host.startsWith(WWW_PREFIX) ? host.slice(WWW_PREFIX.length) : host;
  } catch {
    return url;
  }
}

/** Local time of a timestamp in ms, on the 24-hour clock: `"09:05"`. */
export function formatTime(ms: number): string {
  return new Intl.DateTimeFormat(locale(), TIME_OPTIONS).format(ms);
}

/** Mirrors `normalize_tags` in core: one tag per comma, whitespace collapsed, lowercase, no empties or duplicates. */
export function parseTags(input: string): string[] {
  const tags = input
    .split(TAG_SEPARATOR)
    .map((tag) => tag.trim().split(/\s+/).join(' ').toLowerCase())
    .filter((tag) => tag !== '');
  return [...new Set(tags)];
}

const ELLIPSIS = '…';

/** At most `max` characters of `text` (trailing whitespace dropped) plus `…` when it was cut. */
export function truncate(text: string, max: number): string {
  return text.length <= max ? text : text.slice(0, max).trimEnd() + ELLIPSIS;
}
