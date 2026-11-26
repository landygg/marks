// @vitest-environment jsdom
import { createRawSnippet, flushSync, mount, unmount } from 'svelte';
import { afterEach, describe, expect, it } from 'vitest';
import palette from '../features/palette/CommandPalette.svelte?raw';
import CommandItem from './CommandItem.svelte';

let view: ReturnType<typeof mount> | null = null;

afterEach(() => {
  if (view) unmount(view);
  view = null;
  document.body.replaceChildren();
});

describe('CommandItem', () => {
  it('is an option outside the Tab order: the focus stays in the combobox', () => {
    const children = createRawSnippet(() => ({ render: () => '<span>x</span>' }));
    view = mount(CommandItem, { target: document.body, props: { children } });
    flushSync();
    const button = document.body.querySelector('button');
    expect(button?.getAttribute('role')).toBe('option');
    expect(button?.getAttribute('tabindex')).toBe('-1');
  });
});

// Mounting the palette pulls in its state and the IPC; its markup is checked as source instead.
describe('CommandPalette listbox', () => {
  const listbox = palette.slice(palette.indexOf('role="listbox"'));
  const inside = listbox.slice(0, listbox.indexOf('</div>'));

  it('holds only options', () => {
    expect(inside).not.toMatch(/<p\b/);
  });

  it('says "no results" and errors in a status region after it', () => {
    expect(listbox).toMatch(/<p\b[^>]*role="status"/);
    expect(inside).not.toContain('role="status"');
  });
});
