<!-- `input` and `input-inset` (DESIGN.md, Components). -->
<script lang="ts">
  import type { HTMLInputAttributes } from 'svelte/elements';

  interface Props extends HTMLInputAttributes {
    value?: string | null;
    /** The element, for a parent that must focus or measure it. */
    ref?: HTMLInputElement;
    inset?: boolean;
    mono?: boolean;
    /** Fixed `field-width` instead of filling its container. */
    field?: boolean;
    /** No border or fill: the container draws them (palette search, tag field, inline names). */
    bare?: boolean;
  }

  let {
    value = $bindable(),
    ref = $bindable(),
    inset = false,
    mono = false,
    field = false,
    bare = false,
    ...rest
  }: Props = $props();
</script>

<input {...rest} class={['input', { inset, mono, field, bare }]} bind:value bind:this={ref} />

<style>
  /* Wins over the shared `.input` look, focus included. */
  input.bare,
  input.bare:focus-visible {
    border-color: transparent;
    background: none;
    outline: none;
  }
</style>
