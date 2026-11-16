// Native context menus of a profile in the rail and of a bookmark in the list.
import { Menu, MenuItem, Submenu } from '@tauri-apps/api/menu';
import type { Profile } from '../../api';
import { DEFAULT_PROFILE_ID } from './profiles';
import { askDeleteProfile, moveBookmarkTo, openRenameProfile } from './profilesState.svelte';
import { t } from '../../strings';
import { show } from '../library/tagMenu';

/** Menu for a profile: "Rename" and, except on profile 1, "Delete profile…". */
export function showProfileMenu(profile: Profile): Promise<void> {
  return show(async (keep) => {
    const items = [
      keep(
        await MenuItem.new({ text: t.renameProfile, action: () => openRenameProfile(profile.id) }),
      ),
    ];
    if (profile.id !== DEFAULT_PROFILE_ID) {
      items.push(
        keep(
          await MenuItem.new({
            text: t.deleteProfileMenu,
            action: () => void askDeleteProfile(profile.id),
          }),
        ),
      );
    }
    return Menu.new({ items });
  });
}

/**
 * Menu for bookmark `bookmarkId` of profile `active` (the one its row was shown in): "Move to profile",
 * every other profile. Not shown with a single profile.
 */
export function showBookmarkMenu(
  bookmarkId: number,
  profiles: Profile[],
  active: number,
): Promise<void> {
  const targets = profiles.filter((profile) => profile.id !== active);
  if (targets.length === 0) return Promise.resolve();
  return show(async (keep) => {
    const items = await Promise.all(
      targets.map(async (profile) =>
        keep(
          await MenuItem.new({
            text: profile.name,
            action: () => void moveBookmarkTo(bookmarkId, active, profile.id),
          }),
        ),
      ),
    );
    const moveTo = keep(await Submenu.new({ text: t.moveToProfile, items }));
    return Menu.new({ items: [moveTo] });
  });
}
