<!-- `toast` (DESIGN.md, Components): a notification item in its own box, with the severity's bar. -->
<script lang="ts">
  import type { ComponentProps } from 'svelte';
  import NotificationItem from './NotificationItem.svelte';

  interface Props extends Omit<ComponentProps<typeof NotificationItem>, 'unread'> {
    onpointerenter?: () => void;
    onpointerleave?: () => void;
    onfocusin?: () => void;
    onfocusout?: () => void;
  }

  let { onpointerenter, onpointerleave, onfocusin, onfocusout, ...item }: Props = $props();
</script>

<!-- The handlers only pause and resume its timer; the toast itself is not a control. No live role of
     its own: the stack around it (`role="status"`) announces it, once. -->
<div class={item.severity} role="group" {onpointerenter} {onpointerleave} {onfocusin} {onfocusout}>
  <NotificationItem {...item} unread={false} />
</div>

<style>
  div {
    position: relative;
    width: var(--notification-panel);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--surface);
  }

  div::before {
    position: absolute;
    z-index: 1;
    inset: 0 auto 0 0;
    width: var(--indicator);
    background: var(--muted);
    content: '';
  }

  .error::before {
    background: var(--danger);
  }

  .warning::before {
    background: var(--warn);
  }
</style>
