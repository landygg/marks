<script lang="ts">
  import Button from '../../ui/Button.svelte';
  import Segmented from '../../ui/Segmented.svelte';
  import SettingRow from '../../ui/SettingRow.svelte';
  import SettingRows from '../../ui/SettingRows.svelte';
  import { onMount } from 'svelte';
  import {
    checkUpdate,
    installUpdate,
    openExternal,
    setUpdateAuto,
    updateInfo,
    type Target,
    type UpdateInfo,
  } from '../../api';
  import { report } from '../library/libraryState.svelte';
  import { t } from '../../strings';
  import { AI_MODES, aiMode, enabledFor, RELEASES_URL, type AiMode } from './settingsLogic';

  /** Updates are global: their failures show in every profile and lead back here. */
  const UPDATES_GLOBAL: { profile: null; target: Target } = {
    profile: null,
    target: { kind: 'updates' },
  };

  const uid = $props.id();
  let info = $state<UpdateInfo | null>(null);
  let busy = $state(false);
  /** What the last check or install said, under the version; `failed` turns it into an error. */
  let result = $state<string | null>(null);
  let failed = $state(false);

  onMount(async () => {
    try {
      info = await updateInfo();
    } catch (err) {
      report(err, { ...UPDATES_GLOBAL, toast: false });
    }
  });

  /** Runs a check or an install, showing its failure in line. */
  async function run(action: () => Promise<void>): Promise<void> {
    busy = true;
    failed = false;
    result = null;
    try {
      await action();
    } catch (err) {
      failed = true;
      result = err instanceof Error ? err.message : String(err);
      report(err, { ...UPDATES_GLOBAL, toast: false });
    } finally {
      busy = false;
    }
  }

  const check = () =>
    run(async () => {
      const found = await checkUpdate();
      if (info) info.available = found.kind === 'available' ? found.version : null;
      // A version this Mac cannot run is news, not a failure: plain status text, no notification.
      if (found.kind === 'up_to_date') result = t.updateNone;
      else if (found.kind === 'needs_macos') result = found.message;
    });

  const install = () => run(installUpdate);

  async function setAuto(mode: AiMode): Promise<void> {
    const auto = enabledFor(mode);
    try {
      await setUpdateAuto(auto);
      if (info) info.auto = auto;
    } catch (err) {
      report(err, UPDATES_GLOBAL);
    }
  }

  function openReleases(): void {
    openExternal(RELEASES_URL).catch((err: unknown) => report(err, UPDATES_GLOBAL));
  }
</script>

{#if info}
  <SettingRows>
    {#if info.supported}
      <SettingRow
        label={t.updateCurrent(info.current)}
        help={result}
        helpId="{uid}-result"
        helpRole={failed ? 'alert' : 'status'}
        helpError={failed}
      >
        {#if info.available}
          <Button
            variant="primary"
            disabled={busy}
            aria-describedby="{uid}-result"
            onclick={install}
          >
            {t.updateInstall(info.available)}
          </Button>
        {:else}
          <Button disabled={busy} aria-describedby="{uid}-result" onclick={check}>
            {t.updateCheck}
          </Button>
        {/if}
      </SettingRow>
      <SettingRow label={t.updateAuto} labelId="{uid}-auto" help={t.updateAutoHelp}>
        <Segmented
          labelledby="{uid}-auto"
          value={aiMode(info.auto)}
          options={AI_MODES.map((mode) => ({ value: mode, label: t.updateAutoModes[mode] }))}
          onchange={setAuto}
        />
      </SettingRow>
    {:else}
      <SettingRow label={t.updateCurrent(info.current)} help={t.updateManual}>
        <Button onclick={openReleases}>{t.updateReleases}</Button>
      </SettingRow>
    {/if}
  </SettingRows>
{/if}
