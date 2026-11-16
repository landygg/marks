// The active catalog (`t`) and the locale. `t` starts as Spanish so tests that never call
// `setLocale` read the same texts as before; `main.ts` sets the real locale before mounting.
import { en } from './en';
import { es, type Messages } from './es';
import { setIntlLocale, type Locale } from './format';
import { pseudo } from './pseudo';

export { date, locale, num, plural, type Locale } from './format';
export type { Messages } from './es';

export let t: Messages = es;

export function setLocale(l: Locale | 'pseudo'): void {
  t = l === 'es' ? es : l === 'en' ? en : pseudo(en);
  const intl = l === 'pseudo' ? 'en' : l;
  setIntlLocale(intl);
  // Some tests run without a DOM.
  if (typeof document !== 'undefined') document.documentElement.lang = intl;
}
