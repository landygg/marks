<!-- Toasts for the user's own actions, stacked at the bottom right; none while the panel is open. -->
<script lang="ts">
  import { tick } from 'svelte';
  import { formatTime } from '../../format';
  import { t } from '../../strings';
  import Button from '../../ui/Button.svelte';
  import IconButton from '../../ui/IconButton.svelte';
  import Toast from '../../ui/Toast.svelte';
  import { lib } from '../library/libraryState.svelte';
  import {
    ACTION_ICONS,
    BELL_SELECTOR,
    actionsFor,
    focusAfterClose,
    visibleToasts,
    type NoticeAction,
  } from './notifications';
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
  let stack: HTMLElement;

  const toastBox = (id: number) => stack.querySelector(`[data-toast-id="${id}"]`);

  /** Closes a toast; if the focus was in it, hands it to the next toast, the previous one or the bell. */
  async function close(n: Notification, then?: () => void): Promise<void> {
    const hadFocus = toastBox(n.id)?.contains(document.activeElement) ?? false;
    const next = focusAfterClose(
      visible.shown.map((x) => x.id),
      n.id,
    );
    closeToast(n.id);
    then?.();
    if (!hadFocus) return;
    await tick();
    // An action that moved the focus itself (opening a bookmark or Settings) keeps it there.
    if (document.activeElement && document.activeElement !== document.body) return;
    const target =
      next === null
        ? document.querySelector(BELL_SELECTOR)
        : toastBox(next)?.querySelector('[data-toast-close]');
    if (target instanceof HTMLElement) target.focus();
  }

  const act = (n: Notification, action: NoticeAction) =>
    void close(n, () => runNoticeAction(n, action));
</script>

<div class="stack" role="status" bind:this={stack}>
  {#if visible.hidden > 0}
    <Button variant="link" onclick={openNotifications}>{t.notices.more(visible.hidden)}</Button>
  {/if}
  {#each visible.shown as n (n.id)}
    <Toast
      toastId={n.id}
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
        <IconButton
          icon="close"
          label={t.notices.close}
          data-toast-close
          onclick={() => void close(n)}
        />
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
