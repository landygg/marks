// Shared frontend state: the bookmark list, the tag groups, what is selected and which view is shown.
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import {
  EXTRACTING_CHANGED,
  LIBRARY_CHANGED,
  NOTIFICATIONS_CHANGED,
  OPEN_ADD_DIALOG,
  createTagGroup,
  deleteTagGroup,
  extractImported,
  extractWithAi,
  extracting as loadExtracting,
  get,
  getSettings,
  hasExtractionModel,
  list,
  openExternal,
  profiles as loadProfiles,
  reextract,
  remove,
  renameTagGroup,
  retry,
  setTagGroup,
  stats,
  tagGroups,
  update,
  type Bookmark,
  type BookmarkPatch,
  type Extracting,
  type Profile,
  type Stats,
  type TagGroup,
  type Target,
} from '../../api';
import { notify, refreshNotifications } from '../notifications/notificationsState.svelte';
import { t } from '../../strings';
import { coalesce, createRefreshGuard, keepUnchanged, kept } from './refreshGuard';
import { isRemoteEndpoint, type SettingsSection } from '../settings/settingsLogic';
import { ALL, matches, validSelection, type Selection } from './sidebar';

/** Key of the automatic "Categories" group in `lib.collapsed`. */
export const CATEGORIES_KEY = 'categories';
/** Key of the trailing group of ungrouped tags in `lib.collapsed`. */
export const UNGROUPED_KEY = 'none';

const sortedUnique = (values: string[]): string[] =>
  [...new Set(values)].sort((a, b) => a.localeCompare(b));

// A class, because `$state({...})` cannot hold `$derived` properties.
class Library {
  bookmarks = $state<Bookmark[]>([]);
  groups = $state<TagGroup[]>([]);
  profiles = $state<Profile[]>([]);
  /** The active profile; `bookmarks`, `groups` and `stats` belong to it. */
  profileId = $state(1);
  selection = $state<Selection>(ALL);
  /** Collapsed sidebar groups, by `CATEGORIES_KEY`, `UNGROUPED_KEY` or the group id. */
  collapsed = $state<Record<string, boolean>>({});
  selectedId = $state<number | null>(null);
  view = $state<'library' | 'settings'>('library');
  /** The section Settings shows; it stays where it was left for the session. */
  settingsSection = $state<SettingsSection>('appearance');
  /** The detail covers the list. */
  expanded = $state(false);
  /**
   * Inline group-name field in the sidebar: `id: null` creates a group (and moves `tag` into it), else
   * renames. `profile`: the one it was opened in, where the submit goes (group ids repeat across profiles).
   */
  groupEdit = $state<{ profile: number; id: number | null; tag: string | null } | null>(null);
  /** User group whose deletion is waiting for its inline confirmation. */
  groupDeleteId = $state<number | null>(null);
  /** The selected bookmark's deletion is waiting for its inline confirmation. */
  confirmDelete = $state(false);
  /** A re-extraction of the selected bookmark is waiting for its inline confirmation. */
  confirmReextract = $state<ReextractKind | null>(null);
  /**
   * Queuing the imported bookmarks of the view is waiting for its confirmation above the list.
   * `profile` and `ids`: those it was asked for. `remote`: their pages will reach a remote AI provider.
   */
  confirmExtractView = $state<{ profile: number; ids: number[]; remote: boolean } | null>(null);
  /** Whether "Extract with AI" can run: the extraction model was built. */
  extractionModel = $state(false);
  /** Bumped on every `select`, so the list scrolls to the row even when it was already selected. */
  revealCount = $state(0);
  /** Bookmarks being extracted now, across profiles. */
  extracting = $state<Extracting[]>([]);
  stats = $state<Stats | null>(null);
  addDialogOpen = $state(false);

  /** The bookmarks of the current selection. */
  visible = $derived(this.bookmarks.filter((bookmark) => matches(this.selection, bookmark)));
  categories = $derived(
    sortedUnique(this.bookmarks.flatMap((b) => (b.category ? [b.category] : []))),
  );
  tags = $derived(sortedUnique(this.bookmarks.flatMap((b) => b.tags)));
}

/** "Extract again" (Defuddle) or "Extract with AI" (the extraction model). */
export type ReextractKind = 'defuddle' | 'ai';

export const lib = new Library();

/**
 * Records a failure in the notifications. `toast` (the default) for the user's own actions; off for
 * background work and for a form that already shows the error in place. `profile`: the active one
 * by default, `null` for a global failure (AI, theme).
 */
export function report(
  err: unknown,
  opts: { toast?: boolean; profile?: number | null; target?: Target } = {},
): void {
  void notify(
    {
      profile: opts.profile === undefined ? lib.profileId : opts.profile,
      severity: 'error',
      // Tauri rejects with the command's error string; anything else is a JS Error.
      message: err instanceof Error ? err.message : String(err),
      detail: null,
      target: opts.target ?? null,
    },
    { toast: opts.toast ?? true },
  );
}

/** Discards refreshes whose four calls may straddle a profile switch (see `profilesState.svelte.ts`). */
export const refreshGuard = createRefreshGuard();

/**
 * Reloads bookmarks, groups, stats and profiles; drops a selected bookmark or a selection that no
 * longer exists, and everything that pointed into the previous profile when the active one changed.
 * One run at a time (`coalesce`): calls made while it runs share one more run.
 */
export const refresh = coalesce(async () => {
  const ticket = refreshGuard.start();
  try {
    const [bookmarks, current, groups, view, extractionModel] = await Promise.all([
      list(),
      stats(),
      tagGroups(),
      loadProfiles(),
      hasExtractionModel(),
    ]);
    // A profile switch started meanwhile: the refresh it asks for brings consistent data.
    if (!refreshGuard.current(ticket)) return;
    const previous = lib.profileId;
    lib.profiles = view.profiles;
    lib.profileId = view.active;
    // Under its own guard: a refresh started earlier must not bring back an older list.
    void refreshNotifications();
    // The worker refreshes once per processed bookmark: only rows that changed are drawn again.
    lib.bookmarks = keepUnchanged(lib.bookmarks, bookmarks);
    lib.stats = current;
    lib.groups = groups;
    lib.extractionModel = extractionModel;
    const ids = (rows: { id: number }[]) => rows.map((row) => row.id);
    lib.selectedId = kept(lib.selectedId, ids(bookmarks), previous, view.active);
    // Pending confirmations die with their target.
    if (lib.selectedId === null) {
      lib.confirmDelete = false;
      lib.confirmReextract = null;
    }
    lib.groupDeleteId = kept(lib.groupDeleteId, ids(groups), previous, view.active);
    if (previous !== view.active) {
      lib.groupEdit = null;
      lib.confirmExtractView = null;
      lib.expanded = false;
    }
    // Otherwise the list would stay empty with nothing in the sidebar to click back from.
    lib.selection = validSelection(lib.selection, lib.categories, lib.tags);
  } catch (err) {
    // A discarded refresh's error belongs to data nobody will see; the next one reports its own.
    if (refreshGuard.current(ticket)) report(err);
  }
});

const extractingGuard = createRefreshGuard();

/** Reloads the list of bookmarks being extracted; calls made while it runs share one more run. */
export const refreshExtracting = coalesce(async () => {
  const ticket = extractingGuard.start();
  try {
    const list = await loadExtracting();
    if (extractingGuard.current(ticket)) lib.extracting = list;
  } catch (err) {
    if (extractingGuard.current(ticket)) report(err, { toast: false });
  }
});

export function select(id: number): void {
  lib.selectedId = id;
  lib.revealCount += 1;
  // A pending confirmation belongs to the bookmark it was asked for.
  lib.confirmDelete = false;
  lib.confirmReextract = null;
}

/** Asks to confirm deleting the selected bookmark (the detail shows the question). */
export function askDeleteBookmark(): void {
  if (lib.selectedId === null) return;
  lib.view = 'library';
  // One question at a time under the detail's strip.
  lib.confirmReextract = null;
  lib.confirmDelete = true;
}

/** Shows the bookmarks of `selection` in the list, leaving Settings and the expanded detail. */
export function choose(selection: Selection): void {
  lib.selection = selection;
  lib.view = 'library';
  lib.expanded = false;
  // The question holds the ids of the view it was asked in.
  lib.confirmExtractView = null;
}

export function toggleExpanded(): void {
  lib.expanded = !lib.expanded;
}

export function toggleCollapsed(key: string): void {
  lib.collapsed[key] = !lib.collapsed[key];
}

export function toggleSettings(): void {
  lib.view = lib.view === 'settings' ? 'library' : 'settings';
}

/** Shows Settings, in `section` or where it was last. */
export function openSettings(section: SettingsSection = lib.settingsSection): void {
  lib.settingsSection = section;
  lib.view = 'settings';
}

/** Back to the library as it was left: same selection, same bookmark. */
export function closeSettings(): void {
  lib.view = 'library';
}

/** Puts the backend's copy of a bookmark in place of the one in the list and drops a selection that no longer exists. */
export function replaceBookmark(updated: Bookmark): void {
  const i = lib.bookmarks.findIndex((b) => b.id === updated.id);
  if (i !== -1) lib.bookmarks[i] = updated;
  // Removing a tag from its last bookmark makes a selected tag disappear from the sidebar.
  lib.selection = validSelection(lib.selection, lib.categories, lib.tags);
}

// Writes go to the profile the bookmark was loaded in. A response that arrives after a profile
// switch leaves the list alone: the same id may be another bookmark in the new profile.

/** Saves the patch and updates the list with the result; `false` when it failed (already reported). */
export async function saveBookmark(
  profile: number,
  id: number,
  patch: BookmarkPatch,
): Promise<boolean> {
  try {
    const updated = await update(profile, id, patch);
    if (lib.profileId === profile) replaceBookmark(updated);
    return true;
  } catch (err) {
    report(err);
    return false;
  }
}

/** Queues the bookmark for processing again and shows it as pending. */
export async function retryBookmark(profile: number, id: number): Promise<void> {
  try {
    await retry(profile, id);
    if (lib.profileId === profile) await refresh();
  } catch (err) {
    report(err);
  }
}

/**
 * Re-extracts the selected bookmark; one that already has content asks first (the detail shows
 * the question), since its content is replaced.
 */
export async function askReextract(kind: ReextractKind): Promise<void> {
  const id = lib.selectedId;
  const profile = lib.profileId;
  if (id === null) return;
  lib.view = 'library';
  // One question at a time under the detail's strip.
  lib.confirmDelete = false;
  try {
    const full = await get(profile, id);
    // The selection moved on while the content was being read.
    if (lib.selectedId !== id || lib.profileId !== profile) return;
    if (full?.content?.trim()) lib.confirmReextract = kind;
    else await runReextract(profile, id, kind);
  } catch (err) {
    report(err);
  }
}

/** Runs the re-extraction; the content changes only if the result is not thin, else the error shows. */
export async function runReextract(
  profile: number,
  id: number,
  kind: ReextractKind,
): Promise<void> {
  lib.confirmReextract = null;
  const target: Target = { kind: 'bookmark', id };
  // Taken before the call: afterwards the active profile may be another one, where `id` is not this bookmark.
  const title = lib.profileId === profile ? lib.bookmarks.find((b) => b.id === id)?.title : null;
  try {
    await (kind === 'ai' ? extractWithAi : reextract)(profile, id);
    if (lib.profileId === profile) await refresh();
  } catch (err) {
    report(err, { profile, target });
    return;
  }
  void notify(
    {
      profile,
      severity: 'info',
      message: t.notices.reextracted[kind],
      detail: title ?? null,
      target,
    },
    { toast: true },
  );
}

/** Asks to confirm queuing the imported bookmarks of the view (the list shows the question). */
export async function askExtractView(): Promise<void> {
  const profile = lib.profileId;
  const ids = lib.visible.filter((b) => b.status === 'imported').map((b) => b.id);
  if (ids.length === 0) return;
  lib.view = 'library';
  try {
    const { settings } = await getSettings();
    // The question belongs to the view of the profile it was asked in.
    if (lib.profileId !== profile) return;
    const remote = settings.enabled && isRemoteEndpoint(settings.provider, settings.endpoint);
    lib.confirmExtractView = { profile, ids, remote };
  } catch (err) {
    // The AI settings are global.
    report(err, { profile: null });
  }
}

/** Queues the bookmarks the confirmation was asked for, in the profile it was asked in. */
export async function runExtractView(): Promise<void> {
  const asked = lib.confirmExtractView;
  if (!asked) return;
  lib.confirmExtractView = null;
  const { profile, ids } = asked;
  try {
    const queued = await extractImported(profile, ids);
    void notify(
      {
        profile,
        severity: 'info',
        message: t.notices.extractQueued(queued),
        detail: null,
        target: null,
      },
      { toast: true },
    );
    if (lib.profileId === profile) await refresh();
  } catch (err) {
    report(err, { profile });
  }
}

/** Removes the bookmark; unsaved notes of a deleted bookmark are discarded. */
export async function deleteBookmark(profile: number, id: number): Promise<void> {
  try {
    await remove(profile, id);
    if (lib.profileId !== profile) return;
    lib.expanded = false;
    lib.confirmDelete = false;
    await refresh();
  } catch (err) {
    report(err);
  }
}

/** Opens the inline field for a new group; `tag` is moved into it once created. */
export function startNewGroup(tag: string | null = null): void {
  lib.view = 'library';
  lib.groupDeleteId = null;
  lib.groupEdit = { profile: lib.profileId, id: null, tag };
}

export function startRenameGroup(id: number): void {
  lib.view = 'library';
  lib.groupDeleteId = null;
  lib.groupEdit = { profile: lib.profileId, id, tag: null };
}

export function cancelGroupEdit(): void {
  lib.groupEdit = null;
}

let submittingGroup = false;

/**
 * Creates or renames the group being edited: `"ok"`, `"failed"` (already reported; the field stays
 * open) or `"busy"` when another submit is still in flight and this one was ignored.
 */
export async function submitGroupEdit(name: string): Promise<'ok' | 'failed' | 'busy'> {
  const edit = lib.groupEdit;
  if (!edit || submittingGroup) return 'busy';
  submittingGroup = true;
  try {
    if (edit.id === null) {
      const created = await createTagGroup(edit.profile, name);
      // The group exists from here on: close the field even if moving the tag fails. Only this
      // edit's field: another one may have opened while the request was running.
      if (lib.groupEdit === edit) lib.groupEdit = null;
      if (edit.tag !== null) await setTagGroup(edit.profile, edit.tag, created.id);
    } else {
      await renameTagGroup(edit.profile, edit.id, name);
      if (lib.groupEdit === edit) lib.groupEdit = null;
    }
    return 'ok';
  } catch (err) {
    report(err);
    return 'failed';
  } finally {
    await refresh();
    submittingGroup = false;
  }
}

/** Asks to confirm deleting the group (the sidebar shows the question under it). */
export function askDeleteGroup(id: number): void {
  lib.view = 'library';
  lib.groupEdit = null;
  lib.groupDeleteId = id;
}

export function cancelDeleteGroup(): void {
  lib.groupDeleteId = null;
}

// Group writes, like bookmark writes, go to the profile they were started in and leave the
// screen alone when a switch landed meanwhile.

/** Removes group `id` of `profile`; its tags become ungrouped. */
export async function deleteGroup(profile: number, id: number): Promise<void> {
  lib.groupDeleteId = null;
  try {
    await deleteTagGroup(profile, id);
    if (lib.profileId !== profile) return;
    // SQLite reuses the id of the last deleted row: a new group must not inherit this one's state.
    delete lib.collapsed[String(id)];
    await refresh();
  } catch (err) {
    report(err);
  }
}

/** Moves the tag of `profile` into the group; `null` takes it out of its group. */
export async function moveTag(profile: number, tag: string, groupId: number | null): Promise<void> {
  try {
    await setTagGroup(profile, tag, groupId);
    if (lib.profileId === profile) await refresh();
  } catch (err) {
    report(err);
  }
}

export function openLink(url: string): void {
  openExternal(url).catch(report);
}

export function openAddDialog(): void {
  lib.addDialogOpen = true;
}

export function closeAddDialog(): void {
  lib.addDialogOpen = false;
}

// A failed subscription is reported and yields a no-op unsubscribe.
const subscribe = (event: string, handler: () => void): Promise<UnlistenFn> =>
  listen(event, handler).catch((err: unknown) => {
    report(err, { toast: false });
    return () => {};
  });

/** Keeps the list in sync with backend changes and opens the add dialog from the tray; resolves to one unsubscribe function. */
export async function startListening(): Promise<UnlistenFn> {
  const stops = await Promise.all([
    subscribe(LIBRARY_CHANGED, () => void refresh()),
    subscribe(OPEN_ADD_DIALOG, openAddDialog),
    subscribe(EXTRACTING_CHANGED, () => void refreshExtracting()),
    subscribe(NOTIFICATIONS_CHANGED, () => void refreshNotifications()),
  ]);
  void refreshExtracting();
  return () => stops.forEach((stop) => stop());
}
