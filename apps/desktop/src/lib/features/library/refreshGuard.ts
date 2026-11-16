// Which refresh results may be applied, and what survives a refresh. Bookmark and group ids repeat
// across profiles, so anything loaded or kept across a profile switch could name the wrong row.

export interface RefreshGuard {
  /** A refresh starts: returns its ticket. */
  start(): number;
  /** A profile switch starts: every refresh already running, or started before it ends, is stale. */
  beginSwitch(): void;
  endSwitch(): void;
  /** The refresh with this ticket may apply its results: nothing newer started and no switch is in flight. */
  current(ticket: number): boolean;
}

export function createRefreshGuard(): RefreshGuard {
  let generation = 0;
  let switching = 0;
  return {
    start: () => ++generation,
    beginSwitch: () => {
      generation += 1;
      switching += 1;
    },
    endSwitch: () => {
      switching -= 1;
    },
    current: (ticket) => ticket === generation && switching === 0,
  };
}

/**
 * `load`, one run at a time: a call while it runs asks for one more run after it, which serves every
 * call made meanwhile, and resolves when that run ends. Starting a run per call instead, with the
 * guard dropping each overtaken one, applied nothing while events (one per processed bookmark)
 * came faster than a run took: a new profile stayed off the rail. `load` handles its own errors.
 */
export function coalesce(load: () => Promise<void>): () => Promise<void> {
  let running: Promise<void> | null = null;
  let again = false;
  return () => {
    if (running) {
      again = true;
      return running;
    }
    running = (async () => {
      try {
        do {
          again = false;
          await load();
        } while (again);
      } finally {
        running = null;
      }
    })();
    return running;
  };
}

/** `id` if it still names a loaded row of the same profile, else `null`. */
export function kept(
  id: number | null,
  ids: number[],
  previousProfile: number,
  loadedProfile: number,
): number | null {
  return previousProfile === loadedProfile && id !== null && ids.includes(id) ? id : null;
}

// Arrays (a bookmark's tags) compare item by item; everything else by value.
const sameValue = (a: unknown, b: unknown): boolean =>
  Array.isArray(a) && Array.isArray(b)
    ? a.length === b.length && a.every((item, i) => item === b[i])
    : a === b;

/**
 * `next`, with each row whose fields all equal its previous copy's replaced by that copy: keyed
 * `{#each}` and `$derived` then see the same object and skip it, instead of redoing every row on
 * every refresh. Every field counts, so a field added in Rust needs nothing here.
 */
export function keepUnchanged<T extends { id: number }>(previous: T[], next: T[]): T[] {
  const byId = new Map(previous.map((row) => [row.id, row]));
  return next.map((row) => {
    const old = byId.get(row.id);
    if (old === undefined) return row;
    const fields = Object.entries(row);
    const same =
      fields.length === Object.keys(old).length &&
      fields.every(([key, value]) => sameValue((old as Record<string, unknown>)[key], value));
    return same ? old : row;
  });
}
