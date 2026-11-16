<script lang="ts">
  import Segmented from '../../ui/Segmented.svelte';
  import SettingRow from '../../ui/SettingRow.svelte';
  import { t } from '../../strings';
  import { CORNERS, parseCorner } from './corners';
  import { corners, setCorner } from './cornersState.svelte';

  const uid = $props.id();
</script>

<SettingRow label={t.cornersLabel} labelId="{uid}-corners" help={t.cornersHelp}>
  <Segmented
    labelledby="{uid}-corners"
    value={String(corners.value)}
    options={CORNERS.map((c) => ({ value: String(c), label: String(c) }))}
    onchange={(v) => setCorner(parseCorner(v))}
  />
</SettingRow>
