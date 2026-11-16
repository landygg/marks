<!-- `palette-search` (DESIGN.md, Components): the palette's search row. Not a bare TextField: it is taller and set in `body-lg`. -->
<script lang="ts">
  import type { HTMLInputAttributes } from 'svelte/elements';
  import Icon from './Icon.svelte';

  interface Props extends HTMLInputAttributes {
    /** The element, for a parent that must focus it. */
    ref?: HTMLInputElement;
  }

  let { ref = $bindable(), ...rest }: Props = $props();
</script>

<label>
  <Icon name="search" />
  <!-- Queries, not prose: no system autocorrect, capitals or spelling marks. -->
  <input bind:this={ref} autocorrect="off" autocapitalize="off" spellcheck={false} {...rest} />
</label>

<style>
  label {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: 0 var(--space-md);
    border-bottom: 1px solid var(--border);
    color: var(--muted);
  }

  input {
    flex: 1;
    min-width: 0;
    height: var(--palette-search);
    color: var(--fg);
    font: var(--type-body-lg);
  }

  /* The ring goes on the whole row, where the dialog's edge does not clip it. */
  input:focus-visible {
    outline: none;
  }

  label:focus-within {
    outline: 2px solid var(--focus);
    outline-offset: calc(-1 * var(--indicator));
  }
</style>
