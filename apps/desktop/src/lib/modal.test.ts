import { describe, expect, it } from 'vitest';
import { modal } from './modal';

// Stands in for the two `<dialog>` methods the attachment calls.
function fakeDialog(open: boolean) {
  const calls: string[] = [];
  const dialog = {
    open,
    showModal() {
      calls.push('showModal');
      this.open = true;
    },
    close() {
      calls.push('close');
      this.open = false;
    },
  };
  return { dialog: dialog as unknown as HTMLDialogElement, calls };
}

describe('modal', () => {
  it('opens a closed dialog after onopen, and leaves an open one alone', () => {
    const { dialog, calls } = fakeDialog(false);
    const onopen = () => void calls.push('onopen');
    modal(true, onopen)(dialog);
    modal(true, onopen)(dialog);
    expect(calls).toEqual(['onopen', 'showModal']);
  });

  it('closes an open dialog, and leaves a closed one alone', () => {
    const { dialog, calls } = fakeDialog(true);
    modal(false)(dialog);
    modal(false)(dialog);
    expect(calls).toEqual(['close']);
  });
});
