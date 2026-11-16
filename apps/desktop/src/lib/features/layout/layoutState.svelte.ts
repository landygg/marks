// The panel widths and whether the sidebar is folded. They are preferences of this window, so they
// live in the webview's localStorage, like the theme.
import { LIST, SIDEBAR, SIDEBAR_ID, parseLayout, type Layout, type Pane } from './layout';
import { rescueFocus } from './focus';

export const LAYOUT_KEY = 'marks.layout';
const TOGGLE_SELECTOR = '[data-sidebar-toggle]';
const BOUNDS = { sidebar: SIDEBAR, list: LIST };

function stored(): Layout {
  try {
    return parseLayout(localStorage.getItem(LAYOUT_KEY));
  } catch {
    return parseLayout(null);
  }
}

export const layout: Layout = $state(stored());

/** How long a fold or unfold animates; App.svelte hands it to the CSS as --fold. */
export const FOLD_MS = 180;
/** A fold is under way: only then do the columns animate, so other width changes snap. */
export const fold = $state({ active: false });
let foldTimer: ReturnType<typeof setTimeout> | undefined;

function save(): void {
  try {
    localStorage.setItem(LAYOUT_KEY, JSON.stringify(layout));
  } catch {
    // Storage unavailable: the layout still applies until the app closes.
  }
}

export function setWidth(pane: Pane, px: number): void {
  const { min, max } = BOUNDS[pane];
  layout[pane] = Math.min(max, Math.max(min, Math.round(px)));
  save();
}

export function resetWidth(pane: Pane): void {
  layout[pane] = BOUNDS[pane].reset;
  save();
}

export function toggleSidebar(): void {
  if (!layout.collapsed) {
    rescueFocus(document.getElementById(SIDEBAR_ID), document.querySelector(TOGGLE_SELECTOR));
  }
  layout.collapsed = !layout.collapsed;
  fold.active = true;
  clearTimeout(foldTimer);
  foldTimer = setTimeout(() => (fold.active = false), FOLD_MS);
  save();
}
