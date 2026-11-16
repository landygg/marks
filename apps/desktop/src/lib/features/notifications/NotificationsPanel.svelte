<!-- The notification panel, anchored at the bottom right; the bell, the palette or Esc open and close it. -->
<script lang="ts">
  import { formatTime } from '../../format';
  import { dismissOnOutside } from '../../outside';
  import { t } from '../../strings';
  import IconButton from '../../ui/IconButton.svelte';
  import NotificationItem from '../../ui/NotificationItem.svelte';
  import NotificationPanel from '../../ui/NotificationPanel.svelte';
  import { lib } from '../library/libraryState.svelte';
  import { ACTION_ICONS, BELL_SELECTOR, PANEL_ID, actionsFor } from './notifications';
  import {
    clearNotices,
    closeNotifications,
    dismissNotice,
    markNoticesRead,
    notices,
    runNoticeAction,
    unreadCount,
  } from './notificationsState.svelte';

  const unread = $derived(unreadCount());
  const heading = $derived(
    unread > 0 ? [t.notices.title, t.notices.unread(unread)].join(t.separator) : t.notices.title,
  );
  // The bell lives in another zone: found by its attribute instead of a binding across zones.
  const bell = () => document.querySelector(BELL_SELECTOR);
  // A click elsewhere keeps the focus where it went.
  const closeFromOutside = () => void closeNotifications({ restoreFocus: false });
</script>

{#snippet rows()}
  {#each notices.list as n (n.id)}
    <NotificationItem
      severity={n.severity}
      severityLabel={t.notices.severities[n.severity]}
      count={n.count}
      time={formatTime(n.last_at)}
      message={n.message}
      detail={n.detail}
      unread={!n.read}
      unreadLabel={t.notices.unreadRow}
    >
      {#snippet actions()}
        {#each actionsFor(n, lib.bookmarks, lib.profileId) as action (action)}
          <IconButton
            icon={ACTION_ICONS[action]}
            label={t.notices[action]}
            onclick={() => runNoticeAction(n, action)}
          />
        {/each}
        <IconButton icon="close" label={t.notices.dismiss} onclick={() => dismissNotice(n.id)} />
      {/snippet}
    </NotificationItem>
  {/each}
{/snippet}

{#if notices.open}
  <div class="anchor" {@attach dismissOnOutside(closeFromOutside, bell)}>
    <NotificationPanel
      id={PANEL_ID}
      label={t.notices.title}
      {heading}
      empty={t.notices.empty}
      children={notices.list.length > 0 ? rows : undefined}
    >
      {#snippet actions()}
        <IconButton icon="check" label={t.notices.markRead} onclick={() => markNoticesRead()} />
        <IconButton
          icon="trash"
          label={t.notices.clear}
          onclick={() => clearNotices(lib.profileId)}
        />
        <IconButton icon="close" label={t.notices.close} onclick={() => closeNotifications()} />
      {/snippet}
    </NotificationPanel>
  </div>
{/if}

<style>
  .anchor {
    position: fixed;
    right: var(--gap);
    /* A gap above the bottom bar, which sits a gap above the window edge. */
    bottom: calc(var(--control) + 2 * var(--gap));
  }
</style>
