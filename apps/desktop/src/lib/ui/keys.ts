// Splits a shortcut as strings.ts writes it ("⌥⌘2") into its keys, for Keys.svelte.

const MODIFIERS = ['⌃', '⌥', '⇧', '⌘'];

/** "⌥⌘2" → ["⌥", "⌘", "2"], "⌘," → ["⌘", ","], "esc" → ["esc"]: each leading modifier is a key, the rest is the last one. */
export function shortcutKeys(shortcut: string): string[] {
  const chars = [...shortcut];
  const keyStart = chars.findIndex((char) => !MODIFIERS.includes(char));
  if (keyStart < 0) return chars;
  return [...chars.slice(0, keyStart), chars.slice(keyStart).join('')];
}
