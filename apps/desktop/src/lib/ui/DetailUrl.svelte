<!-- `detail-url` (DESIGN.md, Components): the bookmark's URL on one line, cut in the middle. -->
<script lang="ts">
  import { splitTail } from './ellipsis';

  interface Props {
    url: string;
    /** Opens the URL; the click's default is already prevented. */
    onopen: (url: string) => void;
  }

  let { url, onopen }: Props = $props();

  const parts = $derived(splitTail(url));

  function open(event: MouseEvent): void {
    event.preventDefault();
    onopen(url);
  }
</script>

<!-- The whole URL in the tooltip and for screen readers; the two halves are only what is drawn. -->
<a href={url} title={url} onclick={open}>
  <span class="whole">{url}</span>
  <span class="head" aria-hidden="true">{parts.head}</span><span class="tail" aria-hidden="true"
    >{parts.tail}</span
  >
</a>

<style>
  a {
    display: flex;
    min-width: 0;
    font: var(--type-label-md);
    white-space: nowrap;
  }

  /* Room below for the `link` underline, which `overflow: hidden` would clip otherwise. */
  .head {
    min-width: 0;
    margin-bottom: calc(-1 * var(--space-xs));
    padding-bottom: var(--space-xs);
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tail {
    flex: none;
  }

  .whole {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip-path: inset(50%);
  }
</style>
