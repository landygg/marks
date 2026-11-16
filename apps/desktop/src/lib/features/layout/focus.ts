/** Moves focus to `target` (or drops it) when it is inside `container`, so hiding it does not lose it. */
export function rescueFocus(container: Element | null, target: HTMLElement | null): void {
  const active = document.activeElement;
  if (!container || !active || !container.contains(active)) return;
  if (target) target.focus();
  else if (active instanceof HTMLElement) active.blur();
}
