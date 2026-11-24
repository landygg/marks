import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { Bookmark } from '../../api';
import { t } from '../../strings';

const api = vi.hoisted(() => ({
  retryFailed: vi.fn<(profile: number) => Promise<number>>(async () => 0),
  list: vi.fn(async () => []),
  getSettings: vi.fn(async () => ({
    settings: { enabled: true, provider: 'openai', endpoint: null },
  })),
}));
const notices = vi.hoisted(() => ({ notify: vi.fn(async () => {}) }));

vi.mock('../../api', async (original) => ({
  ...(await original<typeof import('../../api')>()),
  ...api,
}));
vi.mock('../notifications/notificationsState.svelte', async (original) => ({
  ...(await original<typeof import('../notifications/notificationsState.svelte')>()),
  ...notices,
}));

const { lib, askRetryFailed, runRetryFailed, choose } = await import('./libraryState.svelte');
const { ALL, FAILED } = await import('./sidebar');

const failed = (id: number) =>
  ({ id, status: 'failed', category: null, tags: [] }) as unknown as Bookmark;

describe('retry failed', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    lib.profileId = 2;
    lib.bookmarks = [failed(1), failed(2), failed(3)];
    lib.selection = FAILED;
    lib.confirmRetryFailed = null;
  });

  it('ask_retry_failed_remembers_the_profile_and_count', async () => {
    await askRetryFailed();
    expect(lib.confirmRetryFailed).toEqual({ profile: 2, count: 3, remote: true });
  });

  it('retry_failed_runs_in_the_asked_profile', async () => {
    await askRetryFailed();
    lib.profileId = 1;
    api.retryFailed.mockResolvedValueOnce(3);
    await runRetryFailed();
    expect(api.retryFailed).toHaveBeenCalledWith(2);
    expect(api.list).not.toHaveBeenCalled();
    expect(lib.confirmRetryFailed).toBeNull();
  });

  it('retry_failed_toasts_the_real_count', async () => {
    await askRetryFailed();
    lib.profileId = 1;
    api.retryFailed.mockResolvedValueOnce(4);
    await runRetryFailed();
    expect(notices.notify).toHaveBeenCalledWith(
      expect.objectContaining({ profile: 2, message: t.notices.retryQueued(4) }),
      { toast: true },
    );
  });

  it('choose_drops_the_retry_question', async () => {
    await askRetryFailed();
    choose(ALL);
    expect(lib.confirmRetryFailed).toBeNull();
  });
});
