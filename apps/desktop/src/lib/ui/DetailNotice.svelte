<!-- `detail-blocked` (DESIGN.md, Components): a notice in the detail, message left, action right. -->
<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    message: string;
    /** The action, usually a secondary Button. */
    children?: Snippet;
  }

  let { message, children }: Props = $props();
</script>

<div>
  <span>{message}</span>
  {@render children?.()}
</div>

<style>
  div {
    display: flex;
    align-items: center;
    gap: var(--space-lg);
    padding: var(--space-sm);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--bg);
    color: var(--fg);
    font: var(--type-body-md);
  }

  span {
    flex: 1;
    min-width: 0;
  }
</style>
