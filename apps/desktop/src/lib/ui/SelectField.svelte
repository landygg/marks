<!-- `input` as a choice among a few fixed options (DESIGN.md, Components). -->
<script lang="ts" generics="T extends string">
  import type { HTMLSelectAttributes } from 'svelte/elements';

  interface Props extends HTMLSelectAttributes {
    value: T;
    options: readonly { value: T; label: string }[];
    inset?: boolean;
    mono?: boolean;
    field?: boolean;
  }

  let {
    value = $bindable(),
    options,
    inset = false,
    mono = false,
    field = false,
    ...rest
  }: Props = $props();
</script>

<select {...rest} class={['input', { inset, mono, field }]} bind:value>
  {#each options as option (option.value)}
    <option value={option.value}>{option.label}</option>
  {/each}
</select>
