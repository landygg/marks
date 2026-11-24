import { describe, expect, it } from 'vitest';
import type { AiStatus, ApiStatus, Bookmark, Stats } from '../../api';
import { t } from '../../strings';
import { statusItems, systemItems, systemView } from './status';

/** Any port: the real one comes from Rust (`app_status`), and these only check how it is passed on. */
const PORT = 1234;

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

describe('systemItems', () => {
  const status = (ai: AiStatus, api: ApiStatus = 'listening') => ({ ai, api, api_port: PORT });

  it('nothing_before_the_first_status', () => {
    expect(systemItems(null)).toEqual([]);
  });

  it('ai_off_ready_and_failed', () => {
    const ai = (s: AiStatus) => systemItems(status(s, 'starting'))[0];
    expect(ai({ kind: 'off' })).toEqual({ kind: 'ai', state: 'off', model: null, error: null });
    expect(ai({ kind: 'ready', model: 'qwen3:8b' })).toEqual({
      kind: 'ai',
      state: 'ready',
      model: 'qwen3:8b',
      error: null,
    });
    expect(ai({ kind: 'failed', model: 'qwen3:8b', error: 'down' })).toEqual({
      kind: 'ai',
      state: 'failed',
      model: 'qwen3:8b',
      error: 'down',
    });
    expect(ai({ kind: 'failed', model: null, error: 'no key' })).toEqual({
      kind: 'ai',
      state: 'failed',
      model: null,
      error: 'no key',
    });
  });

  it('api_starting_shows_nothing', () => {
    expect(systemItems(status({ kind: 'off' }, 'starting')).map((i) => i.kind)).toEqual(['ai']);
  });

  it('api_listening_and_busy_carry_the_port', () => {
    for (const api of ['listening', 'busy', 'stopped'] as const) {
      expect(systemItems(status({ kind: 'off' }, api))[1]).toEqual({
        kind: 'api',
        state: api,
        port: PORT,
      });
    }
  });

  it('ai_comes_before_api', () => {
    expect(systemItems(status({ kind: 'off' })).map((i) => i.kind)).toEqual(['ai', 'api']);
  });
});

describe('systemView', () => {
  it('failing_entries_are_strong_and_name_their_error', () => {
    const ai = systemView({ kind: 'ai', state: 'failed', model: 'm', error: 'down' });
    expect(ai).toEqual({
      text: t.statusBar.ai.failed,
      title: t.statusBar.ai.failedTitle('down'),
      dot: 'failed',
      strong: true,
      section: 'ai',
    });
    const unbuilt = systemView({ kind: 'ai', state: 'failed', model: null, error: 'no key' });
    expect(unbuilt.title).toBe(t.statusBar.ai.unavailableTitle('no key'));
    const busy = systemView({ kind: 'api', state: 'busy', port: PORT });
    expect(busy).toMatchObject({
      text: t.statusBar.api.busy(PORT),
      strong: true,
      section: 'extension',
    });
  });

  it('off_has_no_dot_and_listening_names_the_address', () => {
    expect(systemView({ kind: 'ai', state: 'off', model: null, error: null }).dot).toBeUndefined();
    const api = systemView({ kind: 'api', state: 'listening', port: PORT });
    expect(api).toMatchObject({
      text: t.statusBar.api.listening,
      title: t.statusBar.api.listeningTitle(PORT),
      dot: 'ready',
      strong: false,
    });
  });
});
