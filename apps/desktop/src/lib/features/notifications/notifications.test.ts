import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { Bookmark, Notification, Target } from '../../api';
import {
  MAX_TOASTS,
  TOAST_MS,
  actionsFor,
  createToastTimer,
  focusAfterClose,
  settingsSectionFor,
  upsertToast,
  visibleToasts,
} from './notifications';

function notice(id: number, target: Target | null = null): Notification {
  return {
    id,
    profile: 1,
    severity: 'error',
    message: `m${id}`,
    detail: null,
    target,
    count: 1,
    last_at: id,
    read: false,
  };
}

const bookmark = (id: number, status: Bookmark['status']) => ({ id, status }) as Bookmark;

describe('actionsFor', () => {
  it('open_for_a_bookmark_target', () => {
    expect(actionsFor(notice(1, { kind: 'bookmark', id: 3 }), [], 1)).toEqual(['open']);
  });

  it('retry_only_for_a_failed_bookmark_here', () => {
    const n = notice(1, { kind: 'bookmark', id: 3 });
    expect(actionsFor(n, [bookmark(3, 'failed')], 1)).toEqual(['open', 'retry']);
    expect(actionsFor(n, [bookmark(3, 'ready')], 1)).toEqual(['open']);
    // Another bookmark failed, not this one.
    expect(actionsFor(n, [bookmark(4, 'failed')], 1)).toEqual(['open']);
  });

  it('settings_for_ai_and_extension', () => {
    expect(actionsFor(notice(1, { kind: 'ai' }), [], 1)).toEqual(['settings']);
    expect(actionsFor(notice(1, { kind: 'extension' }), [], 1)).toEqual(['settings']);
    expect(settingsSectionFor({ kind: 'ai' })).toBe('ai');
    expect(settingsSectionFor({ kind: 'extension' })).toBe('extension');
    expect(settingsSectionFor({ kind: 'updates' })).toBe('updates');
  });

  it('nothing_without_target', () => {
    expect(actionsFor(notice(1), [], 1)).toEqual([]);
  });

  it('no_bookmark_actions_for_another_profile', () => {
    // Bookmark 3 of profile 1 failed; the toast names bookmark 3 of profile 2, now behind a switch.
    const elsewhere = { ...notice(1, { kind: 'bookmark', id: 3 }), profile: 2 };
    expect(actionsFor(elsewhere, [bookmark(3, 'failed')], 1)).toEqual([]);
    expect(actionsFor({ ...elsewhere, target: { kind: 'ai' } }, [], 1)).toEqual(['settings']);
  });
});

describe('toasts', () => {
  it('upsert_toast_replaces_the_same_notification', () => {
    const a = notice(1);
    const b = notice(2);
    const again = { ...a, count: 2 };
    expect(upsertToast([a, b], again)).toEqual([b, again]);
  });

  it('visible_toasts_keep_the_newest_three', () => {
    const toasts = [1, 2, 3, 4, 5].map((id) => notice(id));
    expect(MAX_TOASTS).toBe(3);
    const { shown, hidden } = visibleToasts(toasts);
    expect(shown.map((n) => n.id)).toEqual([3, 4, 5]);
    expect(hidden).toBe(2);
    expect(visibleToasts(toasts.slice(0, 2))).toEqual({ shown: toasts.slice(0, 2), hidden: 0 });
  });
});

describe('createToastTimer', () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it('toast_timer_expires_after_6s', () => {
    const onExpire = vi.fn();
    createToastTimer(onExpire);
    expect(TOAST_MS).toBe(6000);
    vi.advanceTimersByTime(5999);
    expect(onExpire).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1);
    expect(onExpire).toHaveBeenCalledOnce();
  });

  it('toast_timer_pauses_and_resumes_with_the_time_left', () => {
    const onExpire = vi.fn();
    const timer = createToastTimer(onExpire);
    vi.advanceTimersByTime(4000);
    timer.pause();
    vi.advanceTimersByTime(10000);
    expect(onExpire).not.toHaveBeenCalled();
    timer.resume();
    vi.advanceTimersByTime(1999);
    expect(onExpire).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1);
    expect(onExpire).toHaveBeenCalledOnce();
  });

  it('pause_and_resume_twice_in_a_row_change_nothing', () => {
    const onExpire = vi.fn();
    const timer = createToastTimer(onExpire);
    vi.advanceTimersByTime(1000);
    timer.pause();
    timer.pause();
    vi.advanceTimersByTime(1000);
    timer.resume();
    timer.resume();
    vi.advanceTimersByTime(4999);
    expect(onExpire).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1);
    expect(onExpire).toHaveBeenCalledOnce();
  });

  it('pause_holds_until_every_cause_is_gone', () => {
    const onExpire = vi.fn();
    const timer = createToastTimer(onExpire);
    timer.pause('pointer');
    timer.pause('focus');
    timer.resume('pointer');
    vi.advanceTimersByTime(TOAST_MS * 2);
    expect(onExpire).not.toHaveBeenCalled();
    timer.resume('focus');
    vi.advanceTimersByTime(TOAST_MS);
    expect(onExpire).toHaveBeenCalledOnce();
  });

  it('release_resumes_whatever_held_it', () => {
    // A hovered toast pushed out of view never gets its pointerleave.
    const onExpire = vi.fn();
    const timer = createToastTimer(onExpire);
    timer.pause('pointer');
    timer.pause('focus');
    timer.release();
    vi.advanceTimersByTime(TOAST_MS);
    expect(onExpire).toHaveBeenCalledOnce();
  });

  it('cancel_stops_the_timer', () => {
    const onExpire = vi.fn();
    const timer = createToastTimer(onExpire);
    timer.cancel();
    vi.advanceTimersByTime(TOAST_MS * 2);
    timer.resume();
    vi.advanceTimersByTime(TOAST_MS * 2);
    expect(onExpire).not.toHaveBeenCalled();
  });
});

describe('focusAfterClose', () => {
  it('moves to the next toast, else the previous one, else the bell (null)', () => {
    expect(focusAfterClose([1, 2, 3], 2)).toBe(3);
    expect(focusAfterClose([1, 2, 3], 3)).toBe(2);
    expect(focusAfterClose([1], 1)).toBeNull();
    expect(focusAfterClose([1, 2], 9)).toBeNull();
  });
});
