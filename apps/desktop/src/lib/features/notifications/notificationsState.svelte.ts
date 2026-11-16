// State of the notification panel and its toasts. The log itself lives in the backend: `list` is
// the active profile's copy, read again on every `NOTIFICATIONS_CHANGED`. Toasts live only here,
// because only the window asks for them.
import {
  clearNotifications,
  dismissNotification,
  markNotificationsRead,
  notifications as loadNotifications,
  pushNotification,
  type NewNotification,
  type Notification,
} from '../../api';
import {
  choose,
  lib,
  openSettings,
  report,
  retryBookmark,
  select,
} from '../library/libraryState.svelte';
import { coalesce, createRefreshGuard } from '../library/refreshGuard';
import { ALL } from '../library/sidebar';
import {
  BELL_SELECTOR,
  createToastTimer,
  settingsSectionFor,
  upsertToast,
  visibleToasts,
  type NoticeAction,
  type PauseCause,
} from './notifications';

export const notices = $state({
  list: [] as Notification[],
  open: false,
  toasts: [] as Notification[],
  /** The profile the panel was opened in: closing marks its notifications as read. */
  openedIn: null as number | null,
});

export const unreadCount = (): number => notices.list.filter((n) => !n.read).length;

const guard = createRefreshGuard();

/**
 * Reloads the list of the profile on screen, one run at a time (`coalesce`); a response for a profile
 * no longer on screen is dropped. The profile goes with the request, so the backend cannot answer
 * for another one.
 */
export const refreshNotifications = coalesce(async () => {
  const ticket = guard.start();
  const profile = lib.profileId;
  try {
    const list = await loadNotifications(profile);
    if (guard.current(ticket) && lib.profileId === profile) notices.list = list;
  } catch (err) {
    // Not reported: the report would change the log, and its event would land here again.
    console.error(err);
  }
});

// eslint-disable-next-line svelte/prefer-svelte-reactivity -- bookkeeping, never rendered: `toasts` is the state.
const timers = new Map<number, ReturnType<typeof createToastTimer>>();

function removeToast(id: number): void {
  timers.get(id)?.cancel();
  timers.delete(id);
  notices.toasts = notices.toasts.filter((toast) => toast.id !== id);
}

function showToast(n: Notification): void {
  // A repeat replaces its toast, and its clock starts again.
  timers.get(n.id)?.cancel();
  notices.toasts = upsertToast(notices.toasts, n);
  timers.set(
    n.id,
    createToastTimer(() => removeToast(n.id)),
  );
  // A toast pushed out of view gets no pointerleave or focusout: whatever paused it lets go.
  const { shown } = visibleToasts(notices.toasts);
  for (const toast of notices.toasts) if (!shown.includes(toast)) timers.get(toast.id)?.release();
}

function clearToasts(): void {
  for (const timer of timers.values()) timer.cancel();
  timers.clear();
  notices.toasts = [];
}

/** Toasts do not expire while the pointer or the focus is on them; each cause is let go on its own. */
export const pauseToast = (id: number, cause: PauseCause): void => timers.get(id)?.pause(cause);
export const resumeToast = (id: number, cause: PauseCause): void => timers.get(id)?.resume(cause);
/** Hides a toast; the notification stays in the panel. */
export const closeToast = (id: number): void => removeToast(id);

/** Records `n`; with `toast`, and the panel closed, it also shows as a toast. */
export async function notify(n: NewNotification, { toast }: { toast: boolean }): Promise<void> {
  let saved: Notification;
  try {
    saved = await pushNotification(n);
  } catch (err) {
    // Nowhere left to show it: reporting would push again.
    console.error(err);
    return;
  }
  // With the panel open, the new entry shows in the list itself.
  if (toast && !notices.open) showToast(saved);
}

export function openNotifications(): void {
  clearToasts();
  notices.openedIn = lib.profileId;
  notices.open = true;
}

/**
 * Closes the panel and marks as read what it showed. The focus goes back to the bell, except after a
 * click elsewhere, which already put it where the user wanted.
 */
export async function closeNotifications({ restoreFocus = true } = {}): Promise<void> {
  if (!notices.open) return;
  const profile = notices.openedIn;
  notices.open = false;
  notices.openedIn = null;
  if (restoreFocus) document.querySelector<HTMLElement>(BELL_SELECTOR)?.focus();
  if (profile !== null) await markNoticesRead(profile);
}

export function toggleNotifications(): void {
  if (notices.open) void closeNotifications();
  else openNotifications();
}

/** "Mark all as read": the panel stays open. */
export async function markNoticesRead(profile = notices.openedIn ?? lib.profileId): Promise<void> {
  try {
    await markNotificationsRead(profile);
  } catch (err) {
    report(err, { toast: false });
  }
}

export async function dismissNotice(id: number): Promise<void> {
  try {
    await dismissNotification(id);
  } catch (err) {
    report(err);
  }
}

/** Empties the profile's notifications and the global ones. */
export async function clearNotices(profile: number): Promise<void> {
  try {
    await clearNotifications(profile);
  } catch (err) {
    report(err);
  }
}

/**
 * A bookmark action works on the notification's own profile: a toast can outlive a profile switch,
 * and ids repeat across profiles. `actionsFor` offers none for another profile; this checks again.
 */
export function runNoticeAction(n: Notification, action: NoticeAction): void {
  const target = n.target;
  if (!target) return;
  if (action === 'settings') {
    openSettings(settingsSectionFor(target));
    void closeNotifications();
  } else if (target.kind === 'bookmark' && n.profile === lib.profileId) {
    if (action === 'retry') {
      void retryBookmark(n.profile, target.id);
      return;
    }
    // Back to "All" so the row shows whatever was selected before.
    choose(ALL);
    select(target.id);
    void closeNotifications();
  }
}
