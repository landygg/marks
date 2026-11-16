<script lang="ts">
  import BookmarkRow from './BookmarkRow.svelte';
  import EmptyState from '../../ui/EmptyState.svelte';
  import IconButton from '../../ui/IconButton.svelte';
  import InlineConfirm from '../../ui/InlineConfirm.svelte';
  import Panel from '../../ui/Panel.svelte';
  import Resizer from '../../ui/Resizer.svelte';
  import Topbar from '../../ui/Topbar.svelte';
  import { lib, openAddDialog, runExtractView, select } from './libraryState.svelte';
  import { openPalette } from '../palette/paletteState.svelte';
  import { t } from '../../strings';
  import { LIST, LIST_ID } from '../layout/layout';
  import { layout, setWidth } from '../layout/layoutState.svelte';

  interface Props {
    /** Drawn width, which can be under the saved one in a narrow window. */
    width: number;
    /** The widest the detail leaves room for now. */
    max: number;
    /** A drag of the handle starts or ends. */
    onresizing: (active: boolean) => void;
  }

  let { width, max, onresizing }: Props = $props();

  const rows = $derived(lib.visible);
  const name = $derived(lib.selection.kind === 'all' ? t.allBookmarks : lib.selection.value);
</script>

<section id={LIST_ID} aria-label={t.bookmarksLabel} data-tauri-drag-region>
  <Topbar lights={layout.collapsed}>
    {#snippet title()}<b>{name}</b>{t.separator}{t.bookmarkCount(rows.length)}{/snippet}
    {#snippet end()}
      <IconButton icon="search" label={t.search} onclick={() => openPalette('bookmarks')} />
      <IconButton icon="plus" label={t.addWithShortcut} onclick={openAddDialog} />
    {/snippet}
  </Topbar>

  <div class="body">
    <Panel>
      {#if lib.confirmExtractView}
        {@const n = lib.confirmExtractView.ids.length}
        {@const question = lib.confirmExtractView.remote
          ? `${t.confirmExtractView(n)} ${t.confirmExtractRemote(n)}`
          : t.confirmExtractView(n)}
        <div class="confirm">
          <InlineConfirm
            tone="neutral"
            {question}
            label={question}
            yes={t.extractYes}
            cancel={t.cancel}
            onyes={runExtractView}
            oncancel={() => (lib.confirmExtractView = null)}
          />
        </div>
      {/if}

      {#if lib.bookmarks.length === 0}
        <EmptyState>{t.emptyLibrary}</EmptyState>
      {:else}
        <ul>
          {#each rows as bookmark (bookmark.id)}
            <BookmarkRow {bookmark} selected={bookmark.id === lib.selectedId} onopen={select} />
          {/each}
        </ul>
      {/if}
    </Panel>
    <Resizer
      label={t.layout.listWidth}
      value={width}
      min={LIST.min}
      {max}
      reset={LIST.reset}
      controls={LIST_ID}
      onchange={(v) => setWidth('list', v)}
      onstart={() => onresizing(true)}
      onend={() => onresizing(false)}
    />
  </div>
</section>

<style>
  section {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    padding: 0 var(--gap) var(--gap) 0;
  }

  /* The handle sits over the gap to the right of the panel, from its top edge to its bottom one. */
  .body {
    position: relative;
    display: flex;
    flex: 1;
    min-height: 0;
  }

  .confirm {
    display: flex;
    flex: none;
    padding: var(--space-sm) var(--space-md);
    border-bottom: 1px solid var(--border);
  }

  ul {
    flex: 1;
    min-height: 0;
    margin: 0;
    padding: 0;
    overflow-y: auto;
    list-style: none;
  }
</style>
