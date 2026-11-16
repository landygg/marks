<!-- Settings → AI: the global switch with "Summary and search", then "Extraction with AI". -->
<script lang="ts">
  import AiConnectionFields from './AiConnectionFields.svelte';
  import Button from '../../ui/Button.svelte';
  import ExtractionSettingsForm from './ExtractionSettingsForm.svelte';
  import ModelField from './ModelField.svelte';
  import Segmented from '../../ui/Segmented.svelte';
  import SettingRow from '../../ui/SettingRow.svelte';
  import SettingRows from '../../ui/SettingRows.svelte';
  import { onMount } from 'svelte';
  import { getSettings, saveSettings, testConnection, type AiSettings } from '../../api';
  import { refresh, report } from '../library/libraryState.svelte';
  import {
    AI_GLOBAL,
    AI_MODES,
    SAVED_NOTICE_MS,
    aiMode,
    enabledFor,
    keyToSend,
  } from './settingsLogic';
  import { t } from '../../strings';

  let form = $state<AiSettings | null>(null);
  let hasApiKey = $state(false);
  let apiKey = $state('');
  let clearKey = $state(false);
  // Validation error returned by the backend on save.
  let message = $state<string | null>(null);
  let saved = $state(false);
  let saving = $state(false);
  let testing = $state(false);
  let connection = $state<string | null>(null);
  let connectionError = $state<string | null>(null);
  let savedTimer: ReturnType<typeof setTimeout> | undefined;
  const uid = $props.id();

  /** The last row's line: an error first, then the freshest result, else the hint. */
  const result = $derived.by((): { text: string; role?: 'alert' | 'status' } => {
    if (message) return { text: message, role: 'alert' };
    if (connectionError) return { text: connectionError, role: 'alert' };
    if (saved) return { text: t.saved, role: 'status' };
    if (connection) return { text: connection, role: 'status' };
    return { text: t.testConnectionHint };
  });

  async function load(): Promise<void> {
    try {
      const view = await getSettings();
      form = view.settings;
      hasApiKey = view.has_api_key;
      apiKey = '';
      clearKey = false;
    } catch (err) {
      report(err, AI_GLOBAL);
    }
  }

  async function save(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    if (!form) return;
    message = null;
    saving = true;
    try {
      await saveSettings(form, keyToSend(apiKey, clearKey));
    } catch (err) {
      message = String(err);
      report(err, { ...AI_GLOBAL, toast: false });
      return;
    } finally {
      saving = false;
    }
    await load();
    // "Extract with AI" in the detail follows the model these settings build.
    void refresh();
    saved = true;
    clearTimeout(savedTimer);
    savedTimer = setTimeout(() => (saved = false), SAVED_NOTICE_MS);
  }

  async function runTest(): Promise<void> {
    testing = true;
    connection = null;
    connectionError = null;
    try {
      const r = await testConnection('summary');
      connection = t.connectionOk(r.chat_ms, r.embed_ms, r.dim);
    } catch (err) {
      connectionError = String(err);
      report(err, { ...AI_GLOBAL, toast: false });
    } finally {
      testing = false;
    }
  }

  onMount(() => {
    void load();
    return () => clearTimeout(savedTimer);
  });
</script>

{#if form}
  {#if form.enabled}
    <h2 class="section-label">{t.summaryBlock}</h2>
  {/if}
  <SettingRows onsubmit={save}>
    <SettingRow label={t.aiLabel} labelId="{uid}-ai">
      <Segmented
        labelledby="{uid}-ai"
        value={aiMode(form.enabled)}
        options={AI_MODES.map((mode) => ({ value: mode, label: t.aiModes[mode] }))}
        onchange={(mode) => (form!.enabled = enabledFor(mode))}
      />
    </SettingRow>
    {#if !form.enabled}
      <SettingRow help={t.aiOffNotice} />
    {:else}
      <AiConnectionFields
        bind:provider={form.provider}
        bind:endpoint={form.endpoint}
        bind:apiKey
        bind:clearKey
        {hasApiKey}
      />
      <ModelField
        label={t.chatModelLabel}
        provider={form.provider}
        role="chat"
        bind:value={form.chat_model}
      />
      <ModelField
        label={t.embedModelLabel}
        provider={form.provider}
        role="embed"
        bind:value={form.embed_model}
      />
    {/if}
    <SettingRow help={result.text} helpRole={result.role} helpError={result.role === 'alert'}>
      {#if form.enabled}
        <Button disabled={testing} onclick={runTest}>{t.testConnection}</Button>
      {/if}
      <Button variant="primary" type="submit" disabled={saving}>{t.save}</Button>
    </SettingRow>
  </SettingRows>
  {#if form.enabled}
    <h2 class="section-label">{t.extractionBlock}</h2>
    <ExtractionSettingsForm summary={form} />
  {/if}
{/if}
