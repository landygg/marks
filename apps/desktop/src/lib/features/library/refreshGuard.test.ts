import { describe, expect, it } from 'vitest';
import { createRefreshGuard, keepUnchanged, kept } from './refreshGuard';

describe('createRefreshGuard', () => {
  it('applies the latest refresh only', () => {
    const guard = createRefreshGuard();
    const first = guard.start();
    const second = guard.start();
    expect(guard.current(first)).toBe(false);
    expect(guard.current(second)).toBe(true);
  });

  it('discards a refresh that was running when a switch started', () => {
    const guard = createRefreshGuard();
    const before = guard.start();
    guard.beginSwitch();
    guard.endSwitch();
    expect(guard.current(before)).toBe(false);
  });

  it('discards a refresh that resolves while a switch is in flight', () => {
    const guard = createRefreshGuard();
    guard.beginSwitch();
    const during = guard.start();
    expect(guard.current(during)).toBe(false);
    guard.endSwitch();
    // Started mid-switch: its calls may straddle both profiles even if it resolves afterwards.
    const after = guard.start();
    expect(guard.current(after)).toBe(true);
  });
});

describe('kept', () => {
  it('keeps an id that is still loaded in the same profile', () => {
    expect(kept(3, [1, 3], 1, 1)).toBe(3);
  });

  it('drops an id that is gone', () => {
    expect(kept(3, [1, 2], 1, 1)).toBeNull();
    expect(kept(null, [1], 1, 1)).toBeNull();
  });

  it('drops an id after a profile switch, even if the new profile has the same id', () => {
    expect(kept(3, [1, 3], 1, 2)).toBeNull();
  });
});

describe('keepUnchanged', () => {
  const row = (id: number, title = 'a', tags = ['x']) => ({ id, title, tags });

  it('keeps the previous object of a row whose fields are all equal', () => {
    const old = row(1);
    expect(keepUnchanged([old], [row(1)])[0]).toBe(old);
  });

  it('takes the new object when a field or a tag changed', () => {
    const fresh = [row(1, 'b'), row(2, 'a', ['y'])];
    const result = keepUnchanged([row(1), row(2)], fresh);
    expect(result[0]).toBe(fresh[0]);
    expect(result[1]).toBe(fresh[1]);
  });

  it('follows the new list: new rows in, gone rows out, new order', () => {
    const one = row(1);
    const three = row(3);
    expect(keepUnchanged([one, row(2)], [three, row(1)])).toEqual([three, one]);
    expect(keepUnchanged([one, row(2)], [three, row(1)])[1]).toBe(one);
  });
});
