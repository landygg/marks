<!-- `sidebar-item` (DESIGN.md, Components). -->
<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLButtonAttributes } from 'svelte/elements';
  import Icon from './Icon.svelte';
  import type { IconName } from './icons';

  interface Props extends HTMLButtonAttributes {
    selected?: boolean;
    /** The item opens a view (a Settings section): selected, it is the current page. */
    page?: boolean;
    /** Indented under a group heading. */
    nested?: boolean;
    /** Icon before the label (the "back" item). */
    icon?: IconName;
    /** Shows the drag handle: the item can be dragged. */
    grip?: boolean;
    /** The item is being dragged: it stays in place, dimmed. */
    dragging?: boolean;
    children: Snippet;
  }

  let {
    selected = false,
    page = false,
    nested = false,
    icon,
    grip = false,
    dragging = false,
    children,
    ...rest
  }: Props = $props();
</script>

<button
  type="button"
  class={{ nested, grip, dragging }}
  aria-current={selected ? (page ? 'page' : 'true') : undefined}
  {...rest}
>
  {#if icon}<Icon name={icon} small />{/if}
  <span>{@render children()}</span>
  {#if grip}<i><Icon name="grip" small /></i>{/if}
</button>

<style>
  button {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    width: 100%;
    height: var(--row);
    padding: 0 var(--space-sm);
    color: var(--muted);
    font: var(--type-label-md);
    text-align: left;
  }

  .nested {
    padding-left: var(--space-xl);
  }

  span {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  button:hover {
    color: var(--fg);
  }

  button[aria-current] {
    background: var(--accent);
    color: var(--on-accent);
  }

  .grip {
    padding-right: var(--space-xs);
    cursor: grab;
  }

  /* Drag handle: always there, so the item reads as draggable before the pointer reaches it. */
  i {
    display: grid;
    flex: none;
    opacity: 0.45;
  }

  button:hover i,
  button:focus-visible i {
    opacity: 1;
  }

  .dragging {
    opacity: 0.5;
  }
</style>
