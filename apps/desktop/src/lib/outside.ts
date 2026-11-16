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
