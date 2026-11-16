// Inline icons: stroke-only shapes on a 24×24 viewBox, drawn by Icon.svelte with `currentColor`.
export interface IconShape {
  paths: string[];
  /** Optional circle as [cx, cy, r]. */
  circle?: [number, number, number];
}

const GEAR_TEETH =
  'M19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1.1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1.1 1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z';

export const ICONS = {
  sidebar: {
    paths: [
      'M4.5 4h15A1.5 1.5 0 0 1 21 5.5v13a1.5 1.5 0 0 1-1.5 1.5h-15A1.5 1.5 0 0 1 3 18.5v-13A1.5 1.5 0 0 1 4.5 4z',
      'M9 4v16',
    ],
  },
  search: { paths: ['M16.5 16.5L21 21'], circle: [11, 11, 7] },
  folderPlus: {
    paths: [
      'M3 6.5A1.5 1.5 0 0 1 4.5 5H9l2 2.5h8.5A1.5 1.5 0 0 1 21 9v9.5a1.5 1.5 0 0 1-1.5 1.5h-15A1.5 1.5 0 0 1 3 18.5z',
      'M12 11v5M9.5 13.5h5',
    ],
  },
  command: {
    paths: ['M15 6v12a3 3 0 1 0 3-3H6a3 3 0 1 0 3 3V6a3 3 0 1 0-3 3h12a3 3 0 1 0-3-3'],
  },
  gear: { paths: [GEAR_TEETH], circle: [12, 12, 3] },
  plus: { paths: ['M12 5v14M5 12h14'] },
  expand: { paths: ['M15 3h6v6M9 21H3v-6M21 3l-7 7M3 21l7-7'] },
  collapse: { paths: ['M4 14h6v6M20 10h-6V4M14 10l7-7M3 21l7-7'] },
  external: {
    paths: [
      'M14 4h6v6M20 4l-9 9M18 14v4.5a1.5 1.5 0 0 1-1.5 1.5h-11A1.5 1.5 0 0 1 4 18.5v-11A1.5 1.5 0 0 1 5.5 6H10',
    ],
  },
  retry: {
    paths: [
      'M20 4v4.5h-4.5M4 12a8 8 0 0 1 13.7-5.7L20 8.5M4 20v-4.5h4.5M20 12a8 8 0 0 1-13.7 5.7L4 15.5',
    ],
  },
  // A page with an arrow into it: extract the page again.
  reextract: {
    paths: [
      'M14 3H6.5A1.5 1.5 0 0 0 5 4.5v15A1.5 1.5 0 0 0 6.5 21h11a1.5 1.5 0 0 0 1.5-1.5V8z',
      'M14 3v5h5M12 11v6M9.5 14.5L12 17l2.5-2.5',
    ],
  },
  // Two sparkles: done by the AI.
  sparkles: {
    paths: [
      'M10 3l1.6 4.4L16 9l-4.4 1.6L10 15l-1.6-4.4L4 9l4.4-1.6z',
      'M18 14l.8 2.2 2.2.8-2.2.8L18 20l-.8-2.2L15 17l2.2-.8z',
    ],
  },
  bell: {
    paths: ['M6 16V10a6 6 0 0 1 12 0v6l1.5 2h-15zM10 20.5h4'],
  },
  trash: {
    paths: [
      'M4 7h16M9 7V4.5h6V7M6.5 7l.8 12a1.5 1.5 0 0 0 1.5 1.5h6.4a1.5 1.5 0 0 0 1.5-1.5l.8-12M10 11v6M14 11v6',
    ],
  },
  // Six dots: the drag handle of a tag.
  grip: { paths: ['M9 6h.01M15 6h.01M9 12h.01M15 12h.01M9 18h.01M15 18h.01'] },
  check: { paths: ['M5 12.5l4.5 4.5L19 7.5'] },
  // Two sheets: copy to the clipboard.
  copy: {
    paths: [
      'M9 9h10.5A1.5 1.5 0 0 1 21 10.5v9a1.5 1.5 0 0 1-1.5 1.5h-9A1.5 1.5 0 0 1 9 19.5z',
      'M15 9V4.5A1.5 1.5 0 0 0 13.5 3h-9A1.5 1.5 0 0 0 3 4.5v9A1.5 1.5 0 0 0 4.5 15H9',
    ],
  },
  close: { paths: ['M6 6l12 12M18 6L6 18'] },
  chevronDown: { paths: ['M6 9l6 6 6-6'] },
  chevronRight: { paths: ['M9 6l6 6-6 6'] },
  chevronLeft: { paths: ['M15 6l-6 6 6 6'] },
} as const satisfies Record<string, IconShape>;

export type IconName = keyof typeof ICONS;
