<script lang="ts">
  import { ICONS, type IconName, type IconShape } from './icons';

  interface Props {
    name: IconName;
    /** `icon-small-size`, for chevrons, the drag handle and the tag's remove mark. */
    small?: boolean;
  }

  let { name, small = false }: Props = $props();

  const shape: IconShape = $derived(ICONS[name]);
</script>

<svg class={{ small }} viewBox="0 0 24 24" aria-hidden="true">
  {#if shape.circle}
    <circle cx={shape.circle[0]} cy={shape.circle[1]} r={shape.circle[2]} />
  {/if}
  {#each shape.paths as d (d)}
    <path {d} />
  {/each}
</svg>

<style>
  svg {
    flex: none;
    width: var(--icon);
    height: var(--icon);
    fill: none;
    stroke: currentColor;
    stroke-width: var(--icon-stroke);
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .small {
    width: var(--icon-small);
    height: var(--icon-small);
  }
</style>
