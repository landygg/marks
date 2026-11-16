// Keeps a `<dialog>` modal in step with its open state, as an attachment: `{@attach modal(open)}`.
// The expression runs again when `open` changes, and with it the attachment.

/** Opens the dialog as a modal when `open` turns true (after `onopen`) and closes it when it turns false. */
export const modal =
  (open: boolean, onopen?: () => void) =>
  (dialog: HTMLDialogElement): void => {
    if (open && !dialog.open) {
      onopen?.();
      // `showModal` focuses the autofocus field; esc arrives as the `close` event.
      dialog.showModal();
    } else if (!open && dialog.open) {
      dialog.close();
    }
  };
