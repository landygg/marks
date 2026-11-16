<!-- `list-row` (DESIGN.md, Components): a bookmark in the list. -->
<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLButtonAttributes } from 'svelte/elements';

  interface Props extends HTMLButtonAttributes {
    title: string;
    /** `list-row-summary`; the line is left out when empty. */
    summary?: string | null;
    /** `list-row-meta`: data, in mono. */
    meta: string;
    selected?: boolean;
    /** Before the text, level with the title (the status dot). */
    lead?: Snippet;
  }

  let { title, summary, meta, selected = false, lead, ...rest }: Props = $props();
</script>

<button type="button" aria-current={selected ? 'true' : undefined} {...rest}>
  {#if lead}<span class="lead">{@render lead()}</span>{/if}
  <span class="body">
    <span class="title">{title}</span>
    {#if summary}<span class="summary">{summary}</span>{/if}
    <span class="meta">{meta}</span>
  </span>
</button>

<style>
  button {
    display: flex;
    gap: var(--space-sm);
    width: 100%;
    /* The indicator is part of the left inset, so the dot starts where the topbar title does. */
    padding: var(--space-sm) var(--space-md);
    padding-left: calc(var(--space-md) - var(--indicator));
    border-bottom: 1px solid var(--border);
    border-left: var(--indicator) solid transparent;
    border-radius: 0;
    text-align: left;
  }

  button:hover,
  button[aria-current='true'] {
    background: var(--bg);
  }

  button[aria-current='true'] {
    border-left-color: var(--accent);
  }

  /* Inside the row: the list clips anything drawn outside it. */
  button:focus-visible {
    outline-offset: calc(-1 * var(--indicator));
  }

  /* As tall as the title's first line, which centres the dot on it. */
  .lead {
    display: flex;
    flex: none;
    align-items: center;
    height: 1lh;
    font: var(--type-label-lg);
  }

  .body {
    display: flex;
    flex-direction: column;
    gap: var(--space-xxs);
    min-width: 0;
  }

  .title,
  .summary,
  .meta {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .title {
    font: var(--type-label-lg);
  }

  .summary {
    color: var(--muted);
    font: var(--type-body-sm);
  }

  .meta {
    color: var(--muted);
    font: var(--type-label-md);
  }
</style>
