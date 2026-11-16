// The only module that reads the extension namespace (`browser` on Safari/Firefox, `chrome` on Chromium).
export const ext = globalThis.browser ?? globalThis.chrome;

export const t = (key, substitution) => ext.i18n.getMessage(key, substitution);

/** Fills every `data-i18n` element with its localized message and sets the page language. */
export function translatePage() {
	// The locale's own `lang` message: it names the language actually shown, not the browser's.
	document.documentElement.lang = t("lang") || document.documentElement.lang;
	for (const el of document.querySelectorAll("[data-i18n]"))
		el.textContent = t(el.dataset.i18n);
}
