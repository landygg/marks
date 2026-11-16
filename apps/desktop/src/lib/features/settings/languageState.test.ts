import { beforeEach, describe, expect, it, vi } from 'vitest';
import * as api from '../../api';
import { applyLanguage, askLanguage, language, loadLanguage } from './languageState.svelte';

vi.mock('../../api', async (original) => ({
  ...(await original<typeof api>()),
  localeInfo: vi.fn(),
  setLocale: vi.fn(),
}));

beforeEach(() => {
  vi.mocked(api.localeInfo).mockReset();
  vi.mocked(api.setLocale).mockReset();
  language.setting = 'system';
  language.pending = null;
});

describe('language state', () => {
  it('keeps pending while the relaunch is in flight and ignores a second confirm', async () => {
    let finish: () => void = () => {};
    vi.mocked(api.setLocale).mockReturnValue(new Promise<void>((r) => (finish = r)));
    askLanguage('en');
    const first = applyLanguage();
    await applyLanguage();
    expect(language.pending).toBe('en');
    expect(api.setLocale).toHaveBeenCalledTimes(1);
    finish();
    await first;
  });

  it('clears pending when the relaunch fails', async () => {
    vi.mocked(api.setLocale).mockRejectedValue('boom');
    askLanguage('en');
    await applyLanguage();
    expect(language.pending).toBeNull();
  });

  it('loads the setting lazily and survives a synchronous throw', async () => {
    expect(api.localeInfo).not.toHaveBeenCalled();
    vi.mocked(api.localeInfo).mockResolvedValue({ setting: 'es', active: 'es' });
    await loadLanguage();
    expect(language.setting).toBe('es');
    vi.mocked(api.localeInfo).mockImplementation(() => {
      throw new Error('no tauri');
    });
    await loadLanguage();
    expect(language.setting).toBe('es');
  });
});
