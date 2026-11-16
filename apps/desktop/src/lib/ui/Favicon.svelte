<!-- `detail-favicon` (DESIGN.md, Components): the page's favicon, stored by core and passed as a
     `data:` URL. Decorative, and nothing at all when missing or broken. An SVG favicon must stay
     in `<img>`, never inline: there WebKit runs no script and loads nothing but `data:`. -->
<script lang="ts">
  interface Props {
    /** `null`: nothing stored, nothing drawn. */
    src: string | null;
  }

  let { src }: Props = $props();

  // The source that failed to decode; a new one gets its own chance.
  let broken = $state<string | null>(null);
</script>

{#if src && src !== broken}
  <img {src} alt="" onerror={() => (broken = src)} />
{/if}

<style>
  img {
    display: block;
    flex: none;
    width: var(--icon);
    height: var(--icon);
    border-radius: var(--radius-sm);
    object-fit: contain;
  }
</style>
