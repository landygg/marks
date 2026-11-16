import { describe, expect, it } from 'vitest';
import { isExtracting } from './extracting';

describe('isExtracting', () => {
  const list = [{ profile: 1, id: 5 }];

  it('matches profile and id together', () => {
    expect(isExtracting(list, 1, 5)).toBe(true);
  });

  it('ignores the same id in another profile', () => {
    expect(isExtracting(list, 2, 5)).toBe(false);
  });

  it('is false for an empty list', () => {
    expect(isExtracting([], 1, 5)).toBe(false);
  });
});
