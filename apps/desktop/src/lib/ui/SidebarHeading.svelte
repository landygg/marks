<!-- `sidebar-heading` (DESIGN.md, Components): names what the sidebar's panel shows. -->
<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    /** The name; also its tooltip, since a long one is cut with an ellipsis. */
    title: string;
    /** Controls at the right edge, on the name's line. */
    end?: Snippet;
  }

  let { title, end }: Props = $props();
</script>

<div data-tauri-drag-region>
  <h2 {title}>{title}</h2>
  {@render end?.()}
</div>

<style>
  div {
    display: flex;
    flex: none;
    align-items: center;
    gap: var(--space-xs);
    padding: var(--space-sm) var(--space-xs) var(--space-sm) var(--space-lg);
  }

  /* The marker bar: yellow is a fill or an indicator, never a glyph. */
  div::before {
    flex: none;
    width: var(--indicator);
    height: var(--icon);
    background: var(--accent);
    content: '';
  }

  h2 {
    flex: 1;
    min-width: 0;
    margin: 0;
    overflow: hidden;
    font: var(--type-headline-md);
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
