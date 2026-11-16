// Switching, creating, renaming and deleting profiles.
import { tick } from 'svelte';
import {
  createProfile,
  deleteProfile,
  moveBookmark,
  renameProfile,
  switchProfile,
} from '../../api';
import { lib, openSettings, refresh, refreshGuard, report } from '../library/libraryState.svelte';
import { isOnScreen, suggestInitials } from './profiles';
import { ALL } from '../library/sidebar';

/**
 * Marks a bookmark row's drag (data: `bookmarkDragData`). Anything else dragged over the rail carries
 * no such type and is refused.
 */
export const BOOKMARK_MIME = 'application/x-marks-bookmark';

/** The create/rename dialog: `id: null` creates a profile, else renames that one. */
export const profileDialog = $state({
  open: false,
  id: null as number | null,
  /** The fields, filled when the dialog opens. */
  name: '',
  initials: '',
  /** The initials follow the name until the user types in them. */
  initialsEdited: false,
  /** The backend's error on submit, shown in the dialog. */
  error: null as string | null,
});

/** The question of "Delete profile" in Settings > Profile; it is about the active profile. */
export const profileDelete = $state({ asking: false });

// The profile a switch in flight is heading to: `lib.profileId` only changes when its refresh lands.
let target: number | null = null;

/**
 * Makes `id` the active profile. Pending edits of the detail are sent first, to the profile they
 * were made in; the library then shows "All" with nothing selected. Settings stays open.
 */
export async function switchTo(id: number): Promise<void> {
  if (id === (target ?? lib.profileId)) return;
  target = id;
  leaveTextField();
  // Unmounting the detail flushes pending notes.
  lib.selectedId = null;
  await tick();
  refreshGuard.beginSwitch();
  try {
    await switchProfile(id);
    resetView();
  } catch (err) {
    report(err);
  } finally {
    refreshGuard.endSwitch();
  }
  await refresh();
  // A later switch owns `target` now.
  if (target === id) target = null;
}

// Leaving the title or notes field saves it. Only text fields lose focus, so a rail item used from
// the keyboard keeps it.
function leaveTextField(): void {
  const active = document.activeElement;
  if (active instanceof HTMLElement && active.matches('input, textarea, [contenteditable]')) {
    active.blur();
  }
}

// What the library shows on entering a profile; pending questions belonged to the one left.
function resetView(): void {
  lib.selection = ALL;
  lib.collapsed = {};
  lib.expanded = false;
  lib.confirmDelete = false;
  lib.groupDeleteId = null;
  lib.groupEdit = null;
  profileDelete.asking = false;
}

/**
 * The rail has no room for the question: switches to `id`, so the count shown is its own, and asks
 * in Settings > Profile.
 */
export async function askDeleteProfile(id: number): Promise<void> {
  await switchTo(id);
  // The switch failed, or a later one took over.
  if (lib.profileId !== id) return;
  openSettings('profile');
  profileDelete.asking = true;
}

/** Deletes the profile, its bookmarks and its token; the backend makes profile 1 the active one. */
export async function removeProfile(id: number): Promise<void> {
  profileDelete.asking = false;
  // It may be the active profile: to the refreshes in flight this is a switch.
  refreshGuard.beginSwitch();
  try {
    await deleteProfile(id);
    resetView();
  } catch (err) {
    report(err);
  } finally {
    refreshGuard.endSwitch();
  }
  await refresh();
}

/**
 * Moves bookmark `id` of profile `from` (the one it was shown in: ids repeat across profiles) to
 * `to`; the app stays on the active profile and the bookmark leaves the list. A switch that landed
 * since only means the list on screen is another profile's: the move still names its own bookmark.
 * The backend's refusal (same profile, URL already there) is reported.
 */
export async function moveBookmarkTo(id: number, from: number, to: number): Promise<void> {
  const moved = { id, profile: from };
  // Pending edits of the detail go first, so the copy carries them.
  if (isOnScreen(moved, lib)) {
    leaveTextField();
    await tick();
  }
  try {
    await moveBookmark(id, from, to);
  } catch (err) {
    report(err);
    return;
  }
  if (isOnScreen(moved, lib)) {
    lib.selectedId = null;
    lib.expanded = false;
  }
  await refresh();
}

/** Moves the selected bookmark, if any, of the profile on screen to profile `to`. */
export function moveSelectedTo(to: number): Promise<void> {
  return lib.selectedId === null
    ? Promise.resolve()
    : moveBookmarkTo(lib.selectedId, lib.profileId, to);
}

function openDialog(id: number | null): void {
  const profile = lib.profiles.find((p) => p.id === id);
  profileDialog.id = id;
  profileDialog.name = profile?.name ?? '';
  profileDialog.initials = profile?.initials ?? '';
  // A rename whose initials are not the suggested ones was already typed by hand.
  profileDialog.initialsEdited =
    profile !== undefined && profile.initials !== suggestInitials(profile.name);
  profileDialog.error = null;
  profileDialog.open = true;
}

export function openNewProfile(): void {
  openDialog(null);
}

export function openRenameProfile(id: number): void {
  openDialog(id);
}

export function closeProfileDialog(): void {
  profileDialog.open = false;
}

/** Creates the profile and switches to it, or renames it; `false` leaves the dialog open with the error. */
export async function submitProfile(name: string, initials: string): Promise<boolean> {
  const id = profileDialog.id;
  profileDialog.error = null;
  let created: number;
  try {
    if (id !== null) {
      await renameProfile(id, name, initials);
      closeProfileDialog();
      await refresh();
      return true;
    }
    created = (await createProfile(name, initials)).id;
  } catch (err) {
    profileDialog.error = err instanceof Error ? err.message : String(err);
    report(err, { toast: false });
    return false;
  }
  closeProfileDialog();
  await switchTo(created);
  return true;
}
