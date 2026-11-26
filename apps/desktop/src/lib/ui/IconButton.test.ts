// @vitest-environment jsdom
import { flushSync, mount, unmount } from 'svelte';
import { afterEach, describe, expect, it, vi } from 'vitest';
import IconButton from './IconButton.svelte';

let view: ReturnType<typeof mount> | null = null;

afterEach(() => {
  if (view) unmount(view);
  view = null;
  document.body.replaceChildren();
});

function button(reason?: string) {
  const onclick = vi.fn();
  view = mount(IconButton, {
    target: document.body,
    props: { icon: 'sparkles', label: 'Extract', reason, onclick },
  });
  flushSync();
  return { el: document.body.querySelector('button') as HTMLButtonElement, onclick };
}

describe('IconButton with a reason', () => {
  it('stays focusable and tells why it does nothing', () => {
    const { el, onclick } = button('No model');
    expect(el.disabled).toBe(false);
    expect(el.getAttribute('aria-disabled')).toBe('true');
    const described = document.getElementById(el.getAttribute('aria-describedby') ?? '');
    expect(described?.textContent).toBe('No model');
    expect(el.title).toBe('No model');
    el.click();
    expect(onclick).not.toHaveBeenCalled();
  });
});

describe('IconButton without a reason', () => {
  it('is a plain button', () => {
    const { el, onclick } = button();
    expect(el.hasAttribute('aria-disabled')).toBe(false);
    expect(el.hasAttribute('aria-describedby')).toBe(false);
    expect(el.title).toBe('Extract');
    el.click();
    expect(onclick).toHaveBeenCalledOnce();
  });
});
