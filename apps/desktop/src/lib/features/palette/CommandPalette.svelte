<script lang="ts">
  import { tick } from 'svelte';
  import { commands, filterCommands, type PaletteMode } from './commands';
  import { domainOf } from '../../format';
  import {
    askDeleteBookmark,
    askDeleteGroup,
    askExtractView,
    askReextract,
    askRetryFailed,
    choose,
    lib,
    openSettings,
    moveTag,
    openAddDialog,
    openLink,
    retryBookmark,
    select,
    startNewGroup,
    startRenameGroup,
    toggleExpanded,
  } from '../library/libraryState.svelte';
  import { toggleSidebar } from '../layout/layoutState.svelte';
  import { moveSelectedTo, openNewProfile, switchTo } from '../profiles/profilesState.svelte';
  import { closePalette, openPalette, palette, setMode, setQuery } from './paletteState.svelte';
  import {
    excerpt,
    highlight,
    moveIndex,
    pointerMoved,
    rowAfterPointerLeaves,
    type Point,
  } from './searchText';
  import { modal } from '../../modal';
  import { ENTER_KEY, isComposing, paletteModeShortcut } from '../../shortcuts';
  import { ALL, FAILED } from '../library/sidebar';
  import { isBlocked } from '../library/detailMeta';
  import { isExtracting } from '../library/extracting';
  import { t } from '../../strings';
  import { askLanguage, language, loadLanguage } from '../settings/languageState.svelte';
  import { setTheme } from '../settings/themeState.svelte';
  import {
    clearNotices,
    notices,
    openNotifications,
  } from '../notifications/notificationsState.svelte';
  import CommandItem from '../../ui/CommandItem.svelte';
  import PaletteSearch from '../../ui/PaletteSearch.svelte';
  import PaletteTab from '../../ui/PaletteTab.svelte';
  import StatusDot from '../../ui/StatusDot.svelte';

  const EXCERPT_MAX_CHARS = 120;
  const MODES: PaletteMode[] = ['commands', 'bookmarks', 'tags'];

  void loadLanguage();
  let dialog: HTMLDialogElement;
  let input = $state<HTMLInputElement>();
  const uid = $props.id();
  const optionId = (index: number): string => `${uid}-option-${index}`;

  // Plain (non-reactive): only compared with the next move. `null` until the first move after opening.
  let pointer: Point | null = null;

  // Plain: the row chosen before the pointer took over, given back when it leaves the list
  // (see `rowAfterPointerLeaves`). `null` while the keyboard owns the selection.
  let chosen: number | null = null;

  /** Selects the row under the pointer, if the pointer really moved (see `pointerMoved`). */
  function pointAt(event: PointerEvent, index: number): void {
    const next = { x: event.clientX, y: event.clientY };
    if (pointerMoved(pointer, next)) {
      chosen ??= palette.index;
      palette.index = index;
    }
    pointer = next;
  }

  function leaveList(): void {
    palette.index = rowAfterPointerLeaves(palette.index, chosen, count);
    chosen = null;
  }

  const selected = $derived(lib.bookmarks.find((b) => b.id === lib.selectedId));
  // The tag selected in the sidebar and the group it is in, for the "move tag" commands.
  const selectedTag = $derived.by(() => {
    if (lib.selection.kind !== 'tag') return null;
    const name = lib.selection.value;
    return { name, groupId: lib.groups.find((g) => g.tags.includes(name))?.id ?? null };
  });

  const allCommands = $derived(
    commands(
      {
        add: openAddDialog,
        searchBookmarks: () => openPalette('bookmarks'),
        newGroup: () => startNewGroup(),
        toggleExpanded,
        toggleSidebar,
        openSettings,
        openInBrowser: () => selected && openLink(selected.original_url),
        retry: () => selected && void retryBookmark(lib.profileId, selected.id),
        reextract: () => void askReextract('defuddle'),
        extractWithAi: () => void askReextract('ai'),
        deleteBookmark: askDeleteBookmark,
        renameGroup: startRenameGroup,
        deleteGroup: askDeleteGroup,
        moveTag: (tag, groupId) => void moveTag(lib.profileId, tag, groupId),
        setTheme,
        newProfile: openNewProfile,
        switchProfile: (id) => void switchTo(id),
        moveToProfile: (id) => void moveSelectedTo(id),
        notifications: openNotifications,
        clearNotifications: () => void clearNotices(lib.profileId),
        extractView: () => void askExtractView(),
        retryFailed: () => {
          choose(FAILED);
          void askRetryFailed();
        },
        setLanguage: askLanguage,
      },
      {
        bookmark: selected
          ? {
              failed: selected.status === 'failed',
              blocked: isBlocked(selected),
              busy: isExtracting(lib.extracting, lib.profileId, selected.id),
              imported: selected.status === 'imported',
            }
          : null,
        groups: lib.groups,
        tag: selectedTag,
        profiles: lib.profiles,
        activeProfile: lib.profileId,
        extractionModel: lib.extractionModel,
        hasNotifications: notices.list.length > 0,
        language: language.setting,
        importedInView: lib.visible.filter((b) => b.status === 'imported').length,
        failed: lib.stats?.failed ?? 0,
      },
    ),
  );

  const query = $derived(palette.query.trim());
  const commandRows = $derived(filterCommands(allCommands, query));
  const tagRows = $derived(lib.tags.filter((tag) => tag.includes(query.toLowerCase())));
  const count = $derived(
    palette.mode === 'commands'
      ? commandRows.length
      : palette.mode === 'bookmarks'
        ? palette.hits.length
        : tagRows.length,
  );

  // Each opening starts with the keyboard owning the selection.
  function resetPointer(): void {
    pointer = null;
    chosen = null;
  }

  /**
   * Keeps the selected row in view: it runs again when the row is selected, appears or the palette
   * opens. After `tick`: on opening, the dialog only shows once this update is done.
   */
  const reveal = (selected: boolean) => (row: HTMLElement) => {
    if (selected) void tick().then(() => row.scrollIntoView({ block: 'nearest' }));
  };

  /** Runs the row at `index`; nothing happens when the list is empty. */
  function run(index: number): void {
    if (palette.mode === 'commands') {
      const command = commandRows[index];
      if (!command) return;
      closePalette();
      // Close now, not in the effect: a command may focus something outside the modal, which stays
      // inert (and takes the focus back) until the dialog is really closed.
      dialog.close();
      command.run();
    } else if (palette.mode === 'bookmarks') {
      // While a search is pending the rows shown belong to the previous query.
      const hit = palette.searching ? undefined : palette.hits[index];
      if (!hit) return;
      closePalette();
      choose(ALL);
      select(hit.bookmark.id);
    } else {
      const tag = tagRows[index];
      if (tag === undefined) return;
      closePalette();
      choose({ kind: 'tag', value: tag });
    }
  }

  // The `close` event is asynchronous: a command may have reopened the palette since (the search
  // command does), and that stale event must not close it again.
  function onDialogClose(): void {
    if (!dialog.open) closePalette();
  }

  /** The footer's run action: the same as ↵; the caret stays in the search field if the palette stays open. */
  function runSelected(): void {
    run(palette.index);
    if (palette.open) input?.focus();
  }

  /** Switches mode from a tab; the caret stays in the search field. */
  function pickMode(mode: PaletteMode): void {
    setMode(mode);
    input?.focus();
  }

  function onKeydown(event: KeyboardEvent): void {
    if (isComposing(event)) return; // the input method's Enter and arrows pick its candidate
    chosen = null; // any key (arrows, typing, ⌘1…) hands the selection back to the keyboard
    const mode = paletteModeShortcut(event);
    if (mode) {
      event.preventDefault();
      setMode(mode);
    } else if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      event.preventDefault();
      palette.index = moveIndex(palette.index, event.key === 'ArrowDown' ? 1 : -1, count);
    } else if (event.key === ENTER_KEY) {
      event.preventDefault();
      run(palette.index);
    }
  }
</script>

{#snippet marked(text: string)}
  <!-- eslint-disable-next-line svelte/require-each-key -- pieces of one string have no identity of their own: a key would only be the index. -->
  {#each highlight(text, query) as piece}{#if piece.match}<mark>{piece.text}</mark
      >{:else}{piece.text}{/if}{/each}
{/snippet}

<dialog
  bind:this={dialog}
  {@attach modal(palette.open, resetPointer)}
  aria-label={t.paletteLabel}
  onclose={onDialogClose}
  onkeydown={onKeydown}
>
  <!-- Inside a modal dialog autofocus is the expected focus target, not a surprise jump. -->
  <PaletteSearch
    bind:ref={input}
    role="combobox"
    aria-label={t.palettePlaceholder[palette.mode]}
    aria-expanded="true"
    aria-controls="{uid}-list"
    aria-activedescendant={count > 0 ? optionId(palette.index) : undefined}
    aria-autocomplete="list"
    placeholder={t.palettePlaceholder[palette.mode]}
    autofocus
    value={palette.query}
    oninput={(e) => setQuery(e.currentTarget.value)}
  />

  <!-- Pointer leaving is mouse-only, so it sits on a wrapper: the listbox stays unfocusable, as the
       combobox pattern wants (focus stays in the field, the row is named by aria-activedescendant). -->
  <div class="list" role="presentation" onpointerleave={leaveList}>
    <div id="{uid}-list" role="listbox" aria-label={t.paletteModes[palette.mode]}>
      {#if palette.mode === 'commands'}
        {#each commandRows as command, i (command.id)}
          <CommandItem
            id={optionId(i)}
            selected={i === palette.index}
            {@attach reveal(i === palette.index && palette.open)}
            icon={command.icon}
            shortcut={command.shortcut}
            onpointermove={(e) => pointAt(e, i)}
            onclick={() => run(i)}
          >
            {command.label}
          </CommandItem>
        {/each}
      {:else if palette.mode === 'bookmarks'}
        {#each palette.hits as hit, i (hit.bookmark.id)}
          <CommandItem
            id={optionId(i)}
            selected={i === palette.index}
            {@attach reveal(i === palette.index && palette.open)}
            meta={domainOf(hit.bookmark.url)}
            onpointermove={(e) => pointAt(e, i)}
            onclick={() => run(i)}
          >
            {#snippet lead()}
              <StatusDot status={hit.bookmark.status} label={t.status[hit.bookmark.status]} />
            {/snippet}
            {@render marked(hit.bookmark.title)}
            {#snippet detail()}
              {@render marked(
                excerpt(hit.snippet ?? hit.bookmark.summary ?? '', query, EXCERPT_MAX_CHARS),
              )}
            {/snippet}
          </CommandItem>
        {/each}
      {:else}
        {#each tagRows as tag, i (tag)}
          <CommandItem
            id={optionId(i)}
            selected={i === palette.index}
            {@attach reveal(i === palette.index && palette.open)}
            mono
            onpointermove={(e) => pointAt(e, i)}
            onclick={() => run(i)}
          >
            {@render marked(tag)}
          </CommandItem>
        {/each}
      {/if}
    </div>
  </div>
  <!-- Always in the DOM, so a screen reader hears what appears in it; empty when there is nothing to say. -->
  <p role="status" class={{ 'error-text': palette.error }}>
    {#if palette.error}{palette.error}{:else if count === 0 && !palette.searching && (palette.mode !== 'bookmarks' || query !== '')}{t.paletteEmpty}{/if}
  </p>

  <footer>
    {#each MODES as mode (mode)}
      <PaletteTab
        selected={palette.mode === mode}
        shortcut={t.paletteModeKeys[mode]}
        onclick={() => pickMode(mode)}
      >
        {t.paletteModes[mode]}
      </PaletteTab>
    {/each}
    <div class="actions">
      <PaletteTab shortcut={t.paletteRunKey} onclick={runSelected}>
        {palette.mode === 'commands' ? t.paletteRun : t.paletteOpen}
      </PaletteTab>
      <!-- Closing the dialog is what esc does; its `close` event resets the palette. -->
      <PaletteTab shortcut={t.paletteCloseKey} onclick={() => dialog.close()}
        >{t.paletteClose}</PaletteTab
      >
    </div>
  </footer>
</dialog>

<style>
  /* DESIGN.md: 640px wide, 96px from the top edge. */
  dialog {
    width: min(var(--palette), calc(100vw - 2 * var(--space-lg)));
    margin: var(--palette-top) auto auto;
    padding: 0;
  }

  .list {
    max-height: var(--palette-list);
    padding: var(--space-xs) 0;
    overflow-y: auto;
  }

  /* The empty and error messages of the list. */
  p {
    margin: 0;
    padding: var(--space-sm) var(--space-md);
  }

  p:empty {
    padding: 0;
  }

  p:not(.error-text) {
    color: var(--muted);
  }

  /* Wraps so longer translations push the actions to a second row instead of clipping. */
  footer {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-xs);
    padding: var(--space-xs) var(--space-sm);
    border-top: 1px solid var(--border);
    color: var(--muted);
  }

  /* Pushes the actions (run, close) to the right edge. */
  .actions {
    display: flex;
    gap: var(--space-xs);
    margin-left: auto;
  }
</style>
