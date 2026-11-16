// Plurals, numbers and dates in the active locale. Kept apart from index.ts so the catalogs can
// import them without a cycle.
export type Locale = 'es' | 'en';

const DATE_OPTIONS: Intl.DateTimeFormatOptions = {
  day: 'numeric',
  month: 'short',
  year: 'numeric',
};

let active: Locale = 'es';
let plurals = new Intl.PluralRules(active);
let numbers = new Intl.NumberFormat(active);
let dates = new Intl.DateTimeFormat(active, DATE_OPTIONS);

/** Called by `setLocale`; the formatters are built once per locale, not per call. */
export function setIntlLocale(l: Locale): void {
  active = l;
  plurals = new Intl.PluralRules(l);
  numbers = new Intl.NumberFormat(l);
  dates = new Intl.DateTimeFormat(l, DATE_OPTIONS);
}

/** The locale for `Intl` (`en` while the pseudo-locale is on). */
export const locale = (): Locale => active;

/** `one` for the singular category; any other category (`many` in Spanish too) takes `other`. */
export const plural = (n: number, forms: { one: string; other: string }): string =>
  plurals.select(n) === 'one' ? forms.one : forms.other;

export const num = (n: number, opts?: Intl.NumberFormatOptions): string =>
  opts ? new Intl.NumberFormat(active, opts).format(n) : numbers.format(n);

/**
 * Local date: `"2026-11-09T12:00:00Z"` → `"9 nov 2026"` in Spanish, `"Nov 9, 2026"` in English.
 * Spanish drops the literals and the month's dot, since ICU gives `"9 nov 2026"` or
 * `"9 de nov. de 2026"` depending on its version.
 */
export function date(iso: string): string {
  if (active !== 'es') return dates.format(new Date(iso));
  return dates
    .formatToParts(new Date(iso))
    .filter((part) => part.type !== 'literal')
    .map((part) => (part.type === 'month' ? part.value.replace(/\.$/, '') : part.value))
    .join(' ');
}
