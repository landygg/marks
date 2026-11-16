<!-- Toasts for the user's own actions, stacked at the bottom right; none while the panel is open. -->
<script lang="ts">
  import { formatTime } from '../../format';
  import { t } from '../../strings';
  import Button from '../../ui/Button.svelte';
  import IconButton from '../../ui/IconButton.svelte';
  import Toast from '../../ui/Toast.svelte';
  import { lib } from '../library/libraryState.svelte';
  import { ACTION_ICONS, actionsFor, visibleToasts, type NoticeAction } from './notifications';
  import type { Notification } from '../../api';
  import {
    closeToast,
    notices,
    openNotifications,
    pauseToast,
    resumeToast,
    runNoticeAction,
  } from './notificationsState.svelte';

  const visible = $derived(visibleToasts(notices.toasts));

  function act(n: Notification, action: NoticeAction): void {
    closeToast(n.id);
    runNoticeAction(n, action);
  }
</script>

<div class="stack" role="status">
  {#if visible.hidden > 0}
    <Button variant="link" onclick={openNotifications}>{t.notices.more(visible.hidden)}</Button>
  {/if}
  {#each visible.shown as n (n.id)}
    <Toast
      severity={n.severity}
      severityLabel={t.notices.severities[n.severity]}
      count={n.count}
      time={formatTime(n.last_at)}
      message={n.message}
      detail={n.detail}
      onpointerenter={() => pauseToast(n.id, 'pointer')}
      onpointerleave={() => resumeToast(n.id, 'pointer')}
      onfocusin={() => pauseToast(n.id, 'focus')}
      onfocusout={() => resumeToast(n.id, 'focus')}
    >
      {#snippet actions()}
        {#each actionsFor(n, lib.bookmarks, lib.profileId) as action (action)}
          <IconButton
            icon={ACTION_ICONS[action]}
            label={t.notices[action]}
            onclick={() => act(n, action)}
          />
        {/each}
        <IconButton icon="close" label={t.notices.close} onclick={() => closeToast(n.id)} />
      {/snippet}
    </Toast>
  {/each}
</div>

<style>
  .stack {
    position: fixed;
    right: var(--gap);
    /* A gap above the bottom bar, which sits a gap above the window edge. */
    bottom: calc(var(--control) + 2 * var(--gap));
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: var(--space-sm);
  }
</style>
