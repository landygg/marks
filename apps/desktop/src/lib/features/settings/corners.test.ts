import { describe, expect, it } from 'vitest';
import design from '../../../../../../DESIGN.md?raw';
import { DEFAULT_CORNER, parseCorner, radii } from './corners';

describe('corners', () => {
  it('parseCorner reads the five values', () => {
    expect(parseCorner('0')).toBe(0);
    expect(parseCorner('8')).toBe(8);
  });

  it('parseCorner falls back to 4', () => {
    for (const value of [null, '', '5', 'abc', '4.5']) expect(parseCorner(value)).toBe(4);
  });

  it('radii keeps controls at 2px at most', () => {
    expect(radii(0)).toEqual({ md: '0px', sm: '0px' });
    expect(radii(2)).toEqual({ md: '2px', sm: '2px' });
    expect(radii(8)).toEqual({ md: '8px', sm: '2px' });
  });

  it('the default is DESIGN.md rounded.md', () => {
    const rounded = design.split('---')[1].match(/^rounded:\n((?: .*\n)+)/m)?.[1] ?? '';
    expect(rounded.match(/^ {2}md: "?([^"\n]+)"?$/m)?.[1]).toBe(`${DEFAULT_CORNER}px`);
  });
});
