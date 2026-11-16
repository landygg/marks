<!-- `palette-tab` (DESIGN.md, Components): a mode of the palette, or one of its actions, in its footer. -->
<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLButtonAttributes } from 'svelte/elements';
  import Keys from './Keys.svelte';

  interface Props extends HTMLButtonAttributes {
    /** Only for a mode: whether it is the current one. An action (run, close) leaves it out and is no toggle. */
    selected?: boolean;
    shortcut?: string;
    children: Snippet;
  }

  let { selected, shortcut, children, ...rest }: Props = $props();
</script>

<button type="button" aria-pressed={selected} {...rest}>
  {@render children()}
  {#if shortcut}<Keys {shortcut} />{/if}
</button>

<style>
  button {
    display: flex;
    flex: none;
    align-items: center;
    gap: var(--space-xs);
    height: var(--row);
    padding: 0 var(--space-sm);
    color: var(--muted);
    font: var(--type-body-md);
  }

  button:hover:not(:disabled) {
    color: var(--fg);
  }

  button[aria-pressed='true'] {
    background: var(--border);
    color: var(--fg);
  }
</style>
