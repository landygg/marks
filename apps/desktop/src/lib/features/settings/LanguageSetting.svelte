<script lang="ts">
  import InlineConfirm from '../../ui/InlineConfirm.svelte';
  import SelectField from '../../ui/SelectField.svelte';
  import SettingRow from '../../ui/SettingRow.svelte';
  import { t } from '../../strings';
  import { LANGUAGES } from '../palette/commands';
  import {
    applyLanguage,
    askLanguage,
    cancelLanguage,
    language,
    loadLanguage,
  } from './languageState.svelte';

  const uid = $props.id();
  void loadLanguage();
</script>

<SettingRow label={t.languageLabel} for="{uid}-language">
  <SelectField
    id="{uid}-language"
    field
    bind:value={() => language.pending ?? language.setting, askLanguage}
    options={LANGUAGES.map((setting) => ({ value: setting, label: t.languages[setting] }))}
  />
  {#snippet note()}
    {#if language.pending}
      <InlineConfirm
        tone="neutral"
        question={t.confirmRelaunch}
        label={t.confirmRelaunch}
        yes={t.confirmRelaunchYes}
        cancel={t.cancel}
        onyes={applyLanguage}
        oncancel={cancelLanguage}
      />
    {/if}
  {/snippet}
</SettingRow>
