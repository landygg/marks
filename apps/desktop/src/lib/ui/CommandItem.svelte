<!-- `command-item` and `command-item-selected` (DESIGN.md, Components): a row of the palette. -->
<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLButtonAttributes } from 'svelte/elements';
  import Icon from './Icon.svelte';
  import Keys from './Keys.svelte';
  import type { IconName } from './icons';

  interface Props extends HTMLButtonAttributes {
    selected?: boolean;
    icon?: IconName;
    /** Something before the label that is not an icon (a status dot). */
    lead?: Snippet;
    /** Second line under the label; with it the row is the two-line search hit. */
    detail?: Snippet;
    /** Data at the right edge (a domain). */
    meta?: string;
    shortcut?: string;
    /** The label is data (a tag), so it goes in mono. */
    mono?: boolean;
    children: Snippet;
  }

  let {
    selected = false,
    icon,
    lead,
    detail,
    meta,
    shortcut,
    mono = false,
    children,
    ...rest
  }: Props = $props();
</script>

<button
  type="button"
  role="option"
  tabindex="-1"
  aria-selected={selected}
  class={{ hit: detail }}
  {...rest}
>
  {#if icon}<span class="icon"><Icon name={icon} /></span>{/if}
  {#if lead}<span class="lead">{@render lead()}</span>{/if}
  <span class={['label', { mono }]}>
    {#if detail}
      <span class="title">{@render children()}</span>
      <span class="detail">{@render detail()}</span>
    {:else}
      {@render children()}
    {/if}
  </span>
  {#if meta}<span class="meta">{meta}</span>{/if}
  {#if shortcut}<Keys {shortcut} />{/if}
</button>

<style>
  button {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    width: 100%;
    height: var(--command-item);
    padding: 0 var(--space-md);
    border-left: var(--indicator) solid transparent;
    border-radius: 0;
    /* Inside the row: the list clips a ring drawn around it. */
    outline-offset: calc(-1 * var(--indicator));
    color: var(--fg);
    font: var(--type-body-md);
    text-align: left;
  }

  /* No box of its own: the icon inherits the colour and keeps its place in the row. */
  .icon {
    display: contents;
    color: var(--muted);
  }

  /* No hover look of its own: the palette selects the row under the pointer, so the row
     that looks active is always the one Enter runs. */
  button[aria-selected='true'] {
    border-left-color: var(--indicator-color);
    background: var(--border);
  }

  .hit {
    align-items: flex-start;
    height: var(--command-hit);
    padding-top: var(--space-sm);
  }

  /* As tall as the title's first line, which centres the dot on it. */
  .lead {
    display: flex;
    flex: none;
    align-items: center;
    height: 1lh;
    font: var(--type-label-lg);
  }

  .label {
    flex: 1;
    min-width: 0;
  }

  .mono,
  .meta {
    font: var(--type-label-md);
  }

  .title,
  .detail {
    display: block;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .title {
    font: var(--type-label-lg);
  }

  .detail {
    font: var(--type-body-sm);
  }

  .detail,
  .meta {
    color: var(--muted);
  }
</style>
