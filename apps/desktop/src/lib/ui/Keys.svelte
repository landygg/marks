<!-- `kbd` (DESIGN.md, Components): a shortcut in one box, its keys split by a centered dot that
     screen readers skip. The box's look is the `kbd` element style in app.css. -->
<script lang="ts">
  import { shortcutKeys } from './keys';

  /** The shortcut as strings.ts writes it ("⌥⌘2"). */
  let { shortcut }: { shortcut: string } = $props();

  const SEPARATOR = '·';
  const keys = $derived(shortcutKeys(shortcut));
</script>

<kbd
  >{#each keys as key, i (key)}{#if i > 0}<span aria-hidden="true">{SEPARATOR}</span
      >{/if}{key}{/each}</kbd
>
