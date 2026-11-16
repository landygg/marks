// The corner radius of panels and overlays, in pixels: a preference of this window.
export const CORNERS = [0, 2, 4, 6, 8] as const;
export type Corner = (typeof CORNERS)[number];
export const DEFAULT_CORNER: Corner = 4;
/** Controls never get rounder than this. */
const CONTROL_MAX_RADIUS = 2;

/** The stored value if it is one of the five, else the default. */
export function parseCorner(value: string | null): Corner {
  return CORNERS.find((corner) => String(corner) === value) ?? DEFAULT_CORNER;
}

/** `md` for panels and overlays, `sm` for controls. */
export function radii(corner: Corner): { md: string; sm: string } {
  return { md: `${corner}px`, sm: `${Math.min(CONTROL_MAX_RADIUS, corner)}px` };
}
