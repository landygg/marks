<!-- `panel` (DESIGN.md, Components): the `surface` body of a zone, rounded, with the gaps around it. -->
<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    id?: string;
    children: Snippet;
  }

  let { id, children }: Props = $props();
</script>

<div {id}>{@render children()}</div>

<style>
  div {
    display: flex;
    flex: 1;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
    border-radius: var(--radius-md);
    background: var(--surface);
  }
</style>
