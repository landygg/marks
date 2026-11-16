<!-- `command-line` (DESIGN.md, Components): a shell command to run elsewhere, with a copy button. -->
<script lang="ts">
  import CopyButton from './CopyButton.svelte';

  interface Props {
    command: string;
    copyLabel: string;
    /** The button's name for a moment after a copy. */
    copiedLabel: string;
  }

  let { command, copyLabel, copiedLabel }: Props = $props();
</script>

<span class="line">
  <code>{command}</code>
  <CopyButton text={command} label={copyLabel} {copiedLabel} />
</span>

<style>
  .line {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    margin-top: var(--space-xs);
    padding-left: var(--space-sm);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--bg);
    width: fit-content;
    max-width: 100%;
  }

  code {
    min-width: 0;
    overflow-wrap: anywhere;
    font: var(--type-label-md);
    color: var(--fg);
    user-select: text;
  }
</style>
