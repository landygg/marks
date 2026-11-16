<!-- `notification-panel` (DESIGN.md, Components): the notification list over the zones. It does not
     place itself: the screen anchors it. -->
<script lang="ts">
  import type { Snippet } from 'svelte';
  import EmptyState from './EmptyState.svelte';

  interface Props {
    /** Accessible name of the region. */
    label: string;
    /** For the `aria-controls` of the control that opens it. */
    id: string;
    /** Text of the header ("Notifications · 2 unread"). */
    heading: string;
    /** Icon buttons at the right of the header. */
    actions: Snippet;
    /** Shown when there are no rows. */
    empty: string;
    /** The rows; left out when there are none. */
    children?: Snippet;
  }

  let { label, id, heading, actions, empty, children }: Props = $props();

  // It only exists while open, so it takes the focus as it appears; Tab goes on to its buttons.
  const takeFocus = (section: HTMLElement) => section.focus();
</script>

<section {id} aria-label={label} tabindex="-1" {@attach takeFocus}>
  <header>
    <span>{heading}</span>
    {@render actions()}
  </header>
  {#if children}
    <div class="list">{@render children()}</div>
  {:else}
    <EmptyState>{empty}</EmptyState>
  {/if}
</section>

<style>
  section {
    display: flex;
    flex-direction: column;
    width: var(--notification-panel);
    max-height: var(--notification-panel-max);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--surface);
    color: var(--fg);
  }

  header {
    display: flex;
    flex: none;
    align-items: center;
    gap: var(--space-xxs);
    padding: var(--space-xs) var(--space-xs) var(--space-xs) var(--space-md);
    border-bottom: 1px solid var(--border);
    color: var(--muted);
    font: var(--type-label-md);
  }

  span {
    flex: 1;
    min-width: 0;
  }

  .list {
    display: flex;
    flex-direction: column;
    overflow-y: auto;
  }
</style>
