<!-- `detail-title` (DESIGN.md, Components): reads as a heading, edits as a field. -->
<script lang="ts">
  import type { HTMLInputAttributes } from 'svelte/elements';

  interface Props extends HTMLInputAttributes {
    value?: string;
  }

  let { value = $bindable(), ...rest }: Props = $props();
</script>

<input bind:value {...rest} />

<style>
  input {
    display: block;
    width: 100%;
    min-width: 0;
    /* The border is part of the inset. */
    padding: var(--space-xxs) calc(var(--space-sm) - 1px);
    border: 1px solid transparent;
    color: var(--fg);
    font: var(--type-headline-lg);
    letter-spacing: var(--tracking-headline-lg);
  }

  input:hover:not(:disabled) {
    border-color: var(--border);
  }

  input:focus-visible {
    border-color: var(--focus);
  }
</style>
