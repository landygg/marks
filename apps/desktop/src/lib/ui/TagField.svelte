<!-- `tag-field` (DESIGN.md, Components): the field that adds tags, drawn as an empty (dashed) `tag`. -->
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
    width: var(--tag-field);
    height: var(--tag);
    padding: 0 calc(var(--space-sm) - 1px);
    border: 1px dashed var(--border);
    color: var(--fg);
    font: var(--type-label-md);
  }

  input::placeholder {
    color: var(--muted);
  }

  input:hover:not(:disabled) {
    border-color: var(--muted);
  }
</style>
