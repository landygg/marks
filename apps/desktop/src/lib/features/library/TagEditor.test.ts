// @vitest-environment jsdom
import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import TagEditor from './TagEditor.svelte';
import { saveBookmark } from './libraryState.svelte';

vi.mock('./libraryState.svelte', () => ({ saveBookmark: vi.fn(async () => true) }));

/** A keydown Enter; `keyCode` 229 is the one an input method sends while it composes. */
function enter(keyCode: number): KeyboardEvent {
  const event = new KeyboardEvent('keydown', { key: 'Enter', bubbles: true });
  Object.defineProperty(event, 'keyCode', { value: keyCode });
  return event;
}

let target: HTMLElement;
let view: ReturnType<typeof mount> | null = null;

beforeEach(() => {
  target = document.createElement('div');
  document.body.append(target);
  vi.mocked(saveBookmark).mockClear();
  view = mount(TagEditor, { target, props: { profile: 1, id: 7, tags: [] } });
  flushSync();
});

afterEach(() => {
  if (view) unmount(view);
  view = null;
  target.remove();
});

function type(text: string): HTMLInputElement {
  const input = target.querySelector('input') as HTMLInputElement;
  input.value = text;
  input.dispatchEvent(new Event('input', { bubbles: true }));
  flushSync();
  return input;
}

describe('TagEditor', () => {
  it('ignores the Enter that confirms an input method candidate', () => {
    type('foo').dispatchEvent(enter(229));
    expect(saveBookmark).not.toHaveBeenCalled();
  });

  it('adds the tags on a plain Enter', () => {
    type('foo').dispatchEvent(enter(13));
    expect(saveBookmark).toHaveBeenCalledWith(1, 7, { tags: ['foo'] });
  });
});
