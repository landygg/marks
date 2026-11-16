import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { ProfilesView } from '../../api';

// Each `profiles` call waits until the test answers it, so refreshes overlap as they do when the
// worker emits `LIBRARY_CHANGED` faster than one refresh takes.
const answers: ((view: ProfilesView) => void)[] = [];

vi.mock('../../api', async (original) => ({
  ...(await original<typeof import('../../api')>()),
  list: vi.fn(async () => []),
  stats: vi.fn(async () => ({
    total: 0,
    pending: 0,
    failed: 0,
    unsummarized: 0,
    unextracted: 0,
    imported: 0,
  })),
  tagGroups: vi.fn(async () => []),
  hasExtractionModel: vi.fn(async () => false),
  notifications: vi.fn(async () => []),
  profiles: vi.fn(() => new Promise<ProfilesView>((resolve) => answers.push(resolve))),
}));

const { lib, refresh } = await import('./libraryState.svelte');

const personal = { id: 1, name: 'Personal', initials: 'PE' };
const created = { id: 2, name: 'Trabajo', initials: 'TR' };

// Lets every settled promise run its continuations.
const settle = () => new Promise((resolve) => setTimeout(resolve, 0));

describe('refresh', () => {
  beforeEach(() => {
    answers.length = 0;
    lib.profiles = [personal];
  });

  it('applies its data even when another refresh is asked for before it lands', async () => {
    const first = refresh();
    await settle();
    // An event arrives while the first refresh waits for the backend.
    const second = refresh();
    await settle();
    answers.shift()?.({ profiles: [personal, created], active: 2 });
    await settle();
    expect(lib.profiles).toEqual([personal, created]);

    // The second call still gets a load of its own, started after it was made.
    expect(answers).toHaveLength(1);
    answers.shift()?.({ profiles: [personal, created], active: 2 });
    await Promise.all([first, second]);
    expect(lib.profileId).toBe(2);
  });

  it('keeps applying data while events never stop', async () => {
    void refresh();
    for (let i = 0; i < 5; i += 1) {
      await settle();
      void refresh();
      await settle();
      answers.shift()?.({ profiles: [personal, { ...created, name: `Trabajo ${i}` }], active: 1 });
      await settle();
      expect(lib.profiles.at(-1)?.name).toBe(`Trabajo ${i}`);
    }
    answers.shift()?.({ profiles: [personal], active: 1 });
  });
});
