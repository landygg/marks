// Pure logic behind the sidebar: what is selected and how tags fall into groups.
import type { Bookmark, TagGroup } from '../../api';

/** What the list shows: everything, one category or one tag. */
export type Selection =
  { kind: 'all' } | { kind: 'category'; value: string } | { kind: 'tag'; value: string };

export const ALL: Selection = { kind: 'all' };

/** A group as the sidebar draws it; `id: null` is the trailing group of ungrouped tags. */
export interface SidebarGroup {
  id: number | null;
  name: string;
  tags: string[];
}

/** User groups in order (kept even when empty), then the ungrouped tags; only tags that exist on some bookmark. */
export function sidebarGroups(
  tags: string[],
  groups: TagGroup[],
  ungroupedName: string,
): SidebarGroup[] {
  const existing = new Set(tags);
  const grouped = new Set(groups.flatMap((group) => group.tags));
  return [
    ...groups.map(({ id, name, tags: members }) => ({
      id,
      name,
      tags: members.filter((tag) => existing.has(tag)),
    })),
    { id: null, name: ungroupedName, tags: tags.filter((tag) => !grouped.has(tag)) },
  ];
}

/** `selection` if it still names an existing category or tag, else `ALL`. */
export function validSelection(
  selection: Selection,
  categories: string[],
  tags: string[],
): Selection {
  if (selection.kind === 'all') return selection;
  const pool = selection.kind === 'category' ? categories : tags;
  return pool.includes(selection.value) ? selection : ALL;
}

export function matches(
  selection: Selection,
  bookmark: Pick<Bookmark, 'category' | 'tags'>,
): boolean {
  if (selection.kind === 'all') return true;
  if (selection.kind === 'category') return bookmark.category === selection.value;
  return bookmark.tags.includes(selection.value);
}

export function sameSelection(a: Selection, b: Selection): boolean {
  if (a.kind === 'all' || b.kind === 'all') return a.kind === b.kind;
  return a.kind === b.kind && a.value === b.value;
}
