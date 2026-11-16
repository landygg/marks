// Native (macOS) context menus of the sidebar. They are the system's menus, so no design tokens apply.
import { CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu } from '@tauri-apps/api/menu';
import type { TagGroup } from '../../api';
import {
  askDeleteGroup,
  lib,
  moveTag,
  report,
  startNewGroup,
  startRenameGroup,
} from './libraryState.svelte';
import { t } from '../../strings';

/** Anything `close()` frees on the Rust side. */
interface Closable {
  close: () => Promise<void>;
}

// Menus and their items are resources held by the backend until closed. Only one context menu is
// alive at a time, so the previous one is freed when the next opens.
let alive: Closable[] = [];

export type Keep = <T extends Closable>(resource: T) => T;

const closeAll = (resources: Closable[]): Promise<unknown> =>
  Promise.all(resources.map((resource) => resource.close().catch(() => {})));

// A native menu is modal. Opening a second one while the first is building or on screen (possible
// through accessibility, not with the pointer) froze the app, so that request is dropped.
let showing = false;

/** Builds the menu with `build` (every resource it creates goes through `keep`) and pops it up. */
export async function show(build: (keep: Keep) => Promise<Menu>): Promise<void> {
  if (showing) return;
  showing = true;
  // Each call collects its own resources, freed when the next menu opens.
  const mine: Closable[] = [];
  const keep: Keep = (resource) => {
    mine.push(resource);
    return resource;
  };
  try {
    const menu = keep(await build(keep));
    const previous = alive;
    alive = mine;
    await closeAll(previous);
    await menu.popup();
  } catch (err) {
    void closeAll(mine);
    report(err);
  } finally {
    showing = false;
  }
}

// A switch can land while a menu is open: what it would open on screen belongs to the profile it
// was built for, and is dropped once another one is shown.
const inProfile = (profile: number, open: () => void) => () => {
  if (lib.profileId === profile) open();
};

/**
 * Menu for a tag of `profile` (the one shown when it opened): "Move to" (groups, the current one
 * checked, then "New group…") and "Remove from group".
 */
export function showTagMenu(
  tag: string,
  currentGroupId: number | null,
  groups: TagGroup[],
  profile: number,
): Promise<void> {
  return show(async (keep) => {
    const targets = await Promise.all(
      groups.map(async (group) =>
        keep(
          await CheckMenuItem.new({
            text: group.name,
            checked: group.id === currentGroupId,
            action: () => void moveTag(profile, tag, group.id),
          }),
        ),
      ),
    );
    const moveTo = keep(
      await Submenu.new({
        text: t.moveTo,
        items: [
          ...targets,
          keep(await PredefinedMenuItem.new({ item: 'Separator' })), // i18n-allow: identifier
          keep(
            await MenuItem.new({
              text: t.newGroupMenu,
              action: inProfile(profile, () => startNewGroup(tag)),
            }),
          ),
        ],
      }),
    );
    const remove = keep(
      await MenuItem.new({
        text: t.removeFromGroup,
        enabled: currentGroupId !== null,
        action: () => void moveTag(profile, tag, null),
      }),
    );
    return Menu.new({ items: [moveTo, remove] });
  });
}

/** Menu for a user group of `profile`: "Rename" and "Delete group" (which asks for confirmation in the sidebar). */
export function showGroupMenu(groupId: number, profile: number): Promise<void> {
  return show(async (keep) =>
    Menu.new({
      items: [
        keep(
          await MenuItem.new({
            text: t.renameGroup,
            action: inProfile(profile, () => startRenameGroup(groupId)),
          }),
        ),
        keep(
          await MenuItem.new({
            text: t.deleteGroup,
            action: inProfile(profile, () => askDeleteGroup(groupId)),
          }),
        ),
      ],
    }),
  );
}
