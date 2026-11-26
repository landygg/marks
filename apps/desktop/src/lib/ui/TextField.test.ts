// @vitest-environment jsdom
import { flushSync, mount, unmount, type Component } from 'svelte';
import { afterEach, describe, expect, it } from 'vitest';
import TagField from './TagField.svelte';
import TextField from './TextField.svelte';

// What WebKit on macOS honours to leave a tag, URL or name as typed.
const IDENTIFIER = { autocorrect: 'off', autocapitalize: 'none', spellcheck: 'false' };

let view: ReturnType<typeof mount> | null = null;

afterEach(() => {
  if (view) unmount(view);
  view = null;
  document.body.replaceChildren();
});

function input(component: Component<{ identifier?: boolean }>, identifier?: boolean) {
  view = mount(component, { target: document.body, props: { identifier } });
  flushSync();
  return document.body.querySelector('input') as HTMLInputElement;
}

describe.each([
  ['TextField', TextField],
  ['TagField', TagField],
] as const)('%s', (_, component) => {
  it('turns off autocorrect, capitals and spelling with identifier', () => {
    const field = input(component as Component<{ identifier?: boolean }>, true);
    for (const [name, value] of Object.entries(IDENTIFIER)) {
      expect(field.getAttribute(name)).toBe(value);
    }
  });

  it('leaves them to the system without it', () => {
    const field = input(component as Component<{ identifier?: boolean }>);
    for (const name of Object.keys(IDENTIFIER)) expect(field.hasAttribute(name)).toBe(false);
  });
});
