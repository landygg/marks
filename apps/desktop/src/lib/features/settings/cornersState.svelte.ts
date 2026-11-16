// The chosen corner radius, in the webview's localStorage like the theme.
import { parseCorner, radii, type Corner } from './corners';

export const CORNERS_KEY = 'marks.corners';

function stored(): Corner {
  try {
    return parseCorner(localStorage.getItem(CORNERS_KEY));
  } catch {
    return parseCorner(null);
  }
}

export const corners = $state({ value: stored() });

/** Inline on `<html>`: app.css keeps the defaults. */
function apply(corner: Corner): void {
  const { md, sm } = radii(corner);
  document.documentElement.style.setProperty('--radius-md', md);
  document.documentElement.style.setProperty('--radius-sm', sm);
}

// At import time, before the first render, so the radius does not flash.
apply(corners.value);

export function setCorner(corner: Corner): void {
  corners.value = corner;
  try {
    localStorage.setItem(CORNERS_KEY, String(corner));
  } catch {
    // Storage unavailable: the choice still applies until the app closes.
  }
  apply(corner);
}
