// The split behind `detail-url` (DESIGN.md, Components): a long text cut in the middle keeps its end.

/** How many characters of the end always stay visible. */
export const TAIL_CHARS = 32;

export interface Split {
  /** Shrinks with an ellipsis when the line runs out. */
  head: string;
  /** Always shown whole; empty when the text is short. */
  tail: string;
}

const graphemes = new Intl.Segmenter(undefined, { granularity: 'grapheme' });

/**
 * Splits `text` so its last `tail` characters stay visible. A text up to twice that long is left
 * whole in `head`. Counts graphemes, so an emoji or an accent is never cut in two.
 */
export function splitTail(text: string, tail = TAIL_CHARS): Split {
  const chars = Array.from(graphemes.segment(text), (s) => s.segment);
  if (chars.length <= tail * 2) return { head: text, tail: '' };
  return { head: chars.slice(0, -tail).join(''), tail: chars.slice(-tail).join('') };
}
