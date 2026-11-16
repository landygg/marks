<!-- `segmented-control` and `segment` (DESIGN.md, Components): one choice among a few fixed options. -->
<script lang="ts" generics="T extends string">
  interface Props {
    value: T;
    options: readonly { value: T; label: string }[];
    onchange: (value: T) => void;
    /** Id of the element that names the group. */
    labelledby: string;
  }

  let { value, options, onchange, labelledby }: Props = $props();
</script>

<div role="group" aria-labelledby={labelledby}>
  {#each options as option (option.value)}
    <button
      type="button"
      aria-pressed={value === option.value}
      onclick={() => onchange(option.value)}
    >
      {option.label}
    </button>
  {/each}
</div>

<style>
  div {
    display: inline-flex;
    flex: none;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--surface);
  }

  button {
    height: var(--row);
    padding: 0 var(--space-md);
    border-right: 1px solid var(--border);
    border-radius: 0;
    color: var(--muted);
    font: var(--type-label-md);
  }

  button:last-child {
    border-right: 0;
  }

  button:hover {
    color: var(--fg);
  }

  button[aria-pressed='true'] {
    background: var(--accent);
    color: var(--on-accent);
  }
</style>
