<!-- The bottom bar: Commands under the sidebar, the background work in the
     middle and the notifications bell at the right. -->
<script lang="ts">
  import { t } from '../../strings';
  import IconButton from '../../ui/IconButton.svelte';
  import StatusBar from '../../ui/StatusBar.svelte';
  import StatusItem from '../../ui/StatusItem.svelte';
  import { choose, lib, select } from '../library/libraryState.svelte';
  import { ALL, FAILED } from '../library/sidebar';
  import {
    notices,
    toggleNotifications,
    unreadCount,
  } from '../notifications/notificationsState.svelte';
  import { PANEL_ID } from '../notifications/notifications';
  import { openPalette } from '../palette/paletteState.svelte';
  import { statusItems } from './status';
  import { fold, layout } from '../layout/layoutState.svelte';

  const items = $derived(
    statusItems({
      extracting: lib.extracting,
      bookmarks: lib.bookmarks,
      profileId: lib.profileId,
      stats: lib.stats,
    }),
  );

  /** Shows the bookmark being extracted, from Settings or any selection, as "Open" does. */
  function reveal(id: number): void {
    choose(ALL);
    select(id);
  }
</script>

<StatusBar label={t.statusBar.label} sidebarHidden={layout.collapsed} animate={fold.active}>
  {#snippet sidebar()}
    <StatusItem
      icon="command"
      text={t.commandsLabel}
      shortcut={t.commandsShortcut}
      onclick={() => openPalette('commands')}
    />
  {/snippet}
  {#each items as item (item.kind)}
    {#if item.kind === 'extracting'}
      {@const text = t.statusBar.extracting(item.title)}
      <StatusItem dot="extracting" {text} title={text} onclick={() => reveal(item.id)} />
    {:else if item.kind === 'pending'}
      <StatusItem text={t.statusBar.pending(item.count)} />
    {:else}
      <StatusItem text={t.statusBar.failed(item.count)} strong onclick={() => choose(FAILED)} />
    {/if}
  {/each}
  {#snippet end()}
    <IconButton
      icon="bell"
      label={t.notices.bell(unreadCount())}
      marked={unreadCount() > 0}
      aria-expanded={notices.open}
      aria-controls={PANEL_ID}
      onclick={toggleNotifications}
      data-notifications-trigger
    />
  {/snippet}
</StatusBar>
