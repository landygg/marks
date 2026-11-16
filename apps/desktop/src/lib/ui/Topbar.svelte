<!-- `topbar` (DESIGN.md, Components): the top strip of a zone, which also drags the window. -->
<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    /** Controls at the left edge. */
    start?: Snippet;
    /** `topbar-title`; wrap the part to stress in `<b>`. */
    title?: Snippet;
    /** Controls at the right edge. */
    end?: Snippet;
    /** Left room for the part of the macOS traffic lights that overhangs the profile rail. */
    lights?: boolean;
  }

  let { start, title, end, lights = false }: Props = $props();
</script>

<header class={{ lights }} data-tauri-drag-region>
  {@render start?.()}
  <p>{@render title?.()}</p>
  {@render end?.()}
</header>

<style>
  header {
    display: flex;
    flex: none;
    align-items: center;
    gap: var(--space-xxs);
    height: var(--topbar);
    padding: 0 var(--space-xs);
    background: var(--bg);
  }

  .lights {
    padding-left: calc(var(--window-controls) - var(--rail));
  }

  p {
    flex: 1;
    min-width: 0;
    margin: 0;
    padding: 0 var(--space-sm);
    overflow: hidden;
    color: var(--muted);
    font: var(--type-label-md);
    text-overflow: ellipsis;
    white-space: nowrap;
    pointer-events: none;
  }

  p :global(b) {
    color: var(--fg);
    font-weight: inherit;
  }
</style>
