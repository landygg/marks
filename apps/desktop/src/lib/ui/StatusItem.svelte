<!-- `status-item` (DESIGN.md, Components): one entry of the status bar; a button only with an action. -->
<script lang="ts">
  import Icon from './Icon.svelte';
  import type { IconName } from './icons';
  import Keys from './Keys.svelte';
  import StatusDot, { type StatusDotKind } from './StatusDot.svelte';

  interface Props {
    text: string;
    icon?: IconName;
    /** A decorative status dot: `text` already says the state. */
    dot?: StatusDotKind;
    /** A shortcut shown as a key after the text. */
    shortcut?: string;
    /** Asks for attention: `on-surface` instead of `secondary`. */
    strong?: boolean;
    /** At most `status-item-max-width`, cutting `text` with an ellipsis: for text of any length. */
    capped?: boolean;
    title?: string;
    onclick?: () => void;
  }

  let {
    text,
    icon,
    dot,
    shortcut,
    strong = false,
    capped = false,
    title,
    onclick,
  }: Props = $props();
</script>

{#snippet body()}
  {#if dot}<StatusDot status={dot} />{/if}
  {#if icon}<Icon name={icon} />{/if}
  <span class="text">{text}</span>
  {#if shortcut}<Keys {shortcut} />{/if}
{/snippet}

{#if onclick}
  <button type="button" class={['item', { strong, capped }]} {title} {onclick}
    >{@render body()}</button
  >
{:else}
  <span class={['item', { strong, capped }]} {title}>{@render body()}</span>
{/if}

<style>
  .item {
    display: flex;
    flex: none;
    align-items: center;
    gap: var(--space-sm);
    max-width: 100%;
    min-width: 0;
    height: var(--control);
    padding: 0 var(--space-sm);
    border-radius: var(--radius-sm);
    color: var(--muted);
    font: var(--type-label-md);
    white-space: nowrap;
  }

  /* Only the entry that can shrink gives way: the extracting title, ahead of the counts. */
  .item:first-child {
    flex: 0 1 auto;
  }

  button.item:hover {
    background: var(--border);
    color: var(--fg);
  }

  .strong {
    color: var(--fg);
  }

  .capped {
    max-width: var(--status-item-max);
  }

  .text {
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
