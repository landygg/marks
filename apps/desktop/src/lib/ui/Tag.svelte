<!-- `tag` (DESIGN.md, Components). -->
<script lang="ts">
  import type { Snippet } from 'svelte';
  import Icon from './Icon.svelte';

  interface Props {
    children: Snippet;
    /** With it, the tag carries a remove button. */
    onremove?: () => void;
    /** Accessible name and tooltip of the remove button. */
    removeLabel?: string;
    disabled?: boolean;
  }

  let { children, onremove, removeLabel, disabled = false }: Props = $props();
</script>

<span class={{ removable: onremove }}>
  {@render children()}
  {#if onremove}
    <button
      type="button"
      aria-label={removeLabel}
      title={removeLabel}
      {disabled}
      onclick={onremove}
    >
      <Icon name="close" small />
    </button>
  {/if}
</span>

<style>
  span {
    display: inline-flex;
    align-items: center;
    gap: var(--space-xs);
    height: var(--tag);
    /* The border is part of the inset. */
    padding: 0 calc(var(--space-sm) - 1px);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--muted);
    font: var(--type-label-md);
  }

  .removable {
    padding-right: var(--space-xxs);
  }

  button {
    display: grid;
    flex: none;
    place-items: center;
    width: var(--icon);
    height: var(--icon);
  }

  button:hover:not(:disabled) {
    color: var(--fg);
  }
</style>
