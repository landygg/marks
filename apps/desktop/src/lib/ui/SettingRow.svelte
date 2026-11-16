<!-- `setting-row` and `setting-help` (DESIGN.md, Components): name on the left, control on the right. -->
<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    label?: string;
    /** Id of the control the label names; without it, pass `labelId` to name a group. */
    for?: string;
    labelId?: string;
    /** Help, state or result under the name. Point the control's `aria-describedby` at `helpId`. */
    help?: string | null;
    helpId?: string;
    helpRole?: 'alert' | 'status';
    helpError?: boolean;
    /** Under the help (a `command-line`). */
    note?: Snippet;
    /** Before the name (a status dot). */
    lead?: Snippet;
    /** The control(s) at the right. */
    children?: Snippet;
  }

  let {
    label,
    for: htmlFor,
    labelId,
    help,
    helpId,
    helpRole,
    helpError = false,
    note,
    lead,
    children,
  }: Props = $props();
</script>

<div class="row">
  {@render lead?.()}
  <div class="name">
    {#if label && htmlFor}
      <label for={htmlFor}>{label}</label>
    {:else if label}
      <span id={labelId}>{label}</span>
    {/if}
    <!-- Keyed by role: a status changes its text in place and an error arrives as a new alert,
         which is how a screen reader announces each of them. -->
    {#key helpRole}
      {#if help}<small id={helpId} class={{ 'error-text': helpError }} role={helpRole}>{help}</small
        >{/if}
    {/key}
    {@render note?.()}
  </div>
  {@render children?.()}
</div>

<style>
  .row {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    box-sizing: border-box;
    min-height: var(--setting-row);
    padding: var(--space-sm) var(--space-md);
    font: var(--type-body-md);
    border-bottom: 1px solid var(--border);
  }

  .row:last-child {
    border-bottom: 0;
  }

  .name {
    flex: 1;
    min-width: 0;
  }

  small {
    display: block;
    font: var(--type-help);
  }

  small:not(.error-text) {
    color: var(--muted);
  }
</style>
