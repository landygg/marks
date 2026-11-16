import { describe, expect, it } from 'vitest';
import design from '../../../../../../DESIGN.md?raw';
import tauriConf from '../../../../src-tauri/tauri.conf.json?raw';
import {
  DEFAULT_LAYOUT,
  DETAIL_MIN,
  GAP,
  LIST,
  RAIL,
  SIDEBAR,
  fit,
  parseLayout,
  room,
} from './layout';

const frontMatter = design.split('---')[1];
const token = (name: string): number => {
  const match = frontMatter.match(new RegExp(`^ {2}${name}: "?(\\d+)px"?$`, 'm'));
  if (!match) throw new Error(`no ${name} token in DESIGN.md`);
  return Number(match[1]);
};

describe('layout', () => {
  it('bounds and defaults are the DESIGN.md tokens', () => {
    expect(SIDEBAR).toEqual({
      min: token('sidebar-min-width'),
      max: token('sidebar-max-width'),
      reset: token('sidebar-width'),
    });
    expect(LIST).toEqual({
      min: token('list-min-width'),
      max: token('list-max-width'),
      reset: token('list-width'),
    });
    expect(RAIL).toBe(token('rail-width'));
    expect(GAP).toBe(token('panel-gap'));
    expect(DETAIL_MIN).toBe(token('detail-min-width'));
  });

  it('the window minimum fits the defaults', () => {
    const sum = RAIL + SIDEBAR.reset + LIST.reset + DETAIL_MIN + 3 * GAP;
    expect(sum).toBe(1032);
    expect(sum).toBe(token('window-min-width'));
    expect(JSON.parse(tauriConf).app.windows[0].minWidth).toBe(sum);
  });

  it('parseLayout falls back to the defaults', () => {
    for (const stored of [null, '{', 'null', '[]', '"x"'])
      expect(parseLayout(stored)).toEqual(DEFAULT_LAYOUT);
  });

  it('parseLayout clamps and rounds', () => {
    expect(parseLayout('{"sidebar":"wide","list":9999,"collapsed":"yes"}')).toEqual({
      sidebar: 200,
      list: 520,
      collapsed: false,
    });
    expect(parseLayout('{"sidebar":100.6,"list":300.4,"collapsed":true}')).toEqual({
      sidebar: 160,
      list: 300,
      collapsed: true,
    });
  });

  it('fit keeps the saved widths when they fit', () => {
    expect(fit(DEFAULT_LAYOUT, 1032, true)).toEqual({ sidebar: 200, list: 340 });
  });

  it('fit shrinks the list before the sidebar', () => {
    expect(fit({ sidebar: 320, list: 520, collapsed: false }, 1032, true)).toEqual({
      sidebar: 260,
      list: 280,
    });
  });

  it('fit does not change the saved layout', () => {
    const layout = { sidebar: 320, list: 520, collapsed: false };
    const copy = { ...layout };
    fit(layout, 1032, true);
    expect(layout).toEqual(copy);
  });

  it('fit with the sidebar collapsed or the list hidden', () => {
    expect(fit({ sidebar: 320, list: 520, collapsed: true }, 1032, true)).toEqual({
      sidebar: 260,
      list: 520,
    });
    expect(fit({ sidebar: 320, list: 520, collapsed: false }, 1032, false)).toEqual({
      sidebar: 320,
      list: 520,
    });
  });

  it('fit draws the collapsed sidebar at its open width, so a fold does not reflow it', () => {
    for (const windowWidth of [1032, 1100, 1400])
      for (const listShown of [true, false]) {
        const open = { sidebar: 320, list: 520, collapsed: false };
        expect(fit({ ...open, collapsed: true }, windowWidth, listShown).sidebar).toBe(
          fit(open, windowWidth, listShown).sidebar,
        );
      }
  });

  it('room', () => {
    expect(room('sidebar', DEFAULT_LAYOUT, 1032, true)).toBe(200);
    expect(room('list', DEFAULT_LAYOUT, 1032, true)).toBe(340);
    expect(room('sidebar', DEFAULT_LAYOUT, 1400, true)).toBe(320);
    expect(room('list', { sidebar: 320, list: 520, collapsed: false }, 1032, true)).toBe(280);
    // The other pane counts as drawn, not as saved.
    expect(room('sidebar', { sidebar: 320, list: 520, collapsed: false }, 1032, true)).toBe(260);
  });
});
