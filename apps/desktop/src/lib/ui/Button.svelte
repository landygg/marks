<!-- `button-primary`, `button-secondary`, `button-danger` (DESIGN.md, Components). -->
<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLButtonAttributes } from 'svelte/elements';

  interface Props extends HTMLButtonAttributes {
    /** `link`: a discreet text action, for an overlay where a boxed button would weigh too much. */
    variant?: 'primary' | 'secondary' | 'danger' | 'link';
    children: Snippet;
  }

  let { variant = 'secondary', type = 'button', children, ...rest }: Props = $props();
</script>

<button class={variant} {type} {...rest}>{@render children()}</button>

<style>
  button {
    flex: none;
    height: var(--control);
    padding: 0 var(--space-md);
    border: 1px solid var(--border);
    background: var(--bg);
    color: var(--fg);
    font: var(--type-body-md);
    white-space: nowrap;
  }

  button:hover:not(:disabled) {
    background: var(--border);
  }

  .primary {
    border-color: var(--accent);
    background: var(--accent);
    color: var(--on-accent);
  }

  .primary:hover:not(:disabled) {
    border-color: var(--accent-hover);
    background: var(--accent-hover);
  }

  .danger {
    border-color: transparent;
    background: none;
    color: var(--danger);
  }

  /* `button-link`: just the words, underlined under the cursor. */
  .link,
  .link:hover:not(:disabled) {
    height: auto;
    padding: 0;
    border-color: transparent;
    background: none;
    color: var(--muted);
  }

  .link:hover:not(:disabled) {
    text-decoration: underline;
  }
</style>
