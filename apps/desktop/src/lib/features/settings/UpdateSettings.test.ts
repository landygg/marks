// @vitest-environment jsdom
import { flushSync, mount, tick, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { checkUpdate, updateInfo, type UpdateInfo } from '../../api';
import { report } from '../library/libraryState.svelte';
import { t } from '../../strings';
import UpdateSettings from './UpdateSettings.svelte';

vi.mock('../../api', () => ({
  checkUpdate: vi.fn(),
  installUpdate: vi.fn(),
  openExternal: vi.fn(),
  setUpdateAuto: vi.fn(),
  updateInfo: vi.fn(),
  UPDATE_CHANGED: 'update-changed',
}));
vi.mock('../library/libraryState.svelte', () => ({ report: vi.fn() }));
// The backend's events: each `listen` keeps its handler here, to be fired by the test.
const handlers = new Map<string, () => void>();
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(async (event: string, handler: () => void) => {
    handlers.set(event, handler);
    return () => handlers.delete(event);
  }),
}));

const INFO: UpdateInfo = { current: '1.2.0', available: null, supported: true, auto: true };
let view: ReturnType<typeof mount> | null = null;

async function open(info: UpdateInfo = INFO): Promise<void> {
  vi.mocked(updateInfo).mockResolvedValue({ ...info });
  view = mount(UpdateSettings, { target: document.body });
  await vi.waitFor(() => expect(document.body.querySelector('button')).not.toBeNull());
}

const buttonNamed = (name: string) =>
  [...document.body.querySelectorAll('button')].find((b) => b.textContent?.trim() === name);

beforeEach(() => {
  vi.mocked(report).mockClear();
});

afterEach(() => {
  if (view) unmount(view);
  view = null;
  document.body.replaceChildren();
});

describe('UpdateSettings, a version that needs a newer macOS', () => {
  it('says so as plain status text, not as an error, and adds no notification', async () => {
    vi.mocked(checkUpdate).mockResolvedValue({ kind: 'needs_macos', message: 'Needs macOS 99.0' });
    await open();
    buttonNamed(t.updateCheck)?.click();
    await vi.waitFor(() => expect(document.body.textContent).toContain('Needs macOS 99.0'));
    await tick();
    flushSync();
    const help = [...document.body.querySelectorAll('[role]')].find((el) =>
      el.textContent?.includes('Needs macOS 99.0'),
    );
    expect(help?.getAttribute('role')).toBe('status');
    expect(help?.classList.contains('error-text')).toBe(false);
    expect(report).not.toHaveBeenCalled();
    expect(buttonNamed(t.updateInstall('1.3.0'))).toBeUndefined();
  });

  it('still offers an available version', async () => {
    vi.mocked(checkUpdate).mockResolvedValue({ kind: 'available', version: '1.3.0' });
    await open();
    buttonNamed(t.updateCheck)?.click();
    await vi.waitFor(() => expect(buttonNamed(t.updateInstall('1.3.0'))).toBeDefined());
  });
});

describe('UpdateSettings, a later check that withdraws the update', () => {
  it('drops the Install button', async () => {
    await open({ ...INFO, available: '1.3.0' });
    expect(buttonNamed(t.updateInstall('1.3.0'))).toBeDefined();
    vi.mocked(updateInfo).mockResolvedValue({ ...INFO, available: null });
    await vi.waitFor(() => expect(handlers.has('update-changed')).toBe(true));
    handlers.get('update-changed')?.();
    await vi.waitFor(() => expect(buttonNamed(t.updateInstall('1.3.0'))).toBeUndefined());
    expect(buttonNamed(t.updateCheck)).toBeDefined();
  });

  it('stops listening when it closes', async () => {
    await open();
    await vi.waitFor(() => expect(handlers.has('update-changed')).toBe(true));
    if (view) unmount(view);
    view = null;
    await vi.waitFor(() => expect(handlers.has('update-changed')).toBe(false));
  });
});
