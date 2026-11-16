// The language setting and the change waiting for its confirmation (the select and the palette share it).
import { localeInfo, setLocale, type LocaleInfo } from '../../api';
import { openSettings, report } from '../library/libraryState.svelte';

type Setting = LocaleInfo['setting'];

export const language = $state<{ setting: Setting; pending: Setting | null }>({
  setting: 'system',
  pending: null,
});

/** Reads the saved setting; lazy, so importing this module never calls the backend. */
export async function loadLanguage(): Promise<void> {
  try {
    language.setting = (await localeInfo()).setting;
  } catch (err) {
    report(err, { profile: null, toast: false });
  }
}

/** Asks before relaunching; the palette lands on the row that shows the question. */
export function askLanguage(setting: Setting): void {
  language.pending = setting === language.setting ? null : setting;
  openSettings('appearance');
}

export function cancelLanguage(): void {
  language.pending = null;
}

// The relaunch is in flight: a second confirmation would start another one.
let applying = false;

/** Relaunches in the pending language; it only returns when that failed. */
export async function applyLanguage(): Promise<void> {
  const { pending } = language;
  if (!pending || applying) return;
  applying = true;
  try {
    await setLocale(pending);
  } catch (err) {
    language.pending = null;
    report(err, { profile: null });
  } finally {
    applying = false;
  }
}
