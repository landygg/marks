<!-- `sidebar-group` (DESIGN.md, Components): the collapsible heading of a group of sidebar items. -->
<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLButtonAttributes } from 'svelte/elements';
  import Icon from './Icon.svelte';

  interface Props extends HTMLButtonAttributes {
    label?: string;
    expanded?: boolean;
    /** A dragged item is over the group. */
    drop?: boolean;
    /** The group is being deleted: the heading is highlighted and `confirm` shows under it. */
    deleting?: boolean;
    /** The name is being edited: `field` (a bare text field) replaces the heading. */
    editing?: boolean;
    field?: Snippet;
    /** The delete question, shown while `deleting`. */
    confirm?: Snippet;
    /** What deleting does, under the question. */
    hint?: string;
  }

  let {
    label = '',
    expanded = true,
    drop = false,
    deleting = false,
    editing = false,
    field,
    confirm,
    hint,
    ...rest
  }: Props = $props();
</script>

{#if editing}
  <div class="field">{@render field?.()}</div>
{:else}
  <button type="button" class={{ drop, deleting }} aria-expanded={expanded} {...rest}>
    <Icon name={expanded ? 'chevronDown' : 'chevronRight'} small />
    <span>{label}</span>
  </button>
{/if}
{#if deleting}
  <!-- Shares the heading's `error` bar, so heading and question read as a unit. -->
  <div class="confirm">
    {@render confirm?.()}
    {#if hint}<p>{hint}</p>{/if}
  </div>
{/if}

<style>
  button {
    display: flex;
    align-items: center;
    /* Chevron and gap put the name where a nested item starts. */
    gap: var(--space-sm);
    width: 100%;
    height: var(--row);
    margin-top: var(--space-xs);
    padding: 0 var(--space-xxs);
    border-left: var(--indicator) solid transparent;
    color: var(--muted);
    font: var(--type-label-sm);
    letter-spacing: var(--tracking-label-sm);
    text-align: left;
    text-transform: uppercase;
  }

  span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  button:hover,
  .drop,
  .deleting {
    color: var(--fg);
  }

  /* Drop target and deletion: the `outline` fill and an indicator bar. */
  .drop,
  .deleting {
    border-left-color: var(--indicator-color);
    background: var(--border);
  }

  .deleting {
    border-left-color: var(--danger);
  }

  /* The bare field has no ring of its own: the box shows the focus. */
  .field {
    margin-top: var(--space-xs);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--bg);
  }

  .field:focus-within {
    border-color: var(--focus);
  }

  .confirm {
    padding-left: var(--space-xs);
    border-left: var(--indicator) solid var(--danger);
  }

  p {
    margin: 0;
    padding: var(--space-xxs) 0 var(--space-xs);
    color: var(--muted);
    font: var(--type-help);
  }
</style>
