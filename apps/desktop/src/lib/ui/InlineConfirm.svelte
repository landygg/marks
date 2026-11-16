<!-- `inline-confirm` (DESIGN.md, Components): the question of an action, asked in place; danger unless `tone` is neutral. -->
<script lang="ts">
  import { tick } from 'svelte';
  import Button from './Button.svelte';
  import IconButton from './IconButton.svelte';
  import { ESCAPE_KEY } from '../shortcuts';

  interface Props {
    /** Shown before the answers; left out when the place already states it. */
    question?: string;
    /** What a screen reader announces when there is no visible question. */
    label: string;
    yes: string;
    cancel: string;
    onyes: () => void;
    oncancel: () => void;
    /** Icon answers, for narrow places (the sidebar). */
    compact?: boolean;
    /** `danger` for what replaces or deletes content; `neutral` for the rest. */
    tone?: 'danger' | 'neutral';
  }

  let {
    question,
    label,
    yes,
    cancel,
    onyes,
    oncancel,
    compact = false,
    tone = 'danger',
  }: Props = $props();

  const uid = $props.id();

  // It may appear from the palette: bring it into view with the safe answer focused.
  // After `tick`, so a modal that is closing in the same update no longer holds the focus.
  function reveal(node: HTMLElement): void {
    void tick().then(() => {
      node.scrollIntoView({ block: 'nearest' });
      node.querySelector<HTMLElement>('[data-cancel]')?.focus();
    });
  }

  function onkeydown(event: KeyboardEvent): void {
    if (event.key !== ESCAPE_KEY) return;
    // Handled here: the window's Esc (closing the notification panel) leaves it alone.
    event.preventDefault();
    oncancel();
  }
</script>

<div
  class={{ compact, asks: !!question, neutral: tone === 'neutral' }}
  role="alertdialog"
  aria-label={question ? undefined : label}
  aria-labelledby={question ? `${uid}-question` : undefined}
  tabindex="-1"
  {onkeydown}
  {@attach reveal}
>
  {#if question}<span id="{uid}-question">{question}</span>{/if}
  {#if compact}
    <IconButton icon="check" label={yes} danger={tone === 'danger'} onclick={onyes} />
    <IconButton icon="close" label={cancel} data-cancel onclick={oncancel} />
  {:else}
    <Button variant={tone === 'danger' ? 'danger' : 'secondary'} onclick={onyes}>{yes}</Button>
    <Button data-cancel onclick={oncancel}>{cancel}</Button>
  {/if}
</div>

<style>
  div {
    display: flex;
    flex: none;
    align-items: center;
    gap: var(--space-sm);
  }

  /* With its question it takes the free width, so the answers sit at the right. */
  .asks {
    flex: 1;
    min-width: 0;
  }

  span {
    flex: 1;
    min-width: 0;
    color: var(--danger);
  }

  .neutral span {
    color: var(--fg);
  }

  .compact {
    gap: 0;
  }

  .compact span {
    overflow: hidden;
    font: var(--type-label-md);
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
