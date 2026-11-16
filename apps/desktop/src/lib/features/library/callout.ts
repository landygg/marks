// Marked extension for the Obsidian-style callouts Defuddle emits: `> [!type] Title`, with an
// optional fold mark (`-`/`+`) and a Markdown body. Any type is accepted, unlike `markedAlert`.
import type { MarkedExtension } from '@humanspeak/svelte-markdown';

// `TokenizerAndRendererExtension`, reached through the type the library re-exports: `marked` is
// not a direct dependency.
type Extension = NonNullable<MarkedExtension['extensions']>[number];

const CALLOUT =
  /^ {0,3}> ?\[!([\w-]+)\][-+]?(?:[ \t]+([^\n]*))?(?:\n|$)((?: {0,3}>[^\n]*(?:\n|$))*)/;
const QUOTE_MARK = /^ {0,3}> ?/;
const CALLOUT_START = /^ {0,3}> ?\[!/m;

export const callout = {
  name: 'callout',
  level: 'block',
  start: (src: string) => src.match(CALLOUT_START)?.index,
  tokenizer(src: string) {
    const match = CALLOUT.exec(src);
    if (!match) return undefined;
    const body = match[3]
      .split('\n')
      .map((line) => line.replace(QUOTE_MARK, ''))
      .join('\n')
      .trimEnd();
    return {
      type: 'callout',
      raw: match[0],
      kind: match[1].toLowerCase(),
      title: (match[2] ?? '').trim(),
      tokens: this.lexer.blockTokens(body, []),
    };
  },
} satisfies Extension;
