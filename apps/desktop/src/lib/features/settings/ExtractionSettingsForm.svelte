<!-- "Extraction with AI": the summary settings, or its own connection and model. -->
<script lang="ts">
  import AiConnectionFields from './AiConnectionFields.svelte';
  import Button from '../../ui/Button.svelte';
  import ModelField from './ModelField.svelte';
  import Segmented from '../../ui/Segmented.svelte';
  import SettingRow from '../../ui/SettingRow.svelte';
  import SettingRows from '../../ui/SettingRows.svelte';
  import { onMount } from 'svelte';
  import {
    getSettings,
    saveExtractionSettings,
    testConnection,
    type AiSettings,
    type ExtractionSettings,
  } from '../../api';
  import { refresh, report } from '../library/libraryState.svelte';
  import {
    AI_GLOBAL,
    EXTRACTION_MODES,
    SAVED_NOTICE_MS,
    extractionMode,
    extractionToSave,
    isRemoteEndpoint,
    sameAsSummaryFor,
  } from './settingsLogic';
  import { t } from '../../strings';

  interface Props {
    /** The summary form as it is now: with the same settings, its connection is the one used. */
    summary: Pick<AiSettings, 'provider' | 'endpoint'>;
  }

  let { summary }: Props = $props();

  let form = $state<ExtractionSettings | null>(null);
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

  /** The page's HTML goes to whichever server does the extraction. */
  const remote = $derived.by(() => {
    if (!form) return false;
    const target = form.same_as_summary ? summary : form;
    return isRemoteEndpoint(target.provider, target.endpoint);
  });

  /** The last row's line: an error first, then the freshest result, else the hint. */
  const result = $derived.by((): { text: string; role?: 'alert' | 'status' } => {
    if (message) return { text: message, role: 'alert' };
    if (connectionError) return { text: connectionError, role: 'alert' };
    if (saved) return { text: t.saved, role: 'status' };
    if (connection) return { text: connection, role: 'status' };
    if (form?.same_as_summary) return { text: t.extractionSameHelp };
    return { text: t.testConnectionHint };
  });

  async function load(): Promise<void> {
    try {
      const view = await getSettings();
      form = view.extraction;
      hasApiKey = view.has_extraction_key;
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
      const toSave = extractionToSave(form, apiKey, clearKey);
      await saveExtractionSettings(toSave.settings, toSave.apiKey);
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
      const r = await testConnection('extraction');
      connection = t.extractionConnectionOk(r.chat_ms);
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
  <SettingRows onsubmit={save}>
    <SettingRow
      label={t.extractionSettingsLabel}
      labelId="{uid}-mode"
      help={remote ? t.extractionRemoteWarning : null}
    >
      <Segmented
        labelledby="{uid}-mode"
        value={extractionMode(form.same_as_summary)}
        options={EXTRACTION_MODES.map((mode) => ({ value: mode, label: t.extractionModes[mode] }))}
        onchange={(mode) => (form!.same_as_summary = sameAsSummaryFor(mode))}
      />
    </SettingRow>
    {#if !form.same_as_summary}
      <AiConnectionFields
        bind:provider={form.provider}
        bind:endpoint={form.endpoint}
        bind:apiKey
        bind:clearKey
        {hasApiKey}
      />
      <ModelField
        label={t.extractionModelLabel}
        provider={form.provider}
        role="extraction"
        required
        bind:value={form.model}
      />
    {/if}
    <SettingRow help={result.text} helpRole={result.role} helpError={result.role === 'alert'}>
      {#if !form.same_as_summary}
        <Button disabled={testing} onclick={runTest}>{t.testConnection}</Button>
      {/if}
      <Button variant="primary" type="submit" disabled={saving}>{t.save}</Button>
    </SettingRow>
  </SettingRows>
{/if}
