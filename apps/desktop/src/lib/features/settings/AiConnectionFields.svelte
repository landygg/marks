<!-- Provider, endpoint and key rows of an AI role; each block adds its model rows and its save. -->
<script lang="ts">
  import Button from '../../ui/Button.svelte';
  import SelectField from '../../ui/SelectField.svelte';
  import SettingRow from '../../ui/SettingRow.svelte';
  import TextField from '../../ui/TextField.svelte';
  import type { Provider } from '../../api';
  import { endpointPlaceholder, requiresEndpoint } from './settingsLogic';
  import { t } from '../../strings';

  interface Props {
    provider: Provider;
    endpoint: string | null;
    /** The key being typed; empty keeps the stored one. */
    apiKey: string;
    /** Whether the stored key is deleted on save. */
    clearKey: boolean;
    hasApiKey: boolean;
  }

  let {
    provider = $bindable(),
    endpoint = $bindable(),
    apiKey = $bindable(),
    clearKey = $bindable(),
    hasApiKey,
  }: Props = $props();

  const providers = Object.keys(t.providers) as Provider[];
  const uid = $props.id();
  const keyHelp = $derived(clearKey ? t.apiKeyWillClear : hasApiKey ? t.apiKeyStored : null);
</script>

<SettingRow label={t.providerLabel} for="{uid}-provider">
  <SelectField
    id="{uid}-provider"
    mono
    field
    options={providers.map((p) => ({ value: p, label: t.providers[p] }))}
    bind:value={provider}
  />
</SettingRow>
<SettingRow
  label={t.endpointLabel}
  for="{uid}-endpoint"
  help={requiresEndpoint(provider) ? t.endpointRequired : null}
  helpId="{uid}-endpoint-help"
>
  <TextField
    id="{uid}-endpoint"
    aria-describedby="{uid}-endpoint-help"
    type="url"
    mono
    field
    placeholder={endpointPlaceholder(provider)}
    required={requiresEndpoint(provider)}
    bind:value={endpoint}
  />
</SettingRow>
<SettingRow label={t.apiKeyLabel} for="{uid}-key" help={keyHelp}>
  {#if hasApiKey && !clearKey}
    <Button onclick={() => (clearKey = true)}>{t.clearKey}</Button>
  {/if}
  <TextField id="{uid}-key" type="password" autocomplete="off" mono field bind:value={apiKey} />
</SettingRow>
