<!-- `icon-button` (DESIGN.md, Components). -->
<script lang="ts">
  import type { HTMLButtonAttributes } from 'svelte/elements';
  import Icon from './Icon.svelte';
  import type { IconName } from './icons';

  interface Props extends HTMLButtonAttributes {
    icon: IconName;
    /** Accessible name and tooltip: the button has no visible text. */
    label: string;
    danger?: boolean;
    /** A confirmation (`copy-button-copied`): the icon in `success`. */
    success?: boolean;
    /** For a toggle: whether it is on. Left out, the button is not a toggle. */
    pressed?: boolean;
    /** Something new behind it (unread notifications); `label` must say what, the mark is only visual. */
    marked?: boolean;
    /** Why it does nothing now: unlike `disabled`, the button stays focusable and the reason is read
        out with it (and is its tooltip). */
    reason?: string;
  }

  let {
    icon,
    label,
    danger = false,
    success = false,
    pressed,
    marked = false,
    reason,
    onclick,
    ...rest
  }: Props = $props();

  const reasonId = $props.id();
</script>

<button
  type="button"
  class={{ danger, success, marked }}
  aria-label={label}
  aria-pressed={pressed}
  aria-disabled={reason ? 'true' : undefined}
  aria-describedby={reason ? reasonId : undefined}
  title={reason ?? label}
  {...rest}
  onclick={reason ? undefined : onclick}
>
  <Icon name={icon} />
  {#if marked}<span class="mark" aria-hidden="true"></span>{/if}
</button>
{#if reason}<span class="announce" id={reasonId}>{reason}</span>{/if}

<style>
  button {
    position: relative;
    display: grid;
    flex: none;
    place-items: center;
    width: var(--control);
    height: var(--control);
    color: var(--muted);
  }

  button:hover:not(:disabled, [aria-disabled='true']),
  button[aria-pressed='true'],
  button[aria-expanded='true'] {
    background: var(--border);
    color: var(--fg);
  }

  .danger,
  .danger:hover:not(:disabled, [aria-disabled='true']) {
    color: var(--danger);
  }

  .success,
  .success:hover:not(:disabled, [aria-disabled='true']) {
    color: var(--ok);
  }

  /* `icon-button-marked`: the icon lights up and carries a marker-yellow square on its corner,
     which beats three times when it appears and then stays still. */
  .marked {
    color: var(--fg);
  }

  .mark {
    position: absolute;
    top: var(--space-xs);
    right: var(--space-xs);
    width: var(--dot);
    height: var(--dot);
    border-radius: var(--radius-sm);
    background: var(--accent);
    animation: beat 1s ease-in-out 3;
  }

  @keyframes beat {
    50% {
      scale: 1.5;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .mark {
      animation: none;
    }
  }
</style>
