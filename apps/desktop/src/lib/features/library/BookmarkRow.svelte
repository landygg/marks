<script lang="ts">
  import type { Bookmark } from '../../api';
  import { domainOf, formatDate } from '../../format';
  import { isBlocked, rowSummary } from './detailMeta';
  import { isExtracting } from './extracting';
  import { lib } from './libraryState.svelte';
  import { showBookmarkMenu } from '../profiles/profileMenu';
  import { bookmarkDragData } from '../profiles/profiles';
  import { BOOKMARK_MIME } from '../profiles/profilesState.svelte';
  import { t } from '../../strings';
  import ListRow from '../../ui/ListRow.svelte';
  import StatusDot from '../../ui/StatusDot.svelte';

  interface Props {
    bookmark: Bookmark;
    selected: boolean;
    onopen: (id: number) => void;
  }

  let { bookmark, selected, onopen }: Props = $props();

  function startDrag(event: DragEvent): void {
    if (!event.dataTransfer) return;
    event.dataTransfer.setData(BOOKMARK_MIME, bookmarkDragData(lib.profileId, bookmark.id));
    event.dataTransfer.effectAllowed = 'move';
  }

  // A bookmark selected from elsewhere (the palette) may be outside the visible part of the list.
  // Only the selected row reads `revealCount`, which scrolls again when the same row is selected
  // again; a refresh keeps the rows, so it never snaps the list back.
  function reveal(row: HTMLElement): void {
    if (!selected) return;
    void lib.revealCount;
    row.scrollIntoView({ block: 'nearest' });
  }

  const busy = $derived(isExtracting(lib.extracting, lib.profileId, bookmark.id));
  const statusLabel = $derived(busy ? t.extracting : t.status[bookmark.status]);
  const statusTitle = $derived(
    busy
      ? statusLabel
      : isBlocked(bookmark)
        ? t.blocked.dot
        : bookmark.status === 'failed' && bookmark.error
          ? bookmark.error
          : statusLabel,
  );
</script>

<li {@attach reveal}>
  <ListRow
    title={bookmark.title}
    summary={rowSummary(bookmark)}
    meta={domainOf(bookmark.url) + t.separator + formatDate(bookmark.created_at)}
    {selected}
    draggable={lib.profiles.length > 1 ? 'true' : undefined}
    ondragstart={startDrag}
    onclick={() => onopen(bookmark.id)}
    oncontextmenu={(e) => {
      e.preventDefault();
      void showBookmarkMenu(bookmark.id, lib.profiles, lib.profileId);
    }}
  >
    {#snippet lead()}<StatusDot
        status={busy ? 'extracting' : bookmark.status}
        label={statusTitle}
      />{/snippet}
  </ListRow>
</li>
