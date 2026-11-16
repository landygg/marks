<!-- `clamp-text` (DESIGN.md, Components): a long text cut to two lines, with "show more" when cut. -->
<script lang="ts">
  import type { Attachment } from 'svelte/attachments';
  import Button from './Button.svelte';

  interface Props {
    text: string;
    moreLabel: string;
    lessLabel: string;
  }

  let { text, moreLabel, lessLabel }: Props = $props();

  const id = $props.id();
  let expanded = $state(false);
  // Whether the clamped text runs past its lines: only then is there anything to show.
  let cut = $state(false);

  /** Measures the clamp again when the text or the width changes. */
  const measure: Attachment<HTMLElement> = (node) => {
    void text;
    const check = (): void => {
      if (!node.classList.contains('open')) cut = node.scrollHeight > node.clientHeight;
    };
    const observer = new ResizeObserver(check);
    observer.observe(node);
    check();
    return () => observer.disconnect();
  };
</script>

<div class="clamp-text">
  <!-- Clamped only visually: screen readers get the whole text. -->
  <p {id} class={{ open: expanded }} {@attach measure}>{text}</p>
  {#if cut || expanded}
    <Button
      variant="link"
      aria-expanded={expanded}
      aria-controls={id}
      onclick={() => (expanded = !expanded)}>{expanded ? lessLabel : moreLabel}</Button
    >
  {/if}
</div>

<style>
  .clamp-text {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: var(--space-xs);
    min-width: 0;
  }

  p {
    display: -webkit-box;
    margin: 0;
    overflow: hidden;
    font: var(--type-label-md);
    overflow-wrap: anywhere;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 2;
    line-clamp: 2;
  }

  .open {
    display: block;
  }
</style>
