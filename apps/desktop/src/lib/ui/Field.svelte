<!-- `field-label` (DESIGN.md, Components): a control of a dialog form with its label above. -->
<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    label: string;
    /** Id of the control the label names. */
    for: string;
    children: Snippet;
  }

  let { label, for: htmlFor, children }: Props = $props();
</script>

<div>
  <label for={htmlFor}>{label}</label>
  {@render children()}
</div>

<style>
  div {
    display: grid;
    gap: var(--space-xs);
  }

  label {
    color: var(--muted);
    font: var(--type-help);
  }
</style>
