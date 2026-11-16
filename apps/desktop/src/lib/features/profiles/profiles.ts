// Pure rules of profiles. The backend validates the dialog too; those rules only enable the submit button.

/** The first profile: the backend never deletes it. */
export const DEFAULT_PROFILE_ID = 1;

/** Longest profile name, in characters (the backend's limit). */
export const NAME_MAX = 40;
const INITIALS_MAX = 2;

// Characters, not UTF-16 units: an emoji counts as one, like in the backend.
const chars = (text: string): string[] => Array.from(text);

/** "Reading" -> "RE"; "Acme Client" -> "AC"; "" -> "". */
export function suggestInitials(name: string): string {
  const words = name.trim().split(/\s+/).filter(Boolean);
  const letters =
    words.length > 1
      ? words.slice(0, INITIALS_MAX).map((word) => chars(word)[0])
      : chars(words[0] ?? '');
  return letters.slice(0, INITIALS_MAX).join('').toUpperCase();
}

/** A trimmed name of 1 to `NAME_MAX` characters and trimmed initials of 1 or 2 once uppercased. */
export function validProfile(name: string, initials: string): boolean {
  const nameLength = chars(name.trim()).length;
  const initialsLength = chars(initials.trim().toUpperCase()).length;
  return (
    nameLength >= 1 &&
    nameLength <= NAME_MAX &&
    initialsLength >= 1 &&
    initialsLength <= INITIALS_MAX
  );
}

/** A bookmark in a profile; ids repeat across profiles, so the id alone names nothing. */
export interface BookmarkRef {
  id: number;
  profile: number;
}

/** `bookmark` is the one selected on screen: same id and the profile shown is still its own. */
export function isOnScreen(
  bookmark: BookmarkRef,
  screen: { selectedId: number | null; profileId: number },
): boolean {
  return screen.profileId === bookmark.profile && screen.selectedId === bookmark.id;
}

/** The data of a bookmark row's drag: `<profile>:<id>`. */
export const bookmarkDragData = (profile: number, id: number): string => `${profile}:${id}`;

/** Reads `bookmarkDragData`; `null` for anything else. */
export function parseBookmarkDrag(data: string): BookmarkRef | null {
  const match = /^(\d+):(\d+)$/.exec(data);
  return match ? { profile: Number(match[1]), id: Number(match[2]) } : null;
}
