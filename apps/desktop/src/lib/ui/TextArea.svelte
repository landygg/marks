<!-- `input` for several lines (DESIGN.md, Components). -->
<script lang="ts">
  import type { HTMLTextareaAttributes } from 'svelte/elements';

  interface Props extends HTMLTextareaAttributes {
    value?: string;
    inset?: boolean;
    mono?: boolean;
  }

  let { value = $bindable(), inset = false, mono = false, ...rest }: Props = $props();
</script>

<textarea {...rest} class={['input', { inset, mono }]} bind:value></textarea>

<style>
  textarea {
    height: auto;
    min-height: var(--textarea);
    padding: var(--space-sm);
    resize: vertical;
  }
</style>
