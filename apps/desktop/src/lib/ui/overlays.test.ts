// @vitest-environment jsdom
// How the notification panel and an inline question share Esc and the focus with the window.
import { createRawSnippet, flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import InlineConfirm from './InlineConfirm.svelte';
import NotificationPanel from './NotificationPanel.svelte';

const views: ReturnType<typeof mount>[] = [];

// jsdom has no layout: InlineConfirm scrolls itself into view when it appears.
beforeAll(() => {
  Element.prototype.scrollIntoView = vi.fn();
});

function host(): HTMLElement {
  const target = document.createElement('div');
  document.body.append(target);
  return target;
}

afterEach(() => {
  views.splice(0).forEach((view) => unmount(view));
  document.body.replaceChildren();
});

describe('overlays', () => {
  it('an_inline_question_keeps_its_esc_from_the_window', () => {
    const oncancel = vi.fn();
    const target = host();
    const props = {
      question: '¿Eliminar?',
      label: 'Eliminar',
      yes: 'Eliminar',
      cancel: 'Cancelar',
      onyes: () => {},
      oncancel,
    };
    views.push(mount(InlineConfirm, { target, props }));
    flushSync();
    const esc = new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true });
    target.querySelector('[role="alertdialog"]')?.dispatchEvent(esc);
    expect(oncancel).toHaveBeenCalledOnce();
    // App.svelte closes the notification panel only on an Esc nobody prevented.
    expect(esc.defaultPrevented).toBe(true);
  });

  it('an_inline_question_is_neutral_only_when_asked', () => {
    const answers = (tone?: 'neutral') => {
      const target = host();
      const props = {
        question: '¿Extraer?',
        label: 'Extraer',
        yes: 'Extraer',
        cancel: 'Cancelar',
        onyes: () => {},
        oncancel: () => {},
        tone,
      };
      views.push(mount(InlineConfirm, { target, props }));
      flushSync();
      return target.querySelector('button:not([data-cancel])')?.classList[0];
    };
    expect(answers()).toBe('danger');
    expect(answers('neutral')).toBe('secondary');
  });

  it('the_panel_takes_the_focus_when_it_appears', () => {
    const target = host();
    const props = {
      id: 'panel',
      label: 'Notificaciones',
      heading: 'Notificaciones',
      empty: 'Vacío',
      actions: createRawSnippet(() => ({ render: () => '<span></span>' })),
    };
    views.push(mount(NotificationPanel, { target, props }));
    flushSync();
    expect(document.activeElement).toBe(target.querySelector('section#panel'));
  });
});
