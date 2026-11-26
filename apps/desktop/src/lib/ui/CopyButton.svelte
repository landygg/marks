<!-- `copy-button` (DESIGN.md, Components): copies a whole text, shown or not, and confirms for a moment. -->
<script lang="ts">
  import IconButton from './IconButton.svelte';

  interface Props {
    /** What goes to the clipboard, whole. */
    text: string;
    label: string;
    /** The button's name for a moment after a copy, also announced to screen readers. */
    copiedLabel: string;
  }

  let { text, label, copiedLabel }: Props = $props();

  /** How long the button shows that the copy worked. */
  const COPIED_MS = 1500;
  let copied = $state(false);
  let timer: ReturnType<typeof setTimeout> | undefined;

  async function copy(): Promise<void> {
    try {
      await navigator.clipboard.writeText(text);
    } catch {
      return; // The text stays selectable to copy by hand.
    }
    copied = true;
    clearTimeout(timer);
    timer = setTimeout(() => (copied = false), COPIED_MS);
  }
</script>

<span class="copy">
  <IconButton
    icon={copied ? 'check' : 'copy'}
    label={copied ? copiedLabel : label}
    success={copied}
    onclick={copy}
  />
  <span class="announce" role="status">{copied ? copiedLabel : ''}</span>
</span>

<style>
  .copy {
    display: flex;
    flex: none;
  }
</style>
