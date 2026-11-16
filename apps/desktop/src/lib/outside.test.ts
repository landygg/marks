// @vitest-environment jsdom
import { describe, expect, it, vi } from 'vitest';
import { dismissOnOutside } from './outside';

describe('dismissOnOutside', () => {
  it('outside_click_calls_back_but_not_inside_or_ignored', () => {
    const panel = document.createElement('div');
    const child = document.createElement('button');
    const bell = document.createElement('button');
    const elsewhere = document.createElement('p');
    panel.append(child);
    document.body.append(panel, bell, elsewhere);
    const onoutside = vi.fn();
    const detach = dismissOnOutside(onoutside, () => bell)(panel);

    const press = (target: Element) =>
      target.dispatchEvent(new Event('pointerdown', { bubbles: true }));
    press(child);
    press(panel);
    press(bell);
    expect(onoutside).not.toHaveBeenCalled();
    press(elsewhere);
    expect(onoutside).toHaveBeenCalledOnce();

    expect(typeof detach).toBe('function');
    if (typeof detach === 'function') detach();
    press(elsewhere);
    expect(onoutside).toHaveBeenCalledOnce();
  });

  it('an_ignored_element_that_is_missing_does_not_count', () => {
    const panel = document.createElement('div');
    document.body.append(panel);
    const onoutside = vi.fn();
    dismissOnOutside(onoutside, () => null)(panel);
    document.body.dispatchEvent(new Event('pointerdown', { bubbles: true }));
    expect(onoutside).toHaveBeenCalledOnce();
  });
});
