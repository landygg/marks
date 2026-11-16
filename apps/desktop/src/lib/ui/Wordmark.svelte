<!-- `wordmark` (DESIGN.md, Components): the brand, in the sidebar's top strip; it also drags the window. -->
<script lang="ts">
  import type { Snippet } from 'svelte';

  let { children }: { children: Snippet } = $props();
</script>

<div data-tauri-drag-region>{@render children()}</div>

<style>
  div {
    display: flex;
    flex: none;
    align-items: center;
    padding: 0 var(--space-sm);
    font: var(--type-headline-md);
  }
</style>
