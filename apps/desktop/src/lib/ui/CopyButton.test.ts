// @vitest-environment jsdom
import { flushSync, mount, tick, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import CopyButton from './CopyButton.svelte';

let target: HTMLElement;
let view: ReturnType<typeof mount> | null = null;
const writeText = vi.fn<(text: string) => Promise<void>>();

beforeEach(() => {
  vi.useFakeTimers();
  target = document.createElement('div');
  document.body.append(target);
  Object.defineProperty(navigator, 'clipboard', { value: { writeText }, configurable: true });
  writeText.mockReset().mockResolvedValue(undefined);
  view = mount(CopyButton, {
    target,
    props: {
      text: 'https://example.com/very/long',
      label: 'Copy link',
      copiedLabel: 'Link copied',
    },
  });
  flushSync();
});

afterEach(() => {
  if (view) unmount(view);
  view = null;
  target.remove();
  vi.useRealTimers();
});

const button = (): HTMLButtonElement | null => target.querySelector('button');
const status = (): string => target.querySelector('[role=status]')?.textContent ?? '';

describe('CopyButton', () => {
  it('copies the whole text, confirms and announces it, then goes back', async () => {
    button()?.click();
    await tick();
    flushSync();
    expect(writeText).toHaveBeenCalledWith('https://example.com/very/long');
    expect(button()?.getAttribute('aria-label')).toBe('Link copied');
    expect(status()).toBe('Link copied');

    vi.advanceTimersByTime(1500);
    flushSync();
    expect(button()?.getAttribute('aria-label')).toBe('Copy link');
    expect(status()).toBe('');
  });

  it('changes nothing when the clipboard refuses', async () => {
    writeText.mockRejectedValue(new Error('denied'));
    button()?.click();
    await tick();
    flushSync();
    expect(button()?.getAttribute('aria-label')).toBe('Copy link');
    expect(status()).toBe('');
  });
});
