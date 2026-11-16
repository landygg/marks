import { describe, expect, it } from 'vitest';
import { keyStep } from './resize';

const b = { min: 160, max: 320, reset: 200 };

describe('keyStep', () => {
  it('moves by 8 and by 24 with shift', () => {
    expect(keyStep('ArrowRight', false, 200, b)).toBe(208);
    expect(keyStep('ArrowLeft', false, 200, b)).toBe(192);
    expect(keyStep('ArrowRight', true, 200, b)).toBe(224);
  });

  it('jumps with Home, End and Enter', () => {
    expect(keyStep('Home', false, 250, b)).toBe(160);
    expect(keyStep('End', false, 250, b)).toBe(320);
    expect(keyStep('Enter', false, 250, b)).toBe(200);
  });

  it('clamps to min and max', () => {
    expect(keyStep('ArrowLeft', false, 160, b)).toBe(160);
    expect(keyStep('ArrowRight', false, 316, { ...b, max: 316 })).toBe(316);
  });

  it('ignores other keys', () => {
    expect(keyStep('a', false, 200, b)).toBeNull();
    expect(keyStep('Tab', false, 200, b)).toBeNull();
  });
});
