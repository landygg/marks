<script module lang="ts">
  import { Html, htmlRendererKeys, type HtmlRenderers } from '@humanspeak/svelte-markdown';
  import ContentText from './ContentText.svelte';
  import { ALLOWED_TAGS } from './reader';

  // A tag outside the whitelist renders only its children: never the element, and never its source
  // as escaped text (what `allowHtmlOnly` does). Tags the library has no component for already do.
  const RENDERERS = {
    html: Object.fromEntries(
      htmlRendererKeys.map((tag) => [
        tag,
        ALLOWED_TAGS.includes(tag) ? Html[tag as keyof HtmlRenderers] : ContentText,
      ]),
    ) as HtmlRenderers,
  };
</script>

<script lang="ts">
  import SvelteMarkdown, {
    type CodeSnippetProps,
    type CodespanSnippetProps,
    type HtmlSnippetProps,
    type ImageSnippetProps,
    type LinkSnippetProps,
    type ListItemSnippetProps,
    type TableSnippetProps,
  } from '@humanspeak/svelte-markdown';
  import type { Snippet } from 'svelte';
  import ContentLink from '../../ui/ContentLink.svelte';
  import ContentMedia from '../../ui/ContentMedia.svelte';
  import { t } from '../../strings';
  import { openLink } from './libraryState.svelte';
  import {
    READER_EXTENSIONS,
    READER_OPTIONS,
    linkAction,
    mediaLabel,
    sanitizeAttributes,
  } from './reader';

  let { content }: { content: string | null } = $props();

  type MediaKind = 'image' | 'video' | 'audio' | 'frame';
  type Attribute = HtmlSnippetProps['attributes'];
  type Footnote = { id: string; text: string };

  // Task glyphs: symbols, not words, so they stay out of strings.ts; their names are there.
  const TASK_DONE = '☑';
  const TASK_OPEN = '☐';

  const text = (value: unknown) => (typeof value === 'string' ? value : undefined);

  // Everything a reader needs, shared by the page and each footnote body.
  const reader = {
    options: READER_OPTIONS,
    extensions: READER_EXTENSIONS,
    renderers: RENDERERS,
    sanitizeAttributes,
    link,
    html_a,
    image,
    html_img,
    html_iframe,
    html_video,
    html_audio,
    html_embed,
    code,
    codespan,
    listitem,
    table,
    html_table,
    footnoteRef,
    footnoteSection,
    callout,
    html_caption,
    html_figure,
    html_figcaption,
    html_ins,
    html_q,
  };
</script>

{#snippet anchor(href: string | undefined, children: Snippet | undefined)}
  {@const action = linkAction(href ?? '')}
  {#if action.kind === 'open'}
    <ContentLink href={action.url} onopen={openLink}>{@render children?.()}</ContentLink>
  {:else}
    {@render children?.()}
  {/if}
{/snippet}

{#snippet link({ href, children }: LinkSnippetProps)}
  {@render anchor(href, children)}
{/snippet}

{#snippet html_a({ attributes, children }: HtmlSnippetProps)}
  {@render anchor(text(attributes?.href), children)}
{/snippet}

{#snippet media(kind: MediaKind, src: string | undefined, alt: string | undefined)}
  {@const action = linkAction(src ?? '')}
  {@const label = mediaLabel(alt, src, t.mediaKinds[kind])}
  <ContentMedia
    {kind}
    url={action.kind === 'open' ? action.url : null}
    {label}
    openLabel={t.mediaOpen}
    openName={t.mediaOpenNamed(label)}
    onopen={openLink}
  />
{/snippet}

{#snippet image({ href, text: alt }: ImageSnippetProps)}
  {@render media('image', href, alt)}
{/snippet}

{#snippet mediaTag(kind: MediaKind, attributes: Attribute, alt: unknown)}
  {@render media(kind, text(attributes?.src), text(alt))}
{/snippet}

{#snippet html_img({ attributes }: HtmlSnippetProps)}
  {@render mediaTag('image', attributes, attributes?.alt)}
{/snippet}

{#snippet html_iframe({ attributes }: HtmlSnippetProps)}
  {@render mediaTag('frame', attributes, attributes?.title)}
{/snippet}

{#snippet html_video({ attributes }: HtmlSnippetProps)}
  {@render mediaTag('video', attributes, attributes?.title)}
{/snippet}

{#snippet html_audio({ attributes }: HtmlSnippetProps)}
  {@render mediaTag('audio', attributes, attributes?.title)}
{/snippet}

{#snippet html_embed({ attributes }: HtmlSnippetProps)}
  {@render mediaTag('frame', attributes, attributes?.title)}
{/snippet}

<!-- The language as text: the default renderer puts it in a `class`. A focusable region, so the
     keyboard can scroll a wide block (our attributes, not the content's; a scroll region needs the
     focus, which Svelte's tabindex rule does not know). -->
{#snippet code({ lang, text: source }: CodeSnippetProps)}
  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
  <pre tabindex="0" role="region" aria-label={t.codeBlock}>{#if lang}<span class="code-lang"
        >{lang}</span
      >{/if}<code>{source}</code></pre>
{/snippet}

<!-- `text`, not `raw`: the default loses the backticks inside (library issue #417). -->
{#snippet codespan({ text: source }: CodespanSnippetProps)}
  <code>{source}</code>
{/snippet}

<!-- The default item drops the task box; a glyph, since only lib/ui may draw fields. -->
{#snippet listitem({ task, checked, children }: ListItemSnippetProps)}
  <li>
    {#if task}<span class="task" role="img" aria-label={checked ? t.taskDone : t.taskOpen}
        >{checked ? TASK_DONE : TASK_OPEN}</span
      >{/if}{@render children?.()}
  </li>
{/snippet}

<!-- `content-table`: its own horizontal scroll, for Markdown and HTML tables alike, reachable by
     keyboard. -->
{#snippet table({ children }: TableSnippetProps)}
  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
  <div class="table" tabindex="0" role="region" aria-label={t.tableRegion}>
    <table>{@render children?.()}</table>
  </div>
{/snippet}

{#snippet html_table({ attributes, children }: HtmlSnippetProps)}
  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
  <div class="table" tabindex="0" role="region" aria-label={t.tableRegion}>
    <table {...attributes}>{@render children?.()}</table>
  </div>
{/snippet}

<!-- The label as written, with no link: the library's reference adds an `id` and a `class`. -->
{#snippet footnoteRef({ id }: { id: string })}
  <sup>{id}</sup>
{/snippet}

<!-- `content-footnotes`: each body is Markdown (the library's section prints it as plain text),
     under its label instead of a number. -->
{#snippet footnoteSection({ footnotes }: { footnotes: Footnote[] })}
  <section class="footnotes">
    {#each footnotes as note (note.id)}
      <p><sup>{note.id}</sup> <SvelteMarkdown source={note.text} isInline {...reader} /></p>
    {/each}
  </section>
{/snippet}

<!-- `content-callout`: a quote with its type as a `section-label`. -->
{#snippet callout({ kind, title, children }: { kind: string; title: string; children?: Snippet })}
  <blockquote class="callout">
    <span class="section-label">{kind}</span>
    {#if title}<p class="callout-title"><strong>{title}</strong></p>{/if}
    {@render children?.()}
  </blockquote>
{/snippet}

<!-- Whitelisted tags the library has no component for. -->
{#snippet html_caption({ attributes, children }: HtmlSnippetProps)}
  <caption {...attributes}>{@render children?.()}</caption>
{/snippet}

{#snippet html_figure({ attributes, children }: HtmlSnippetProps)}
  <figure {...attributes}>{@render children?.()}</figure>
{/snippet}

{#snippet html_figcaption({ attributes, children }: HtmlSnippetProps)}
  <!-- Its parent is whatever the content nests it in. -->
  <!-- svelte-ignore a11y_figcaption_parent -->
  <figcaption {...attributes}>{@render children?.()}</figcaption>
{/snippet}

{#snippet html_ins({ attributes, children }: HtmlSnippetProps)}
  <ins {...attributes}>{@render children?.()}</ins>
{/snippet}

{#snippet html_q({ attributes, children }: HtmlSnippetProps)}
  <q {...attributes}>{@render children?.()}</q>
{/snippet}

<section>
  <h2 class="section-label">{t.contentLabel}</h2>
  {#if content}
    <!-- Keyed so that a failure belongs to one content, not to the next bookmark shown. -->
    {#key content}
      <!-- `content-raw`: the library caps nesting itself (`maxNestingDepth`); this boundary is the
           backstop for any other render error, so the source stays readable. -->
      <svelte:boundary>
        <article class="content"><SvelteMarkdown source={content} {...reader} /></article>
        {#snippet failed()}
          <p class="empty">{t.contentUnrenderable}</p>
          <article class="content raw">{content}</article>
        {/snippet}
      </svelte:boundary>
    {/key}
  {:else}
    <p class="empty">{t.noContent}</p>
  {/if}
</section>

<style>
  .content {
    max-width: var(--measure);
    font: var(--type-body-lg);
    overflow-wrap: anywhere;
  }

  /* Elements drawn by the library's own components, not by this file. */
  .content :global(code) {
    font: var(--type-label-md);
  }

  .content :global(pre) {
    overflow-x: auto;
    padding: var(--space-md);
    border-radius: var(--radius-md);
    background: var(--bg);
  }

  .content :global(:is(th, td)) {
    padding: var(--space-sm);
    border: 1px solid var(--border);
  }

  .raw {
    white-space: pre-wrap;
  }

  .code-lang {
    display: block;
    margin-bottom: var(--space-xs);
    color: var(--muted);
    font: var(--type-label-sm);
  }

  .task {
    margin-inline-end: var(--space-xs);
  }

  .table {
    margin: var(--space-md) 0;
    overflow-x: auto;
  }

  table {
    border-collapse: collapse;
    color: var(--fg);
    font: var(--type-body-md);
  }

  .footnotes {
    margin-top: var(--space-xl);
    padding-top: var(--space-sm);
    border-top: 1px solid var(--border);
    color: var(--muted);
    font: var(--type-body-sm);
  }

  /* Defuddle separates definitions with blank lines, so each one arrives as its own section; they
     are the only sections in the content, since `section` is not whitelisted. */
  .footnotes:not(:first-of-type) {
    margin-top: 0;
    padding-top: 0;
    border-top: 0;
  }

  .footnotes p {
    margin: 0 0 var(--space-xs);
  }

  .callout {
    margin: var(--space-md) 0;
    padding: var(--space-md);
    border-inline-start: 1px solid var(--border);
    color: var(--fg);
    font: var(--type-body-lg);
  }

  .callout .section-label {
    margin-top: 0;
  }

  .callout-title {
    margin: 0 0 var(--space-xs);
  }

  .empty {
    margin: 0;
    color: var(--muted);
  }
</style>
