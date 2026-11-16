<!-- `status-dot-*` (DESIGN.md, Components). -->
<script module lang="ts">
  export type StatusDotKind =
    'extracting' | 'pending' | 'pending_embed' | 'ready' | 'failed' | 'imported';
</script>

<script lang="ts">
  interface Props {
    status: StatusDotKind;
    /** The state in words. Left out when it is written next to the dot, which is then decorative. */
    label?: string;
  }

  let { status, label }: Props = $props();
</script>

<span
  class={status}
  role={label ? 'img' : undefined}
  title={label}
  aria-label={label}
  aria-hidden={label ? undefined : true}
></span>

<style>
  span {
    flex: none;
    width: var(--dot);
    height: var(--dot);
    border-radius: var(--radius-sm);
    background: var(--muted);
  }

  .extracting {
    /* --focus is the accent on dark and ink on light, where yellow has no contrast. */
    background: var(--focus);
  }

  .pending_embed {
    background: var(--warn);
  }

  .ready {
    background: var(--ok);
  }

  .failed {
    background: var(--danger);
  }

  /* Hollow: not queued yet. */
  .imported {
    box-sizing: border-box;
    border: 1px solid var(--muted);
    background: none;
  }
</style>
