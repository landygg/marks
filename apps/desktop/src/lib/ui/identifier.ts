import type { HTMLInputAttributes } from 'svelte/elements';

/** For tags, URLs, group names and initials: WebKit on macOS leaves them as typed. */
export const IDENTIFIER_ATTRIBUTES: HTMLInputAttributes = {
  autocorrect: 'off',
  autocapitalize: 'none',
  spellcheck: false,
};
