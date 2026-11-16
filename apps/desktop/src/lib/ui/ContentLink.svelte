<!-- `content-link` (DESIGN.md, Components): the `link` inside rendered content; opens outside. -->
<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    href: string;
    onopen: (url: string) => void;
    children: Snippet;
  }

  let { href, onopen, children }: Props = $props();

  function open(event: MouseEvent): void {
    // The webview must never navigate away from the app.
    event.preventDefault();
    onopen(href);
  }
</script>

<a {href} onclick={open}>{@render children()}</a>

<style>
  a:hover {
    text-decoration-color: var(--accent-hover);
  }
</style>
