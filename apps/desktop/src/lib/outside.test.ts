// @vitest-environment jsdom
import { describe, expect, it, vi } from 'vitest';
import { dismissOnOutside, lightDismiss } from './outside';

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

describe('lightDismiss', () => {
  function setup() {
    const dialog = document.createElement('dialog');
    const field = document.createElement('input');
    dialog.append(field);
    document.body.append(dialog);
    dialog.getBoundingClientRect = () =>
      ({ left: 100, top: 100, width: 200, height: 100, right: 300, bottom: 200 }) as DOMRect;
    const onclose = vi.fn();
    const detach = lightDismiss(onclose)(dialog);
    const fire = (type: string, target: Element, x: number, y: number) =>
      target.dispatchEvent(new MouseEvent(type, { bubbles: true, clientX: x, clientY: y }));
    const press = (target: Element, x: number, y: number) => {
      fire('pointerdown', target, x, y);
      fire('click', target, x, y);
    };
    return { dialog, field, onclose, detach, fire, press };
  }

  it('closes on a press on the backdrop', () => {
    const { dialog, onclose, press } = setup();
    press(dialog, 10, 10);
    expect(onclose).toHaveBeenCalledOnce();
  });

  it('stays open on a press inside the dialog box, on its padding too', () => {
    const { dialog, field, onclose, press } = setup();
    press(dialog, 150, 150);
    press(field, 150, 150);
    expect(onclose).not.toHaveBeenCalled();
  });

  it('stays open when a press that started inside ends on the backdrop (a text selection)', () => {
    const { dialog, field, onclose, fire } = setup();
    fire('pointerdown', field, 150, 150);
    fire('click', dialog, 10, 10);
    expect(onclose).not.toHaveBeenCalled();
  });

  it('stays open on a click on a child', () => {
    const { dialog, field, onclose, fire } = setup();
    fire('pointerdown', dialog, 10, 10);
    fire('click', field, 10, 10);
    expect(onclose).not.toHaveBeenCalled();
  });

  it('removes its listeners when detached', () => {
    const { dialog, onclose, detach, press } = setup();
    if (typeof detach === 'function') detach();
    press(dialog, 10, 10);
    expect(onclose).not.toHaveBeenCalled();
  });
});
