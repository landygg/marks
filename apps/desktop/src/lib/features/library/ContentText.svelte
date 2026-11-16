<!-- An HTML tag outside the reader's whitelist: its children (text, allowed tags) and nothing else. -->
<script lang="ts">
  import type { Snippet } from 'svelte';

  let { children }: { children?: Snippet } = $props();
</script>

{@render children?.()}
