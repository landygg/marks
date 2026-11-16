<script lang="ts">
  import Segmented from '../../ui/Segmented.svelte';
  import SettingRow from '../../ui/SettingRow.svelte';
  import { t } from '../../strings';
  import { THEMES } from './theme';
  import { appearance, setTheme } from './themeState.svelte';

  const uid = $props.id();
</script>

<SettingRow label={t.themeLabel} labelId="{uid}-theme">
  <Segmented
    labelledby="{uid}-theme"
    value={appearance.theme}
    options={THEMES.map((theme) => ({ value: theme, label: t.themes[theme] }))}
    onchange={setTheme}
  />
</SettingRow>
