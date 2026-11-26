// Closes a floating panel on a press outside it, as an attachment:
// `{@attach dismissOnOutside(close, () => trigger)}`.
import type { Attachment } from 'svelte/attachments';

/** Calls `onoutside` on a `pointerdown` outside the element and outside every `ignore` element (its trigger). */
export function dismissOnOutside(
  onoutside: () => void,
  ...ignore: (() => Element | null)[]
): Attachment {
  return (element) => {
    const onpointerdown = (event: Event) => {
      const target = event.target;
      if (!(target instanceof Node)) return;
      if (element.contains(target) || ignore.some((get) => get()?.contains(target))) return;
      onoutside();
    };
    // Capture, so a control that stops the event on its way still closes the panel.
    document.addEventListener('pointerdown', onpointerdown, true);
    return () => document.removeEventListener('pointerdown', onpointerdown, true);
  };
}

/** Whether a pointer event lands on the dialog itself (its `::backdrop`) and outside its box. */
function onBackdrop(dialog: Element, event: MouseEvent): boolean {
  if (event.target !== dialog) return false;
  const box = dialog.getBoundingClientRect();
  const inside =
    box.left <= event.clientX &&
    event.clientX <= box.left + box.width &&
    box.top <= event.clientY &&
    event.clientY <= box.top + box.height;
  return !inside;
}

/**
 * Light dismiss for a modal `<dialog>`: `onclose` on a click on its backdrop. The fallback of the
 * web guidance, since Safari has no `closedby`. Both `pointerdown` and `click` must hit the
 * backdrop, so a text selection that started in a field and ends outside does not close it.
 */
export function lightDismiss(onclose: () => void): Attachment {
  return (element) => {
    let downOnBackdrop = false;
    const onpointerdown = (event: Event) => {
      downOnBackdrop = event instanceof MouseEvent && onBackdrop(element, event);
    };
    const onclick = (event: Event) => {
      if (downOnBackdrop && event instanceof MouseEvent && onBackdrop(element, event)) onclose();
      downOnBackdrop = false;
    };
    element.addEventListener('pointerdown', onpointerdown);
    element.addEventListener('click', onclick);
    return () => {
      element.removeEventListener('pointerdown', onpointerdown);
      element.removeEventListener('click', onclick);
    };
  };
}
