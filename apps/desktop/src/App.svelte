<script lang="ts">
  import { onMount } from 'svelte';
  import AddDialog from './lib/features/library/AddDialog.svelte';
  import CommandPalette from './lib/features/palette/CommandPalette.svelte';
  import Detail from './lib/features/library/Detail.svelte';
  import ProfileDialog from './lib/features/profiles/ProfileDialog.svelte';
  import Profiles from './lib/features/profiles/Profiles.svelte';
  import BookmarkList from './lib/features/library/BookmarkList.svelte';
  import NotificationsPanel from './lib/features/notifications/NotificationsPanel.svelte';
  import Toasts from './lib/features/notifications/Toasts.svelte';
  import {
    closeNotifications,
    notices,
  } from './lib/features/notifications/notificationsState.svelte';
  import Settings from './lib/features/settings/Settings.svelte';
  import StatusBar from './lib/features/statusbar/StatusBar.svelte';
  import Sidebar from './lib/features/library/Sidebar.svelte';
  import {
    lib,
    openAddDialog,
    refresh,
    startListening,
    toggleSettings,
  } from './lib/features/library/libraryState.svelte';
  import { closePalette, openPalette, palette } from './lib/features/palette/paletteState.svelte';
  import { profileDialog, switchTo } from './lib/features/profiles/profilesState.svelte';
  import './lib/features/settings/cornersState.svelte';
  import { fit, room } from './lib/features/layout/layout';
  import { FOLD_MS, fold, layout, toggleSidebar } from './lib/features/layout/layoutState.svelte';
  import { applyTheme } from './lib/features/settings/themeState.svelte';
  import {
    ESCAPE_KEY,
    isNewShortcut,
    isPaletteShortcut,
    isSettingsShortcut,
    isSidebarShortcut,
    profileShortcut,
  } from './lib/shortcuts';

  let innerWidth = $state(window.innerWidth);
  // A handle is being dragged: the window keeps its cursor.
  let resizing = $state(false);
  const onresizing = (active: boolean) => (resizing = active);

  const listShown = $derived(lib.view !== 'settings' && !lib.expanded);
  const widths = $derived(fit(layout, innerWidth, listShown));

  function onKeydown(e: KeyboardEvent): void {
    if (isPaletteShortcut(e)) {
      e.preventDefault();
      if (palette.open) closePalette();
      else openPalette('commands');
      return;
    }
    // The palette is modal: the other global shortcuts wait until it closes.
    if (palette.open) return;
    // Also from a field: the shortcut types nothing. Not behind the modal dialogs, whose focus
    // would return to an element the fold made inert.
    if (isSidebarShortcut(e)) {
      e.preventDefault();
      if (!profileDialog.open && !lib.addDialogOpen) toggleSidebar();
      return;
    }
    // The dialogs close themselves on Esc, and an inline question or field prevents it; the panel
    // only takes an Esc nothing else handled.
    if (
      e.key === ESCAPE_KEY &&
      !e.defaultPrevented &&
      notices.open &&
      !profileDialog.open &&
      !lib.addDialogOpen
    ) {
      void closeNotifications();
      return;
    }
    const position = profileShortcut(e);
    if (position !== null) {
      e.preventDefault();
      const profile = lib.profiles[position];
      // The profile and add dialogs are modal too: no switching behind them (an added bookmark
      // would land in the new profile).
      if (profile && !profileDialog.open && !lib.addDialogOpen) void switchTo(profile.id);
    } else if (isNewShortcut(e)) {
      e.preventDefault();
      openAddDialog();
    } else if (isSettingsShortcut(e)) {
      e.preventDefault();
      toggleSettings();
    }
  }

  onMount(() => {
    applyTheme();
    void refresh();
    const unlisten = startListening();
    return () => void unlisten.then((stop) => stop());
  });
</script>

<svelte:window bind:innerWidth onkeydown={onKeydown} />

<!-- The frame drags the window where nothing else is: each zone carries the attribute for its own
     gaps, and Tauri only drags when the pressed element itself has it. -->
<div
  class={['window', { collapsed: layout.collapsed, folding: fold.active, resizing }]}
  style:--sidebar="{widths.sidebar}px"
  style:--list="{widths.list}px"
  style:--fold="{FOLD_MS}ms"
  data-tauri-drag-region
>
  <div class="rail"><Profiles /></div>
  <main class={{ single: !listShown }}>
    <!-- Clips the sidebar, which keeps its width, while its column narrows. -->
    <div class="sidebar" inert={layout.collapsed}>
      <Sidebar
        width={widths.sidebar}
        max={room('sidebar', layout, innerWidth, listShown)}
        {onresizing}
      />
    </div>
    {#if lib.view === 'settings'}
      <div class="settings" data-tauri-drag-region><Settings /></div>
    {:else}
      {#if !lib.expanded}
        <BookmarkList
          width={widths.list}
          max={room('list', layout, innerWidth, listShown)}
          {onresizing}
        />
      {/if}
      <Detail />
    {/if}
  </main>
  <div class="bar"><StatusBar /></div>
</div>

<AddDialog />
<ProfileDialog />
<CommandPalette />
<NotificationsPanel />
<Toasts />

<style>
  /* The rail down the whole height, the zones over the bottom bar (DESIGN.md, Layout). */
  .window {
    display: grid;
    grid-template-columns: var(--rail) minmax(0, 1fr);
    grid-template-rows: minmax(0, 1fr) auto;
    height: 100vh;
  }

  .rail {
    display: grid;
    grid-row: 1 / -1;
    min-height: 0;
  }

  /* Each zone carries the gap on its right inside its own column, so it goes with the column. */
  main {
    display: grid;
    grid-column: 2;
    grid-row: 1;
    grid-template-columns:
      calc(var(--sidebar) + var(--gap)) calc(var(--list) + var(--gap))
      minmax(calc(var(--detail-min) + var(--gap)), 1fr);
    min-height: 0;
  }

  /* Only a fold animates; every other width change snaps. */
  .folding main {
    transition: grid-template-columns var(--fold) ease-out;
  }

  /* No list: the detail expanded, or Settings. */
  main.single {
    grid-template-columns: calc(var(--sidebar) + var(--gap)) minmax(0, 1fr);
  }

  /* Folded: the sidebar's column and its gap go to 0. */
  .collapsed main {
    grid-template-columns:
      0 calc(var(--list) + var(--gap))
      minmax(calc(var(--detail-min) + var(--gap)), 1fr);
  }

  .collapsed main.single {
    grid-template-columns: 0 minmax(0, 1fr);
  }

  /* WebKit takes the cursor from the element under the pointer even while the handle holds the
     capture: the whole window keeps the handle's cursor until the drag ends. */
  .resizing,
  .resizing :global(*) {
    cursor: col-resize;
  }

  @media (prefers-reduced-motion: reduce) {
    .folding main {
      transition: none;
    }
  }

  /* Open, unclipped, so the handle's focus ring may cross the column's edge. Folded or folding, the
     sidebar keeps its width and is clipped by the narrowing column. */
  .sidebar {
    display: grid;
    min-width: 0;
    min-height: 0;
  }

  .collapsed .sidebar,
  .folding .sidebar {
    overflow: hidden;
  }

  .settings {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    padding: 0 var(--gap) var(--gap) 0;
  }

  .bar {
    grid-column: 2;
    grid-row: 2;
    margin: 0 var(--gap) var(--gap) 0;
  }
</style>
