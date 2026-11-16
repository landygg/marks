<!-- `resize-handle` (DESIGN.md, Components). Positioned against its nearest positioned ancestor. -->
<script lang="ts">
  import { clampTo, keyStep } from './resize';

  interface Props {
    /** Accessible name. */
    label: string;
    value: number;
    min: number;
    max: number;
    reset: number;
    /** Id of the resized element. */
    controls: string;
    onchange: (value: number) => void;
    /** A pointer drag begins. */
    onstart?: () => void;
    /** A pointer drag ends or is cancelled. */
    onend?: () => void;
  }

  let { label, value, min, max, reset, controls, onchange, onstart, onend }: Props = $props();

  let drag: { x: number; value: number } | null = $state.raw(null);

  function down(e: PointerEvent) {
    if (e.button !== 0) return;
    const handle = e.currentTarget as HTMLElement;
    handle.setPointerCapture(e.pointerId);
    e.preventDefault();
    // `preventDefault` also cancels the focus a press gives; the arrow keys must work after a click.
    handle.focus();
    drag = { x: e.clientX, value };
    onstart?.();
  }

  function move(e: PointerEvent) {
    if (drag) onchange(clampTo(drag.value + e.clientX - drag.x, min, max));
  }

  function up() {
    if (!drag) return;
    drag = null;
    onend?.();
  }

  // Unmounted mid-drag (the panel is hidden): the consumer still has to hear the drag end.
  $effect(() => up);

  function key(e: KeyboardEvent) {
    const next = keyStep(e.key, e.shiftKey, value, { min, max, reset });
    if (next === null) return;
    e.preventDefault();
    onchange(next);
  }
</script>

<!-- A focusable separator is a widget (ARIA); Svelte cannot tell. -->
<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
<div
  role="separator"
  tabindex="0"
  aria-orientation="vertical"
  aria-label={label}
  aria-valuenow={value}
  aria-valuemin={min}
  aria-valuemax={max}
  aria-controls={controls}
  data-dragging={drag ? '' : undefined}
  onpointerdown={down}
  onpointermove={move}
  onpointerup={up}
  onpointercancel={up}
  onlostpointercapture={up}
  ondblclick={() => onchange(reset)}
  onkeydown={key}
></div>

<style>
  div {
    position: absolute;
    top: 0;
    bottom: 0;
    right: calc((var(--gap) + var(--resize-hit)) / -2);
    width: var(--resize-hit);
    cursor: col-resize;
    touch-action: none;
  }

  div::after {
    content: '';
    position: absolute;
    inset-block: 0;
    left: 50%;
    width: var(--indicator);
    translate: -50% 0;
    background: var(--accent);
    opacity: 0;
  }

  div:hover::after {
    opacity: 1;
    transition: opacity 0s 120ms;
  }

  div[data-dragging]::after,
  div:focus-visible::after {
    opacity: 1;
  }
</style>
