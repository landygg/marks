// Pure text helpers for the command palette: match highlighting, excerpts and list navigation.

export interface Piece {
  text: string;
  match: boolean;
}

const ELLIPSIS = '…';
const SPECIAL = /[.*+?^${}()|[\]\\]/g;

const words = (query: string): string[] => query.trim().split(/\s+/).filter(Boolean);

/** Case-insensitive pattern for any of the query's words, taken as literal text; `null` for a blank query. */
function pattern(query: string): RegExp | null {
  const literals = words(query).map((word) => word.replace(SPECIAL, '\\$&'));
  return literals.length === 0 ? null : new RegExp(`(${literals.join('|')})`, 'gi');
}

/** Splits `text` at case-insensitive occurrences of the query's words; the query is literal text, never a pattern. */
export function highlight(text: string, query: string): Piece[] {
  const re = pattern(query);
  if (!re) return text === '' ? [] : [{ text, match: false }];
  // `split` with one capture group alternates plain text (even) and matches (odd).
  return text
    .split(re)
    .map((part, i) => ({ text: part, match: i % 2 === 1 }))
    .filter((piece) => piece.text !== '');
}

/** At most `max` characters of `text` around the first occurrence of a query word (the start of `text` if none), with ellipses where cut. */
export function excerpt(text: string, query: string, max: number): string {
  if (text.length <= max) return text;
  const found = pattern(query)?.exec(text);
  const centered = found ? found.index - Math.floor((max - found[0].length) / 2) : 0;
  const start = Math.max(0, Math.min(centered, text.length - max));
  const end = start + max;
  return (start > 0 ? ELLIPSIS : '') + text.slice(start, end) + (end < text.length ? ELLIPSIS : '');
}

/** `index + delta` wrapped inside `[0, length)`; 0 when `length` is 0. */
export function moveIndex(index: number, delta: number, length: number): number {
  return length === 0 ? 0 : (index + delta + length) % length;
}

export interface Point {
  x: number;
  y: number;
}

/**
 * Whether a pointer event is a real move, which is what selects a row of the palette. A list that
 * scrolls under a resting pointer also fires a move, and it must not take the selection back from
 * the arrow keys: the first event after opening (`previous` is `null`) only shows where the
 * pointer rests, and an event at the same place is not a move.
 */
export const pointerMoved = (previous: Point | null, next: Point): boolean =>
  previous !== null && (next.x !== previous.x || next.y !== previous.y);

/**
 * The row selected once the pointer leaves the list: the one chosen before the pointer took over
 * (`chosen`), so reaching the footer's "Run" across other rows runs the row the user chose.
 * It stays at `current` when there is none, or when that row no longer exists.
 */
export const rowAfterPointerLeaves = (
  current: number,
  chosen: number | null,
  count: number,
): number => (chosen !== null && chosen < count ? chosen : current);
