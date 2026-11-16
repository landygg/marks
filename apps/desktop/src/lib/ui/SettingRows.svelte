<!-- `setting-rows` (DESIGN.md, Components): the bordered block that holds the rows of a Settings section. -->
<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    /** Given, the block is a form. */
    onsubmit?: (event: SubmitEvent) => void;
    children: Snippet;
  }

  let { onsubmit, children }: Props = $props();
</script>

{#if onsubmit}
  <form {onsubmit}>{@render children()}</form>
{:else}
  <div>{@render children()}</div>
{/if}

<style>
  form,
  div {
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--bg);
  }
</style>
