// What the status bar shows of the background work of the active profile.
import type { Bookmark, Extracting, Stats } from '../../api';

export type StatusEntry =
  | { kind: 'extracting'; id: number; title: string }
  | { kind: 'pending'; count: number }
  | { kind: 'failed'; count: number };

/**
 * The first bookmark of this profile being extracted (by title, or URL when untitled), then how
 * many still wait to be extracted and how many failed, when they are not zero. Other profiles' extractions do not show.
 */
export function statusItems(input: {
  extracting: Extracting[];
  bookmarks: Bookmark[];
  profileId: number;
  stats: Stats | null;
}): StatusEntry[] {
  const { extracting, bookmarks, profileId, stats } = input;
  const entries: StatusEntry[] = [];
  const current = extracting.find((e) => e.profile === profileId);
  // Not in the list yet (a refresh is on its way): nothing to name it by.
  const bookmark = current && bookmarks.find((b) => b.id === current.id);
  if (bookmark)
    entries.push({ kind: 'extracting', id: bookmark.id, title: bookmark.title || bookmark.url });
  // Only what the worker will still extract: without AI, extracted bookmarks stay pending for good.
  if (stats && stats.unextracted > 0) entries.push({ kind: 'pending', count: stats.unextracted });
  if (stats && stats.failed > 0) entries.push({ kind: 'failed', count: stats.failed });
  return entries;
}
