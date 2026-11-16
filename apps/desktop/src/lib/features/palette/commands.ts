// The command palette's list of commands: every action of the app, by name.
import type { LocaleInfo, Profile, TagGroup } from '../../api';
import type { IconName } from '../../ui/icons';
import type { SettingsSection } from '../settings/settingsLogic';
import { t } from '../../strings';
import { PROFILE_SHORTCUT_COUNT } from '../../shortcuts';
import { THEMES, type Theme } from '../settings/theme';

/** The language settings, in the order the select and the palette list them. */
export const LANGUAGES: readonly LocaleInfo['setting'][] = ['system', 'es', 'en'];

export type PaletteMode = 'commands' | 'bookmarks' | 'tags';

export interface Command {
  id: string;
  label: string;
  icon: IconName;
  /** Shown at the right of the row; the key handling itself lives in shortcuts.ts. */
  shortcut?: string;
  run: () => void;
}

export interface CommandActions {
  add: () => void;
  searchBookmarks: () => void;
  newGroup: () => void;
  toggleExpanded: () => void;
  toggleSidebar: () => void;
  /** Without a section, Settings opens where it was last. */
  openSettings: (section?: SettingsSection) => void;
  openInBrowser: () => void;
  retry: () => void;
  /** Re-extracts the selected bookmark with Defuddle (asking first if it has content). */
  reextract: () => void;
  /** Re-extracts the selected bookmark with the extraction model (asking first if it has content). */
  extractWithAi: () => void;
  deleteBookmark: () => void;
  renameGroup: (id: number) => void;
  deleteGroup: (id: number) => void;
  moveTag: (tag: string, groupId: number | null) => void;
  setTheme: (theme: Theme) => void;
  newProfile: () => void;
  switchProfile: (id: number) => void;
  /** Moves the selected bookmark to profile `id`. */
  moveToProfile: (id: number) => void;
  /** Opens the notification panel. */
  notifications: () => void;
  /** Empties the notifications of the active profile and the global ones. */
  clearNotifications: () => void;
  /** Asks to queue the imported bookmarks of the current view. */
  extractView: () => void;
  /** Asks to relaunch marks in this language setting. */
  setLanguage: (setting: LocaleInfo['setting']) => void;
}

/** What the commands can act on right now. */
export interface CommandContext {
  /**
   * The selected bookmark, if any. A blocked one is rescued from the extension, and one being
   * extracted (`busy`) waits: neither is offered a re-extraction, as in the detail. An `imported`
   * one has nothing to re-extract yet: it is offered "Extract".
   */
  bookmark: CommandBookmark | null;
  groups: TagGroup[];
  /** The tag selected in the sidebar, if any, and the group it is in. */
  tag: { name: string; groupId: number | null } | null;
  profiles: Profile[];
  activeProfile: number;
  /** Whether "Extract with AI" can run: there is an extraction model. */
  extractionModel: boolean;
  /** The panel has something to clear. */
  hasNotifications: boolean;
  /** How many bookmarks of the current view are `imported`. */
  importedInView: number;
  /** The saved language setting, left out of the language commands. */
  language: LocaleInfo['setting'];
}

type CommandBookmark = { failed: boolean; blocked: boolean; busy: boolean; imported: boolean };

/**
 * Global commands, then the profile ones, those of the selected bookmark, of each group and of the selected tag.
 * The "Settings: …" commands open their section of Settings: their confirmation and result live there.
 */
export function commands(actions: CommandActions, context: CommandContext): Command[] {
  const {
    bookmark,
    groups,
    tag,
    profiles,
    activeProfile,
    extractionModel,
    hasNotifications,
    importedInView,
    language,
  } = context;
  return [
    { id: 'add', label: t.cmdAdd, icon: 'plus', shortcut: t.shortcutAdd, run: actions.add },
    {
      id: 'search',
      label: t.cmdSearch,
      icon: 'search',
      shortcut: t.shortcutSearch,
      run: actions.searchBookmarks,
    },
    { id: 'new-group', label: t.cmdNewGroup, icon: 'folderPlus', run: actions.newGroup },
    ...(importedInView > 0
      ? [
          {
            id: 'extract-view',
            label: t.cmdExtractView(importedInView),
            icon: 'reextract',
            run: actions.extractView,
          } as const,
        ]
      : []),
    {
      id: 'toggle-expanded',
      label: t.cmdToggleExpanded,
      icon: 'expand',
      run: actions.toggleExpanded,
    },
    {
      id: 'toggle-sidebar',
      label: t.cmdToggleSidebar,
      icon: 'sidebar',
      shortcut: t.shortcutSidebar,
      run: actions.toggleSidebar,
    },
    {
      id: 'reindex',
      label: t.cmdReindex,
      icon: 'gear',
      run: () => actions.openSettings('maintenance'),
    },
    {
      id: 'process-unsummarized',
      label: t.cmdProcess,
      icon: 'gear',
      run: () => actions.openSettings('maintenance'),
    },
    {
      id: 'test-connection',
      label: t.cmdTestConnection,
      icon: 'gear',
      run: () => actions.openSettings('ai'),
    },
    {
      id: 'settings',
      label: t.cmdSettings,
      icon: 'gear',
      shortcut: t.shortcutSettings,
      run: () => actions.openSettings(),
    },
    { id: 'notifications', label: t.notifications, icon: 'bell', run: actions.notifications },
    ...(hasNotifications
      ? [
          {
            id: 'clear-notifications',
            label: t.notices.clearCommand,
            icon: 'trash',
            run: actions.clearNotifications,
          } as const,
        ]
      : []),
    { id: 'new-profile', label: t.newProfile, icon: 'plus', run: actions.newProfile },
    {
      id: 'settings-profile',
      label: t.cmdSettingsProfile,
      icon: 'gear',
      run: () => actions.openSettings('profile'),
    },
    { id: 'import', label: t.cmdImport, icon: 'gear', run: () => actions.openSettings('transfer') },
    { id: 'export', label: t.cmdExport, icon: 'gear', run: () => actions.openSettings('transfer') },
    // The shortcut is the profile's position in the rail, so it is taken before dropping the active one.
    ...profiles.flatMap((profile, i): Command[] =>
      profile.id === activeProfile
        ? []
        : [
            {
              id: `switch-profile-${profile.id}`,
              label: t.cmdSwitchProfile(profile.name),
              icon: 'gear',
              shortcut: i < PROFILE_SHORTCUT_COUNT ? t.profileShortcut(i) : undefined,
              run: () => actions.switchProfile(profile.id),
            },
          ],
    ),
    ...THEMES.map((theme): Command => ({
      id: `theme-${theme}`,
      label: t.cmdTheme(theme),
      icon: 'gear',
      run: () => actions.setTheme(theme),
    })),
    ...LANGUAGES.filter((setting) => setting !== language).map((setting): Command => ({
      id: `language:${setting}`,
      label: t.cmdLanguage(setting),
      icon: 'gear',
      run: () => actions.setLanguage(setting),
    })),
    ...(bookmark ? bookmarkCommands(actions, bookmark, extractionModel) : []),
    ...(bookmark
      ? profiles
          .filter((profile) => profile.id !== activeProfile)
          .map((profile): Command => ({
            id: `move-to-profile-${profile.id}`,
            label: t.cmdMoveToProfile(profile.name),
            icon: 'grip',
            run: () => actions.moveToProfile(profile.id),
          }))
      : []),
    ...groups.flatMap((group): Command[] => [
      {
        id: `rename-group-${group.id}`,
        label: t.cmdRenameGroup(group.name),
        icon: 'folderPlus',
        run: () => actions.renameGroup(group.id),
      },
      {
        id: `delete-group-${group.id}`,
        label: t.cmdDeleteGroup(group.name),
        icon: 'trash',
        run: () => actions.deleteGroup(group.id),
      },
    ]),
    ...(tag ? tagCommands(actions, tag, groups) : []),
  ];
}

function bookmarkCommands(
  actions: CommandActions,
  bookmark: CommandBookmark,
  extractionModel: boolean,
): Command[] {
  const reextract: Command[] =
    bookmark.blocked || bookmark.busy || bookmark.imported
      ? []
      : [
          { id: 'reextract', label: t.reextract, icon: 'reextract', run: actions.reextract },
          ...(extractionModel
            ? [
                {
                  id: 'extract-ai',
                  label: t.extractWithAi,
                  icon: 'sparkles',
                  run: actions.extractWithAi,
                } as const,
              ]
            : []),
        ];
  return [
    { id: 'open-link', label: t.cmdOpenLink, icon: 'external', run: actions.openInBrowser },
    ...(bookmark.failed
      ? [{ id: 'retry', label: t.cmdRetry, icon: 'retry', run: actions.retry } as const]
      : []),
    // Queuing an imported bookmark is what retry does to a failed one.
    ...(bookmark.imported
      ? [{ id: 'extract', label: t.cmdExtract, icon: 'reextract', run: actions.retry } as const]
      : []),
    ...reextract,
    { id: 'delete', label: t.cmdDelete, icon: 'trash', run: actions.deleteBookmark },
  ];
}

function tagCommands(
  actions: CommandActions,
  tag: { name: string; groupId: number | null },
  groups: TagGroup[],
): Command[] {
  const moves = groups
    .filter((group) => group.id !== tag.groupId)
    .map((group): Command => ({
      id: `move-tag-${group.id}`,
      label: t.cmdMoveTag(tag.name, group.name),
      icon: 'grip',
      run: () => actions.moveTag(tag.name, group.id),
    }));
  const ungroup: Command = {
    id: 'ungroup-tag',
    label: t.cmdUngroupTag(tag.name),
    icon: 'grip',
    run: () => actions.moveTag(tag.name, null),
  };
  return tag.groupId === null ? moves : [...moves, ungroup];
}

/** Commands whose label contains every word of `query`, case-insensitively; all of them for a blank query. */
export function filterCommands(all: Command[], query: string): Command[] {
  const wanted = query.toLowerCase().split(/\s+/).filter(Boolean);
  return all.filter((command) => {
    const label = command.label.toLowerCase();
    return wanted.every((word) => label.includes(word));
  });
}
