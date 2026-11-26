<!-- `tag-field` (DESIGN.md, Components): the field that adds tags, drawn as an empty (dashed) `tag`. -->
<script lang="ts">
  import type { HTMLInputAttributes } from 'svelte/elements';
  import { IDENTIFIER_ATTRIBUTES } from './identifier';

  interface Props extends HTMLInputAttributes {
    value?: string;
    /** No autocorrect, automatic capitals or spelling marks (tags are identifiers). */
    identifier?: boolean;
  }

  let { value = $bindable(), identifier = false, ...rest }: Props = $props();
</script>

<input bind:value {...identifier ? IDENTIFIER_ATTRIBUTES : {}} {...rest} />

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
