<!-- `notification-item` (DESIGN.md, Components): one entry of the notification panel or a toast. -->
<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    severity: 'error' | 'warning' | 'info';
    /** The severity in words, so it is not told by colour alone. */
    severityLabel: string;
    /** How many times it happened; shown from 2 on. */
    count: number;
    time: string;
    message: string;
    /** Second line: title, domain or operation. */
    detail: string | null;
    /** `-unread`: the marker bar on the left. */
    unread: boolean;
    /** Says "unread" to screen readers, since the bar is only visual. */
    unreadLabel?: string;
    /** Icon buttons at the bottom right. */
    actions?: Snippet;
  }

  let {
    severity,
    severityLabel,
    count,
    time,
    message,
    detail,
    unread,
    unreadLabel,
    actions,
  }: Props = $props();

  const COUNT_PREFIX = '×';
</script>

<div class={['item', { unread }]}>
  <span class={['severity', severity]}>
    <span class="square" aria-hidden="true"></span>{severityLabel}
    {#if unread && unreadLabel}<span class="hidden">{unreadLabel}</span>{/if}
    {#if count > 1}<span class="count">{COUNT_PREFIX}{count}</span>{/if}
  </span>
  <span class="meta time">{time}</span>
  <p>{message}</p>
  <span class="meta detail">{detail ?? ''}</span>
  {#if actions}<span class="actions">{@render actions()}</span>{/if}
</div>

<style>
  .item {
    position: relative;
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: var(--space-xxs) var(--space-sm);
    padding: var(--space-sm) var(--space-sm) var(--space-sm) var(--space-md);
    color: var(--fg);
  }

  /* Rows of the panel are split by a hairline; a toast holds one, so it has none. */
  .item:not(:first-child) {
    border-top: 1px solid var(--border);
  }

  .unread::before {
    position: absolute;
    inset: 0 auto 0 0;
    width: var(--indicator);
    background: var(--accent);
    content: '';
  }

  .severity {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    font: var(--type-label-sm);
    letter-spacing: var(--tracking-label-sm);
    text-transform: uppercase;
  }

  /* Read, not shown. */
  .hidden {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip-path: inset(50%);
    white-space: nowrap;
  }

  .square {
    flex: none;
    width: var(--dot);
    height: var(--dot);
    border-radius: var(--radius-sm);
    background: currentColor;
  }

  .error {
    color: var(--danger);
  }

  .warning {
    color: var(--warn);
  }

  .info {
    color: var(--muted);
  }

  .count {
    padding: 0 var(--space-xs);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--bg);
    color: var(--fg);
    letter-spacing: normal;
  }

  .meta {
    color: var(--muted);
    font: var(--type-label-md);
  }

  .time {
    text-align: right;
  }

  p {
    grid-column: 1 / -1;
    margin: 0;
    font: var(--type-body-md);
    overflow-wrap: anywhere;
  }

  .detail {
    align-self: center;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .actions {
    display: flex;
    grid-column: 2;
    justify-content: flex-end;
    margin: calc(-1 * var(--space-xs)) calc(-1 * var(--space-xs)) calc(-1 * var(--space-xs)) 0;
  }
</style>
