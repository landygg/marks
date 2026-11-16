// Pseudo-locale for a manual pass (`MARKS_LOCALE=pseudo`): accented text about 35 % longer
// between ⟦ ⟧, so untranslated text and text that does not fit stand out.
import type { Messages } from './es';

// Each letter of PLAIN becomes the one at the same index of ACCENTED.
const PLAIN = 'aeiouncAEIOUNC';
const ACCENTED = 'áéíóúñçÁÉÍÓÚÑÇ'; // i18n-allow: pseudo-locale
const GROWTH = 0.35;
const PAD = '~';

const text = (s: string): string =>
  `⟦${s.replace(/[aeiouncAEIOUNC]/g, (ch) => ACCENTED[PLAIN.indexOf(ch)])}${PAD.repeat(Math.ceil(s.length * GROWTH))}⟧`;

function walk(value: unknown): unknown {
  if (typeof value === 'string') return text(value);
  if (typeof value === 'function') return (...args: unknown[]) => walk(value(...args));
  if (Array.isArray(value)) return value.map(walk);
  if (typeof value === 'object' && value !== null)
    return Object.fromEntries(Object.entries(value).map(([key, child]) => [key, walk(child)]));
  return value;
}

export const pseudo = (m: Messages): Messages => walk(m) as Messages;
