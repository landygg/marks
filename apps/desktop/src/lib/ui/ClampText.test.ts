// @vitest-environment jsdom
import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import ClampText from './ClampText.svelte';

const TEXT = 'GET https://signin.example-cloud.com/signin?redirect_uri=… io: peer certificate';
const LINE = 16;

let target: HTMLElement;
let view: ReturnType<typeof mount> | null = null;

/** jsdom has no layout: the paragraph is `lines` tall when open, two lines when clamped. */
function fakeLayout(lines: number): void {
  vi.spyOn(HTMLElement.prototype, 'scrollHeight', 'get').mockReturnValue(lines * LINE);
  vi.spyOn(HTMLElement.prototype, 'clientHeight', 'get').mockReturnValue(Math.min(lines, 2) * LINE);
}

function render(): { p: HTMLElement; button: () => HTMLButtonElement | null } {
  view = mount(ClampText, { target, props: { text: TEXT, moreLabel: 'More', lessLabel: 'Less' } });
  flushSync();
  const p = target.querySelector('p');
  if (!p) throw new Error('no paragraph');
  return { p, button: () => target.querySelector('button') };
}

beforeEach(() => {
  target = document.createElement('div');
  document.body.append(target);
  vi.stubGlobal(
    'ResizeObserver',
    class {
      observe(): void {}
      disconnect(): void {}
    },
  );
});

afterEach(() => {
  if (view) unmount(view);
  view = null;
  target.remove();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

describe('ClampText', () => {
  it('offers nothing to expand when the text fits', () => {
    fakeLayout(1);
    const { p, button } = render();
    expect(button()).toBeNull();
    expect(p.textContent).toBe(TEXT);
  });

  it('toggles between clamped and whole when the text is cut', () => {
    fakeLayout(6);
    const { p, button } = render();
    const toggle = button();
    expect(toggle?.textContent).toBe('More');
    expect(toggle?.getAttribute('aria-expanded')).toBe('false');
    expect(toggle?.getAttribute('aria-controls')).toBe(p.id);
    expect(p.classList.contains('open')).toBe(false);

    toggle?.click();
    flushSync();
    expect(button()?.textContent).toBe('Less');
    expect(button()?.getAttribute('aria-expanded')).toBe('true');
    expect(p.classList.contains('open')).toBe(true);

    button()?.click();
    flushSync();
    expect(button()?.textContent).toBe('More');
    expect(p.classList.contains('open')).toBe(false);
  });

  it('keeps the whole text in the page for screen readers while clamped', () => {
    fakeLayout(6);
    const { p } = render();
    expect(p.textContent).toBe(TEXT);
  });
});
