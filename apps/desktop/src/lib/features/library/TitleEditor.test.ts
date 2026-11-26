// @vitest-environment jsdom
import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import TitleEditor from './TitleEditor.svelte';

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
  view = mount(TitleEditor, { target, props: { profile: 1, id: 7, saved: 'Title' } });
  flushSync();
});

afterEach(() => {
  if (view) unmount(view);
  view = null;
  target.remove();
});

describe('TitleEditor', () => {
  it('keeps the focus on the Enter that confirms an input method candidate', () => {
    const input = target.querySelector('input') as HTMLInputElement;
    input.focus();
    input.dispatchEvent(enter(229));
    expect(document.activeElement).toBe(input);
  });

  it('leaves the field on a plain Enter', () => {
    const input = target.querySelector('input') as HTMLInputElement;
    input.focus();
    input.dispatchEvent(enter(13));
    expect(document.activeElement).not.toBe(input);
  });
});
