<!-- `empty-state`: what a zone says when it has nothing to show. -->
<script lang="ts">
  import type { Snippet } from 'svelte';

  let { children }: { children: Snippet } = $props();
</script>

<p>{@render children()}</p>

<style>
  p {
    display: grid;
    flex: 1;
    place-items: center;
    margin: 0;
    padding: var(--space-lg);
    color: var(--muted);
    font: var(--type-body-md);
    text-align: center;
  }
</style>
