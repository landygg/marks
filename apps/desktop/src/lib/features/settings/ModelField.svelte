<!-- A model row of an AI role: the field, the model recommended for the provider and, for Ollama,
     the command that downloads it. -->
<script lang="ts">
  import Button from '../../ui/Button.svelte';
  import CommandLine from '../../ui/CommandLine.svelte';
  import SettingRow from '../../ui/SettingRow.svelte';
  import TextField from '../../ui/TextField.svelte';
  import type { Provider } from '../../api';
  import {
    OLLAMA_EXTRACT_NUM_CTX,
    OLLAMA_SUMMARY_NUM_CTX,
    offersEmbeddings,
    pullCommand,
    recommendedModel,
    type ModelRole,
  } from './settingsLogic';
  import { t } from '../../strings';

  interface Props {
    label: string;
    provider: Provider;
    role: ModelRole;
    value: string;
    required?: boolean;
  }

  let { label, provider, role, value = $bindable(), required = false }: Props = $props();

  const uid = $props.id();
  const rec = $derived(recommendedModel(provider, role));
  const command = $derived(rec && pullCommand(provider, rec.model));
  const help = $derived.by(() => {
    if (!rec) return role === 'embed' && !offersEmbeddings(provider) ? t.noEmbeddings : null;
    const parts = [t.modelRecommended(rec.model)];
    if (rec.sizeGb && rec.ramGb) parts.push(t.modelLocalNeeds(rec.sizeGb, rec.ramGb));
    if (provider === 'ollama' && role === 'chat')
      parts.push(t.ollamaContextChat(OLLAMA_SUMMARY_NUM_CTX, OLLAMA_EXTRACT_NUM_CTX));
    if (provider === 'ollama' && role === 'extraction')
      parts.push(t.ollamaContextExtraction(OLLAMA_EXTRACT_NUM_CTX));
    return parts.join(' ');
  });
</script>

<SettingRow {label} for="{uid}-model" {help} helpId="{uid}-help">
  {#snippet note()}
    {#if command}
      <CommandLine {command} copyLabel={t.copyCommand} copiedLabel={t.commandCopied} />
    {/if}
  {/snippet}
  {#if rec && value.trim() !== rec.model}
    <Button title={t.useRecommendedTitle(rec.model)} onclick={() => (value = rec.model)}
      >{t.useRecommended}</Button
    >
  {/if}
  <!-- Model names are case-sensitive: no autocorrect or capitals from the system. -->
  <TextField
    id="{uid}-model"
    aria-describedby="{uid}-help"
    mono
    field
    {required}
    identifier
    bind:value
  />
</SettingRow>
