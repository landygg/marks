import { describe, expect, it } from 'vitest';
import { commands, filterCommands, type CommandActions, type CommandContext } from './commands';
import { t } from '../../strings';
import { setLocale } from '../../i18n';

const noop = (): void => {};
const noopActions: CommandActions = {
  add: noop,
  searchBookmarks: noop,
  newGroup: noop,
  toggleExpanded: noop,
  toggleSidebar: noop,
  openSettings: noop,
  openInBrowser: noop,
  retry: noop,
  reextract: noop,
  extractWithAi: noop,
  deleteBookmark: noop,
  renameGroup: noop,
  deleteGroup: noop,
  moveTag: noop,
  setTheme: noop,
  newProfile: noop,
  switchProfile: noop,
  moveToProfile: noop,
  notifications: noop,
  clearNotifications: noop,
  extractView: noop,
  setLanguage: noop,
};
const empty: CommandContext = {
  bookmark: null,
  groups: [],
  tag: null,
  profiles: [],
  activeProfile: 1,
  extractionModel: false,
  hasNotifications: false,
  importedInView: 0,
  language: 'es',
};
const ids = (context: CommandContext): string[] => commands(noopActions, context).map((c) => c.id);

describe('commands', () => {
  it('offers the languages except the current one', () => {
    expect(ids({ ...empty, language: 'es' }).filter((id) => id.startsWith('language:'))).toEqual([
      'language:system',
      'language:en',
    ]);
    expect(
      ids({ ...empty, language: 'system' }).filter((id) => id.startsWith('language:')),
    ).toEqual(['language:es', 'language:en']);
  });

  it('runs the language command with its setting', () => {
    const asked: string[] = [];
    const actions = { ...noopActions, setLanguage: (s: string) => asked.push(s) };
    commands(actions, empty)
      .find((c) => c.id === 'language:en')
      ?.run();
    expect(asked).toEqual(['en']);
  });

  it('keeps the theme label whole per language', () => {
    setLocale('en');
    expect(t.cmdTheme('dark')).toBe('Theme: dark');
    setLocale('es');
  });

  it('notifications_command_always_present', () => {
    const command = commands(noopActions, empty).find((c) => c.id === 'notifications');
    expect(command?.icon).toBe('bell');
    expect(command?.label).toBe(t.notifications);
  });

  it('clear_notifications_only_with_notifications', () => {
    expect(ids(empty)).not.toContain('clear-notifications');
    const cleared: string[] = [];
    const actions = { ...noopActions, clearNotifications: () => cleared.push('cleared') };
    const command = commands(actions, { ...empty, hasNotifications: true }).find(
      (c) => c.id === 'clear-notifications',
    );
    expect(command?.icon).toBe('trash');
    expect(command?.label).toBe(t.notices.clearCommand);
    command?.run();
    expect(cleared).toEqual(['cleared']);
  });

  it('offers toggle-sidebar with its shortcut', () => {
    const calls: string[] = [];
    const command = commands(
      { ...noopActions, toggleSidebar: () => calls.push('toggled') },
      empty,
    ).find((c) => c.id === 'toggle-sidebar');
    expect(command?.shortcut).toBe(t.shortcutSidebar);
    command?.run();
    expect(calls).toEqual(['toggled']);
  });

  it('lists the global actions when nothing is selected', () => {
    expect(ids(empty)).toEqual([
      'add',
      'search',
      'new-group',
      'toggle-expanded',
      'toggle-sidebar',
      'reindex',
      'process-unsummarized',
      'test-connection',
      'settings',
      'notifications',
      'new-profile',
      'settings-profile',
      'import',
      'export',
      'theme-system',
      'theme-light',
      'theme-dark',
      'language:system',
      'language:en',
    ]);
  });

  it('opens each settings command in the section that holds its confirmation and result', () => {
    const opened: unknown[] = [];
    const actions = { ...noopActions, openSettings: (section?: string) => opened.push(section) };
    const run = (id: string): void =>
      commands(actions, empty)
        .find((c) => c.id === id)
        ?.run();
    for (const id of [
      'reindex',
      'process-unsummarized',
      'test-connection',
      'settings',
      'settings-profile',
      'import',
      'export',
    ])
      run(id);
    expect(opened).toEqual([
      'maintenance',
      'maintenance',
      'ai',
      undefined,
      'profile',
      'transfer',
      'transfer',
    ]);
  });

  it('labels import and export as their Settings buttons', () => {
    const all = commands(noopActions, empty);
    expect(all.find((c) => c.id === 'import')?.label).toBe(t.cmdImport);
    expect(all.find((c) => c.id === 'export')?.label).toBe(t.cmdExport);
  });

  it('adds the actions of the selected bookmark, with retry only when it failed', () => {
    expect(
      ids({ ...empty, bookmark: { failed: false, blocked: false, busy: false, imported: false } }),
    ).toEqual(expect.arrayContaining(['open-link', 'delete']));
    expect(
      ids({ ...empty, bookmark: { failed: false, blocked: false, busy: false, imported: false } }),
    ).not.toContain('retry');
    expect(
      ids({ ...empty, bookmark: { failed: true, blocked: false, busy: false, imported: false } }),
    ).toContain('retry');
  });

  it('offers re-extraction for a bookmark, with the AI only when there is an extraction model', () => {
    const ready = { failed: false, blocked: false, busy: false, imported: false };
    expect(ids({ ...empty, bookmark: ready })).toContain('reextract');
    expect(ids({ ...empty, bookmark: ready })).not.toContain('extract-ai');
    expect(ids({ ...empty, bookmark: ready, extractionModel: true })).toEqual(
      expect.arrayContaining(['reextract', 'extract-ai']),
    );
    expect(ids({ ...empty, extractionModel: true })).not.toContain('reextract');
  });

  it('offers no re-extraction while the bookmark is being extracted, like the detail', () => {
    const busy = { failed: false, blocked: false, busy: true, imported: false };
    const found = ids({ ...empty, bookmark: busy, extractionModel: true });
    expect(found).not.toContain('reextract');
    expect(found).not.toContain('extract-ai');
    expect(found).toContain('open-link');
  });

  it('offers no re-extraction for a blocked bookmark: it is rescued from the extension', () => {
    const blocked = { failed: true, blocked: true, busy: false, imported: false };
    const found = ids({ ...empty, bookmark: blocked, extractionModel: true });
    expect(found).not.toContain('reextract');
    expect(found).not.toContain('extract-ai');
  });

  it('names the re-extraction commands like the buttons of the detail and runs them', () => {
    const ran: string[] = [];
    const actions = {
      ...noopActions,
      reextract: () => ran.push('defuddle'),
      extractWithAi: () => ran.push('ai'),
    };
    const context = {
      ...empty,
      bookmark: { failed: false, blocked: false, busy: false, imported: false },
      extractionModel: true,
    };
    const all = commands(actions, context);
    const reextract = all.find((c) => c.id === 'reextract');
    const ai = all.find((c) => c.id === 'extract-ai');
    expect(reextract?.label).toBe(t.reextract);
    expect(ai?.label).toBe(t.extractWithAi);
    reextract?.run();
    ai?.run();
    expect(ran).toEqual(['defuddle', 'ai']);
  });

  it('offers extract-view only with imported bookmarks in view', () => {
    expect(ids(empty)).not.toContain('extract-view');
    const ran: string[] = [];
    const actions = { ...noopActions, extractView: () => ran.push('view') };
    const command = commands(actions, { ...empty, importedInView: 3 }).find(
      (c) => c.id === 'extract-view',
    );
    expect(command?.label).toBe(t.cmdExtractView(3));
    command?.run();
    expect(ran).toEqual(['view']);
  });

  it('an imported bookmark offers Extraer and no re-extraction', () => {
    const ran: string[] = [];
    const actions = { ...noopActions, retry: () => ran.push('extract') };
    const imported = { failed: false, blocked: false, busy: false, imported: true };
    const all = commands(actions, { ...empty, bookmark: imported, extractionModel: true });
    const found = all.map((c) => c.id);
    expect(found).toContain('extract');
    expect(found).not.toContain('reextract');
    expect(found).not.toContain('extract-ai');
    const extract = all.find((c) => c.id === 'extract');
    expect(extract?.label).toBe(t.cmdExtract);
    extract?.run();
    expect(ran).toEqual(['extract']);
    expect(ids({ ...empty, bookmark: { ...imported, imported: false } })).not.toContain('extract');
  });

  it('adds rename and delete for every group', () => {
    const groups = [{ id: 3, name: 'Datos', tags: [] }];
    expect(ids({ ...empty, groups })).toEqual(
      expect.arrayContaining(['rename-group-3', 'delete-group-3']),
    );
  });

  it('offers to move the selected tag to every other group, and out of its own', () => {
    const groups = [
      { id: 1, name: 'Datos', tags: ['sqlite'] },
      { id: 2, name: 'Frontend', tags: [] },
    ];
    const grouped = ids({ ...empty, groups, tag: { name: 'sqlite', groupId: 1 } });
    expect(grouped).toContain('move-tag-2');
    expect(grouped).not.toContain('move-tag-1');
    expect(grouped).toContain('ungroup-tag');
    expect(ids({ ...empty, groups, tag: { name: 'x', groupId: null } })).not.toContain(
      'ungroup-tag',
    );
  });

  it("offers to switch to every profile but the active one, with the rail's shortcut", () => {
    const profiles = [1, 2, 3].map((id) => ({ id, name: `P${id}`, initials: `P${id}` }));
    const switched: number[] = [];
    const all = commands(
      { ...noopActions, switchProfile: (id) => switched.push(id) },
      { ...empty, profiles, activeProfile: 2 },
    );
    const switches = all.filter((c) => c.id.startsWith('switch-profile-'));
    expect(switches.map((c) => c.id)).toEqual(['switch-profile-1', 'switch-profile-3']);
    expect(switches.map((c) => c.shortcut)).toEqual(['⌥⌘1', '⌥⌘3']);
    switches[1].run();
    expect(switched).toEqual([3]);
  });

  it('offers to move the selected bookmark to every profile but the active one', () => {
    const profiles = [1, 2, 3].map((id) => ({ id, name: `P${id}`, initials: `P${id}` }));
    const moved: number[] = [];
    const actions = { ...noopActions, moveToProfile: (id: number) => moved.push(id) };
    const moves = (bookmark: CommandContext['bookmark']): string[] =>
      commands(actions, { ...empty, profiles, activeProfile: 2, bookmark })
        .map((c) => c.id)
        .filter((id) => id.startsWith('move-to-profile-'));
    expect(moves(null)).toEqual([]);
    expect(moves({ failed: false, blocked: false, busy: false, imported: false })).toEqual([
      'move-to-profile-1',
      'move-to-profile-3',
    ]);
    commands(actions, {
      ...empty,
      profiles,
      activeProfile: 2,
      bookmark: { failed: false, blocked: false, busy: false, imported: false },
    })
      .find((c) => c.id === 'move-to-profile-3')
      ?.run();
    expect(moved).toEqual([3]);
  });

  it('gives no shortcut to profiles past the ninth', () => {
    const profiles = Array.from({ length: 10 }, (_, i) => ({
      id: i + 1,
      name: 'P',
      initials: 'P',
    }));
    const all = commands(noopActions, { ...empty, profiles, activeProfile: 1 });
    expect(all.find((c) => c.id === 'switch-profile-10')?.shortcut).toBeUndefined();
    expect(all.find((c) => c.id === 'switch-profile-9')?.shortcut).toBe('⌥⌘9');
  });

  it('sets the theme it names', () => {
    const chosen: string[] = [];
    const all = commands({ ...noopActions, setTheme: (theme) => chosen.push(theme) }, empty);
    all.find((c) => c.id === 'theme-light')?.run();
    expect(chosen).toEqual(['light']);
  });

  it('runs the action it names with its target', () => {
    const moved: [string, number | null][] = [];
    const all = commands(
      { ...noopActions, moveTag: (tag, groupId) => moved.push([tag, groupId]) },
      {
        ...empty,
        groups: [{ id: 2, name: 'Frontend', tags: [] }],
        tag: { name: 'svelte', groupId: null },
      },
    );
    all.find((c) => c.id === 'move-tag-2')?.run();
    expect(moved).toEqual([['svelte', 2]]);
  });
});

describe('filterCommands', () => {
  const all = commands(noopActions, empty);

  it('keeps everything for a blank query', () => {
    expect(filterCommands(all, '  ')).toHaveLength(all.length);
  });

  it('filters by every word of the query, in any order and case', () => {
    expect(filterCommands(all, 'GRUPO nuevo').map((c) => c.id)).toEqual(['new-group']);
    expect(filterCommands(all, 'zzz')).toEqual([]);
  });
});
