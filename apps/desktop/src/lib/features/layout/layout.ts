export interface Bounds {
  min: number;
  max: number;
  reset: number;
}

export const SIDEBAR: Bounds = { min: 160, max: 320, reset: 200 };
export const LIST: Bounds = { min: 280, max: 520, reset: 340 };
export const RAIL = 48;
export const GAP = 8;
export const DETAIL_MIN = 420;
export const SIDEBAR_ID = 'sidebar';
export const LIST_ID = 'list';

export type Pane = 'sidebar' | 'list';
export interface Layout {
  sidebar: number;
  list: number;
  collapsed: boolean;
}

export const DEFAULT_LAYOUT: Layout = {
  sidebar: SIDEBAR.reset,
  list: LIST.reset,
  collapsed: false,
};

const clamp = (value: number, { min, max }: Bounds): number => Math.min(max, Math.max(min, value));

const width = (value: unknown, bounds: Bounds): number =>
  typeof value === 'number' && Number.isFinite(value)
    ? clamp(Math.round(value), bounds)
    : bounds.reset;

/** Each field is taken on its own: anything unreadable falls back to its default. */
export function parseLayout(stored: string | null): Layout {
  let raw: unknown = null;
  try {
    raw = stored === null ? null : JSON.parse(stored);
  } catch {
    // Unreadable JSON: use the defaults.
  }
  const fields = (typeof raw === 'object' && raw !== null ? raw : {}) as Record<string, unknown>;
  return {
    sidebar: width(fields.sidebar, SIDEBAR),
    list: width(fields.list, LIST),
    collapsed: fields.collapsed === true,
  };
}

/** Space the sidebar and the list can share: what the rail, the gaps and the detail leave. */
function free(layout: Layout, windowWidth: number, listShown: boolean): number {
  const gaps = 1 + (layout.collapsed ? 0 : 1) + (listShown ? 1 : 0);
  return windowWidth - RAIL - DETAIL_MIN - GAP * gaps;
}

/** Widths to draw. The list shrinks first, then the sidebar; `layout` is not changed. */
export function fit(
  layout: Layout,
  windowWidth: number,
  listShown: boolean,
): { sidebar: number; list: number } {
  const space = free(layout, windowWidth, listShown);
  const sidebarUsed = layout.collapsed ? 0 : layout.sidebar;
  const list = listShown
    ? Math.min(layout.list, Math.max(LIST.min, space - sidebarUsed))
    : layout.list;
  const listUsed = listShown ? list : 0;
  // Folded, the sidebar keeps the width it has open, so its content does not reflow during a fold.
  const sidebar = layout.collapsed
    ? fit({ ...layout, collapsed: false }, windowWidth, listShown).sidebar
    : Math.min(layout.sidebar, Math.max(SIDEBAR.min, space - listUsed));
  return { sidebar, list };
}

/** The widest `pane` can be now without the detail going under DETAIL_MIN, within its bounds. */
export function room(pane: Pane, layout: Layout, windowWidth: number, listShown: boolean): number {
  const space = free(layout, windowWidth, listShown);
  const drawn = fit(layout, windowWidth, listShown);
  const other =
    pane === 'sidebar' ? (listShown ? drawn.list : 0) : layout.collapsed ? 0 : drawn.sidebar;
  return clamp(space - other, pane === 'sidebar' ? SIDEBAR : LIST);
}
