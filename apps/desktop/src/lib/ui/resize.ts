export interface ResizeBounds {
  min: number;
  max: number;
  reset: number;
}

export const RESIZE_STEP = 8;
export const RESIZE_STEP_LARGE = 24;

export function clampTo(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, Math.round(value)));
}

/** The width a key moves to, or `null` for a key the handle does not take. Rounded and within bounds. */
export function keyStep(
  key: string,
  shift: boolean,
  value: number,
  bounds: ResizeBounds,
): number | null {
  const step = shift ? RESIZE_STEP_LARGE : RESIZE_STEP;
  switch (key) {
    case 'ArrowRight':
      return clampTo(value + step, bounds.min, bounds.max);
    case 'ArrowLeft':
      return clampTo(value - step, bounds.min, bounds.max);
    case 'Home': // i18n-allow: key name
      return bounds.min;
    case 'End': // i18n-allow: key name
      return bounds.max;
    case 'Enter': // i18n-allow: key name
      return bounds.reset;
    default:
      return null;
  }
}
