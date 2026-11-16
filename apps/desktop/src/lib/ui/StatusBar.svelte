<!-- `status-bar` (DESIGN.md, Components): a panel under the sidebar, the list and the detail. -->
<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    /** Accessible name of the bar. */
    label: string;
    /** The first column fits its content instead of the sidebar's width. */
    sidebarHidden?: boolean;
    /** The first column follows the sidebar's fold with a transition; otherwise it snaps. */
    animate?: boolean;
    /** Under the sidebar. */
    sidebar?: Snippet;
    /** The middle, which clips instead of pushing `end` away. */
    children?: Snippet;
    /** The right edge. */
    end?: Snippet;
  }

  let { label, sidebarHidden = false, animate = false, sidebar, children, end }: Props = $props();
</script>

<footer aria-label={label} class={{ sidebarHidden, animate }}>
  <div class="cell">{@render sidebar?.()}</div>
  <div class="cell middle">{@render children?.()}</div>
  <div class="cell end">{@render end?.()}</div>
</footer>

<style>
  footer {
    display: grid;
    /* Never under its content; the same form open and hidden, so the change can be interpolated.
       --sidebar and --fold are set inline on App.svelte's .window: the live sidebar width and the
       fold duration. */
    grid-template-columns: minmax(max-content, var(--sidebar)) minmax(0, 1fr) auto;
    height: var(--control);
    border-radius: var(--radius-md);
    background: var(--surface);
    color: var(--muted);
    font: var(--type-label-md);
  }

  .sidebarHidden {
    grid-template-columns: minmax(max-content, 0) minmax(0, 1fr) auto;
  }

  .animate {
    transition: grid-template-columns var(--fold) ease-out;
  }

  @media (prefers-reduced-motion: reduce) {
    .animate {
      transition: none;
    }
  }

  .cell {
    display: flex;
    align-items: center;
    min-width: 0;
  }

  .middle {
    gap: var(--space-xxs);
    padding-left: var(--space-sm);
    overflow: hidden;
  }

  .end {
    padding-right: var(--space-xs);
  }
</style>
