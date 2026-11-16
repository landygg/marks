<!-- `content-media` (DESIGN.md, Components): stands in for an image, frame, video or audio. -->
<script lang="ts">
  import Button from './Button.svelte';
  import Icon from './Icon.svelte';
  import type { IconName } from './icons';

  interface Props {
    kind: 'image' | 'video' | 'audio' | 'frame';
    /** `null`: nothing to open, so no action. */
    url: string | null;
    /** Alt text or domain. */
    label: string;
    openLabel: string;
    /** Accessible name of the action; contains `openLabel` and tells lines apart. */
    openName: string;
    onopen: (url: string) => void;
  }

  let { kind, url, label, openLabel, openName, onopen }: Props = $props();

  // shortcut: one generic icon for every kind, a per-kind set when the icon set grows.
  const icon: IconName = 'external';
</script>

<span class="media" data-kind={kind}>
  <Icon name={icon} />
  <span class="label">{label}</span>
  {#if url !== null}
    <Button aria-label={openName} onclick={() => onopen(url)}>{openLabel}</Button>
  {/if}
</span>

<style>
  .media {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-sm);
    border-radius: var(--radius-sm);
    background: var(--bg);
    color: var(--muted);
    font: var(--type-label-md);
  }

  .label {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
