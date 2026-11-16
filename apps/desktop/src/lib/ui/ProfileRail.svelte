<!-- `profile-rail` (DESIGN.md, Components): the first zone, one item per profile. -->
<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    /** Accessible name of the zone. */
    label: string;
    children: Snippet;
    /** At the foot, as tall as the bottom bar. */
    foot?: Snippet;
  }

  let { label, children, foot }: Props = $props();
</script>

<nav aria-label={label}>
  <!-- Empty strip under the traffic lights: it only drags the window. -->
  <div class="strip" data-tauri-drag-region></div>
  <div class="items">{@render children()}</div>
  {#if foot}<div class="foot">{@render foot()}</div>{/if}
</nav>

<style>
  nav {
    display: flex;
    flex-direction: column;
    min-height: 0;
    /* The frame's colour: the rail and the top strips share it, and the sidebar's panel sits inside. */
    background: var(--bg);
    -webkit-user-select: none;
    user-select: none;
  }

  .strip {
    flex: none;
    height: var(--topbar);
  }

  .items {
    display: flex;
    flex: 1;
    flex-direction: column;
    gap: var(--space-sm);
    min-height: 0;
    padding-bottom: var(--space-sm);
    overflow-x: hidden;
    overflow-y: auto;
  }

  .foot {
    display: grid;
    flex: none;
    place-items: center;
    height: var(--control);
    margin-bottom: var(--gap);
  }
</style>
