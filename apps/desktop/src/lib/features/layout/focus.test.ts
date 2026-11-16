// @vitest-environment jsdom
import { afterEach, describe, expect, it } from 'vitest';
import { rescueFocus } from './focus';

function setup() {
  document.body.innerHTML = '<div id="c"><button id="in"></button></div><button id="out"></button>';
  const get = (id: string) => document.getElementById(id) as HTMLElement;
  return { container: get('c'), inside: get('in'), outside: get('out') };
}

afterEach(() => {
  document.body.innerHTML = '';
});

describe('rescueFocus', () => {
  it('rescueFocus moves focus out of the container', () => {
    const { container, inside, outside } = setup();
    inside.focus();
    rescueFocus(container, outside);
    expect(document.activeElement).toBe(outside);
  });

  it('rescueFocus leaves focus elsewhere alone', () => {
    const { container, inside, outside } = setup();
    outside.focus();
    rescueFocus(container, inside);
    expect(document.activeElement).toBe(outside);
  });

  it('rescueFocus blurs without a target', () => {
    const { container, inside } = setup();
    inside.focus();
    rescueFocus(container, null);
    expect(document.activeElement).toBe(document.body);
  });
});
