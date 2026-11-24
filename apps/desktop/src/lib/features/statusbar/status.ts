// What the status bar shows of the background work of the active profile, and of the AI and local API.
import type { AppStatus, Bookmark, Extracting, Stats } from '../../api';
import { t } from '../../strings';
import type { SettingsSection } from '../settings/settingsLogic';

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

export type SystemEntry =
  | {
      kind: 'ai';
      state: 'off' | 'ready' | 'failed';
      model: string | null;
      error: string | null;
    }
  | { kind: 'api'; state: 'listening' | 'busy' | 'stopped'; port: number };

/** The AI, then the local API; nothing before the first status, and no API while it is starting. */
export function systemItems(status: AppStatus | null): SystemEntry[] {
  if (!status) return [];
  const { ai, api, api_port: port } = status;
  const entries: SystemEntry[] = [
    {
      kind: 'ai',
      state: ai.kind,
      model: ai.kind === 'off' ? null : ai.model,
      error: ai.kind === 'failed' ? ai.error : null,
    },
  ];
  if (api !== 'starting') entries.push({ kind: 'api', state: api, port });
  return entries;
}

/** How an entry is drawn: its text and `title`, a decorative dot (none when off), `strong` when it fails, and where its click goes. */
export function systemView(item: SystemEntry): {
  text: string;
  title: string;
  dot: 'ready' | 'failed' | undefined;
  strong: boolean;
  section: SettingsSection;
} {
  const failing = { dot: 'failed' as const, strong: true };
  const fine = { dot: 'ready' as const, strong: false };
  if (item.kind === 'ai') {
    const section = 'ai';
    if (item.state === 'off') {
      const text = t.statusBar.ai.off;
      return { text, title: text, dot: undefined, strong: false, section };
    }
    if (item.state === 'ready') {
      const text = t.statusBar.ai.ready(item.model ?? '');
      return { text, title: text, ...fine, section };
    }
    const error = item.error ?? '';
    const title = item.model
      ? t.statusBar.ai.failedTitle(error)
      : t.statusBar.ai.unavailableTitle(error);
    return { text: t.statusBar.ai.failed, title, ...failing, section };
  }
  const section = 'extension';
  if (item.state === 'listening') {
    const title = t.statusBar.api.listeningTitle(item.port);
    return { text: t.statusBar.api.listening, title, ...fine, section };
  }
  const text = item.state === 'busy' ? t.statusBar.api.busy(item.port) : t.statusBar.api.stopped;
  return { text, title: text, ...failing, section };
}
