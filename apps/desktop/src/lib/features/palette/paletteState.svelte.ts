// State of the command palette: open or not, its mode, the query and the bookmark search hits.
import { search, type SearchHit } from '../../api';
import { report } from '../library/libraryState.svelte';
import type { PaletteMode } from './commands';

const SEARCH_DEBOUNCE_MS = 250;

export const palette = $state({
  open: false,
  mode: 'commands' as PaletteMode,
  query: '',
  hits: [] as SearchHit[],
  /** A bookmark search is pending (debounce or request): `hits` still belong to an older query. */
  searching: false,
  /** Why the last search failed; shown inside the palette, which covers the notifications behind it. */
  error: null as string | null,
  /** Selected row. */
  index: 0,
});

let timer: ReturnType<typeof setTimeout> | undefined;

/** Searches for the current query; responses for an outdated query or mode are ignored. */
async function runSearch(): Promise<void> {
  const query = palette.query.trim();
  try {
    const hits = await search(query);
    if (!isCurrent(query)) return;
    palette.hits = hits;
    palette.index = 0;
  } catch (err) {
    if (!isCurrent(query)) return;
    // The rows on screen belong to an older query: they must not stay openable.
    palette.hits = [];
    palette.error = err instanceof Error ? err.message : String(err);
    report(err, { toast: false });
  } finally {
    if (isCurrent(query)) palette.searching = false;
  }
}

const isCurrent = (query: string): boolean =>
  palette.mode === 'bookmarks' && palette.query.trim() === query;

function scheduleSearch(): void {
  clearTimeout(timer);
  palette.error = null;
  palette.searching = palette.mode === 'bookmarks' && palette.query.trim() !== '';
  if (!palette.searching) {
    palette.hits = [];
    return;
  }
  timer = setTimeout(() => void runSearch(), SEARCH_DEBOUNCE_MS);
}

export function openPalette(mode: PaletteMode): void {
  palette.mode = mode;
  palette.query = '';
  palette.hits = [];
  palette.searching = false;
  palette.error = null;
  palette.index = 0;
  palette.open = true;
}

export function closePalette(): void {
  clearTimeout(timer);
  palette.open = false;
}

/** Switches mode keeping the query, which is searched again when the new mode is Bookmarks. */
export function setMode(mode: PaletteMode): void {
  palette.mode = mode;
  palette.index = 0;
  palette.hits = [];
  scheduleSearch();
}

export function setQuery(query: string): void {
  palette.query = query;
  palette.index = 0;
  scheduleSearch();
}
