// Pure rules of the notification panel and its toasts: which actions a notification offers, which
// toasts show, and the timer that pauses while the pointer or the focus is on a toast.
import type { Bookmark, Notification, Target } from '../../api';
import type { SettingsSection } from '../settings/settingsLogic';

export const MAX_TOASTS = 3;
export const TOAST_MS = 6000;
const ERR_NO_SECTION = 'a bookmark notification has no Settings section'; // i18n-allow: invariant

export type NoticeAction = 'open' | 'retry' | 'settings';

/** The icon of each action's button: open in the list, retry, go to Settings. */
export const ACTION_ICONS = { open: 'external', retry: 'retry', settings: 'gear' } as const;

/** How the bell is found from another zone, and the id of the panel it controls. */
export const BELL_SELECTOR = '[data-notifications-trigger]';
export const PANEL_ID = 'notifications-panel';

/**
 * Open for a bookmark, plus retry if it is still failed; Settings for the AI and the API. A bookmark
 * of another profile gets none: ids repeat across profiles, and `bookmarks` are the active one's.
 */
export function actionsFor(
  n: Notification,
  bookmarks: Bookmark[],
  profileId: number,
): NoticeAction[] {
  const target = n.target;
  if (!target) return [];
  if (target.kind !== 'bookmark') return ['settings'];
  if (n.profile !== profileId) return [];
  const failed = bookmarks.some((b) => b.id === target.id && b.status === 'failed');
  return failed ? ['open', 'retry'] : ['open'];
}

/** The section "Open Settings" leads to; only for the targets `actionsFor` gives it to. */
export function settingsSectionFor(target: Target): SettingsSection {
  if (target.kind === 'bookmark') throw new Error(ERR_NO_SECTION);
  return target.kind;
}

/** Adds `n` as the newest toast; a repeat replaces its old toast instead of stacking. */
export function upsertToast(toasts: Notification[], n: Notification): Notification[] {
  return [...toasts.filter((toast) => toast.id !== n.id), n];
}

/** The newest `max` toasts, oldest first, and how many are left out. */
export function visibleToasts(
  toasts: Notification[],
  max = MAX_TOASTS,
): { shown: Notification[]; hidden: number } {
  const hidden = Math.max(0, toasts.length - max);
  return { shown: toasts.slice(hidden), hidden };
}

/** Why a toast is paused: the pointer is on it, or the focus is inside it. */
export type PauseCause = 'pointer' | 'focus';

/** Calls `onExpire` once `ms` have run with no cause pausing it. */
export function createToastTimer(
  onExpire: () => void,
  ms = TOAST_MS,
): {
  pause(cause?: PauseCause): void;
  resume(cause?: PauseCause): void;
  /** Drops every cause: for a toast that left the screen and will get no `pointerleave`. */
  release(): void;
  cancel(): void;
} {
  const causes = new Set<PauseCause>();
  let left = ms;
  let deadline = 0;
  let handle: ReturnType<typeof setTimeout> | undefined;
  let over = false;

  const start = () => {
    deadline = Date.now() + left;
    handle = setTimeout(() => {
      over = true;
      onExpire();
    }, left);
  };
  start();

  const run = () => {
    if (causes.size === 0 && handle === undefined && !over) start();
  };

  return {
    pause(cause = 'pointer') {
      causes.add(cause);
      if (handle === undefined) return;
      clearTimeout(handle);
      handle = undefined;
      left = Math.max(0, deadline - Date.now());
    },
    resume(cause = 'pointer') {
      causes.delete(cause);
      run();
    },
    release() {
      causes.clear();
      run();
    },
    cancel() {
      clearTimeout(handle);
      handle = undefined;
      over = true;
    },
  };
}

/** The toast that takes the focus when the focused one closes: the next, else the previous, else none (the bell). */
export function focusAfterClose(ids: number[], closedId: number): number | null {
  const at = ids.indexOf(closedId);
  if (at < 0) return null;
  return ids[at + 1] ?? ids[at - 1] ?? null;
}
