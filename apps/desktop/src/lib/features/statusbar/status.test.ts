import { describe, expect, it } from 'vitest';
import type { Bookmark, Stats } from '../../api';
import { statusItems } from './status';

const bookmark = (id: number, title: string, url = `https://e.com/${id}`) =>
  ({ id, title, url }) as Bookmark;
// `pending` counts every pending bookmark; `unextracted`, those still waiting to be extracted.
const stats = (
  unextracted: number,
  failed: number,
  pending = unextracted,
  imported = 0,
): Stats => ({
  total: 10,
  pending,
  failed,
  unsummarized: 0,
  unextracted,
  imported,
});

describe('statusItems', () => {
  it('extracting_here_shows_its_title', () => {
    const items = statusItems({
      extracting: [{ profile: 1, id: 3 }],
      bookmarks: [bookmark(3, 'T')],
      profileId: 1,
      stats: stats(2, 1),
    });
    expect(items).toEqual([
      { kind: 'extracting', id: 3, title: 'T' },
      { kind: 'pending', count: 2 },
      { kind: 'failed', count: 1 },
    ]);
  });

  it('imported_is_not_pending_work', () => {
    const items = statusItems({
      extracting: [],
      bookmarks: [],
      profileId: 1,
      stats: stats(0, 0, 0, 3),
    });
    expect(items).toEqual([]);
  });

  it('extracting_elsewhere_is_ignored', () => {
    const items = statusItems({
      extracting: [{ profile: 2, id: 3 }],
      bookmarks: [bookmark(3, 'T')],
      profileId: 1,
      stats: stats(0, 0),
    });
    expect(items).toEqual([]);
  });

  it('untitled_falls_back_to_the_url', () => {
    const items = statusItems({
      extracting: [{ profile: 1, id: 3 }],
      bookmarks: [bookmark(3, '', 'https://e.com/a')],
      profileId: 1,
      stats: null,
    });
    expect(items).toEqual([{ kind: 'extracting', id: 3, title: 'https://e.com/a' }]);
  });

  it('pending_and_failed_only_when_nonzero', () => {
    const items = statusItems({ extracting: [], bookmarks: [], profileId: 1, stats: stats(3, 0) });
    expect(items).toEqual([{ kind: 'pending', count: 3 }]);
  });

  it('pending_counts_only_bookmarks_waiting_to_be_extracted', () => {
    // With the AI off, 5 extracted bookmarks stay pending until a model appears: not work in progress.
    const items = statusItems({
      extracting: [],
      bookmarks: [],
      profileId: 1,
      stats: stats(2, 0, 7),
    });
    expect(items).toEqual([{ kind: 'pending', count: 2 }]);
    const waiting = statusItems({
      extracting: [],
      bookmarks: [],
      profileId: 1,
      stats: stats(0, 0, 5),
    });
    expect(waiting).toEqual([]);
  });

  it('nothing_without_stats_or_work', () => {
    expect(statusItems({ extracting: [], bookmarks: [], profileId: 1, stats: null })).toEqual([]);
  });

  it('the_first_extracting_of_this_profile_wins', () => {
    const items = statusItems({
      extracting: [
        { profile: 2, id: 9 },
        { profile: 1, id: 4 },
        { profile: 1, id: 5 },
      ],
      bookmarks: [bookmark(4, 'Cuatro'), bookmark(5, 'Cinco')],
      profileId: 1,
      stats: null,
    });
    expect(items).toEqual([{ kind: 'extracting', id: 4, title: 'Cuatro' }]);
  });
});
