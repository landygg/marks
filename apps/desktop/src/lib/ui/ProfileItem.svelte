<!-- `profile-item` and `profile-add` (DESIGN.md, Components). -->
<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLButtonAttributes } from 'svelte/elements';
  import Icon from './Icon.svelte';

  interface Props extends HTMLButtonAttributes {
    /** Accessible name and tooltip: the profile's name and shortcut. */
    label: string;
    /** The active profile. */
    selected?: boolean;
    /** Something is dragged over it: it looks selected. */
    drop?: boolean;
    /** The "new profile" item: a `+` in a dashed box. */
    add?: boolean;
    /** The initials. */
    children?: Snippet;
  }

  let { label, selected = false, drop = false, add = false, children, ...rest }: Props = $props();
</script>

<!-- The button spans the rail so the indicator sits on the rail's edge; the box is the span. -->
<button
  type="button"
  class={{ drop, add }}
  aria-label={label}
  title={label}
  aria-current={selected ? 'true' : undefined}
  {...rest}
>
  <span>
    {#if add}<Icon name="plus" />{:else}{@render children?.()}{/if}
  </span>
</button>

<style>
  button {
    position: relative;
    display: grid;
    flex: none;
    place-items: center;
    width: 100%;
    height: var(--control);
    color: var(--muted);
    font: var(--type-label-md);
  }

  span {
    display: grid;
    place-items: center;
    box-sizing: border-box;
    width: var(--control);
    height: var(--control);
    border-radius: var(--radius-sm);
  }

  button:hover:not(:disabled),
  button[aria-current],
  .drop {
    color: var(--fg);
  }

  button:hover:not(:disabled, .add) span,
  button[aria-current] span,
  .drop span {
    background: var(--border);
  }

  button[aria-current]::before,
  .drop::before {
    position: absolute;
    top: 0;
    bottom: 0;
    left: 0;
    width: var(--indicator);
    background: var(--indicator-color);
    content: '';
  }

  .add span {
    border: 1px dashed var(--border);
  }

  .add:hover:not(:disabled) span {
    border-color: var(--muted);
  }

  /* The focus ring goes round the box, like any other control, not round the rail-wide button. */
  button:focus-visible {
    outline: none;
  }

  button:focus-visible span {
    outline: 2px solid var(--focus);
    outline-offset: 1px;
  }
</style>
