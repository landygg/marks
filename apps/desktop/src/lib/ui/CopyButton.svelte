<!-- `copy-button` (DESIGN.md, Components): copies a whole text, shown or not, and confirms for a moment. -->
<script lang="ts">
  import IconButton from './IconButton.svelte';

  interface Props {
    /** What goes to the clipboard, whole. */
    text: string;
    label: string;
    /** The button's name for a moment after a copy, also announced to screen readers. */
    copiedLabel: string;
    /** Announced for the same moment when the clipboard refuses; the icon does not change. */
    failedLabel: string;
  }

  let { text, label, copiedLabel, failedLabel }: Props = $props();

  /** How long the button shows that the copy worked, or the failure is announced. */
  const COPIED_MS = 1500;
  let result = $state<'copied' | 'failed' | null>(null);
  const copied = $derived(result === 'copied');
  let timer: ReturnType<typeof setTimeout> | undefined;

  async function copy(): Promise<void> {
    try {
      await navigator.clipboard.writeText(text);
      result = 'copied';
    } catch {
      result = 'failed'; // The text stays selectable to copy by hand.
    }
    clearTimeout(timer);
    timer = setTimeout(() => (result = null), COPIED_MS);
  }
</script>

<span class="copy">
  <IconButton
    icon={copied ? 'check' : 'copy'}
    label={copied ? copiedLabel : label}
    success={copied}
    onclick={copy}
  />
  <span class="announce" role="status"
    >{result === 'copied' ? copiedLabel : result === 'failed' ? failedLabel : ''}</span
  >
</span>

<style>
  .copy {
    display: flex;
    flex: none;
  }
</style>
