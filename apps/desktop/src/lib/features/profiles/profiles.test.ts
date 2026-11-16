import { describe, expect, it } from 'vitest';
import {
  bookmarkDragData,
  isOnScreen,
  parseBookmarkDrag,
  suggestInitials,
  validProfile,
} from './profiles';

describe('suggestInitials', () => {
  it('takes two letters of one word, or the first letter of the first two words', () => {
    expect(suggestInitials('Lecturas')).toBe('LE');
    expect(suggestInitials('  cliente acme ')).toBe('CA');
    expect(suggestInitials('x')).toBe('X');
    expect(suggestInitials('')).toBe('');
  });

  it('counts characters, not UTF-16 units', () => {
    expect(suggestInitials('🙂 éxito')).toBe('🙂É');
  });
});

describe('validProfile', () => {
  it('wants a name of 1 to 40 characters and initials of 1 or 2', () => {
    expect(validProfile('Trabajo', 'TR')).toBe(true);
    expect(validProfile('  ', 'TR')).toBe(false);
    expect(validProfile('a'.repeat(41), 'A')).toBe(false);
    expect(validProfile('Trabajo', '')).toBe(false);
    expect(validProfile('Trabajo', 'ABC')).toBe(false);
  });

  it('trims both and counts initials after uppercasing, like the backend', () => {
    expect(validProfile(` ${'a'.repeat(40)} `, ' t ')).toBe(true);
    // "ß" becomes "SS".
    expect(validProfile('Straße', 'aß')).toBe(false);
  });
});

describe('isOnScreen', () => {
  const screen = { selectedId: 7, profileId: 2 };

  it('is the selected bookmark only with the same id in the same profile', () => {
    expect(isOnScreen({ id: 7, profile: 2 }, screen)).toBe(true);
    // Ids repeat across profiles: a switch landed after the move started.
    expect(isOnScreen({ id: 7, profile: 1 }, screen)).toBe(false);
    expect(isOnScreen({ id: 8, profile: 2 }, screen)).toBe(false);
    expect(isOnScreen({ id: 7, profile: 2 }, { selectedId: null, profileId: 2 })).toBe(false);
  });
});

describe('bookmark drag data', () => {
  it("carries the profile the row was shown in and the bookmark's id", () => {
    expect(parseBookmarkDrag(bookmarkDragData(3, 42))).toEqual({ profile: 3, id: 42 });
  });

  it('refuses anything that is not two integers', () => {
    for (const data of ['', '3', '3:', ':4', 'a:b', '3:4.5', '3:4:5']) {
      expect(parseBookmarkDrag(data)).toBeNull();
    }
  });
});
