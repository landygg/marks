<script lang="ts">
  import { onDestroy } from 'svelte';
  import Button from '../../ui/Button.svelte';
  import InlineConfirm from '../../ui/InlineConfirm.svelte';
  import SettingRow from '../../ui/SettingRow.svelte';
  import SettingRows from '../../ui/SettingRows.svelte';
  import {
    exportBookmarks,
    importCancel,
    importConfirm,
    importPreview,
    type ImportSummary,
  } from '../../api';
  import { lib, refresh, report } from '../library/libraryState.svelte';
  import { notify } from '../notifications/notificationsState.svelte';
  import { exportFileName } from './settingsLogic';
  import { t } from '../../strings';

  // Settings mounts this again on a profile switch: a pending import belongs to one profile.
  const profile = lib.profileId;
  const profileName = $derived(lib.profiles.find((p) => p.id === profile)?.name ?? '');
  let preview = $state<ImportSummary | null>(null);
  // A native dialog or the import is running: a second click would start another one.
  let busy = $state(false);
  let destroyed = false;

  /** Runs `task` unless another one is running; its failure goes to the notifications. */
  async function once(task: () => Promise<void>): Promise<void> {
    if (busy) return;
    busy = true;
    try {
      await task();
    } catch (err) {
      report(err, { profile });
    } finally {
      busy = false;
    }
  }

  const done = (message: string): void =>
    void notify(
      { profile, severity: 'info', message, detail: null, target: null },
      { toast: true },
    );

  const pick = () =>
    once(async () => {
      const summary = await importPreview(profile);
      // The dialog outlived the section: nothing is left to confirm it.
      if (destroyed && summary)
        importCancel().catch((err: unknown) => report(err, { toast: false, profile }));
      else preview = summary;
    });

  // On failure the backend keeps the preview: it stays on screen to try again or cancel.
  const confirm = () =>
    once(async () => {
      const summary = await importConfirm(profile);
      preview = null;
      done(t.notices.importDone(summary.new));
      if (lib.profileId === profile) await refresh();
    });

  /** Drops the preview here and in the backend; `toast` off when nobody clicked. */
  function discard(toast: boolean): void {
    preview = null;
    importCancel().catch((err: unknown) => report(err, { toast, profile }));
  }

  const exportFile = () =>
    once(async () => {
      const path = await exportBookmarks(profile, exportFileName(profileName, new Date()));
      if (path) done(t.notices.exportDone(path));
    });

  // Leaving Settings or switching profile with the summary on screen: nothing is left to confirm it.
  onDestroy(() => {
    destroyed = true;
    if (preview) discard(false);
  });
</script>

<SettingRows>
  <SettingRow
    label={t.importLabel}
    help={preview ? t.importSummary(preview, profileName) : t.importHelp}
    helpRole={preview ? 'alert' : undefined}
  >
    {#if preview}
      <InlineConfirm
        tone="neutral"
        label={t.importSummary(preview, profileName)}
        yes={t.importYes}
        cancel={t.cancel}
        onyes={confirm}
        oncancel={() => discard(true)}
      />
    {:else}
      <Button disabled={busy} onclick={pick}>{t.importAction}</Button>
    {/if}
  </SettingRow>
  <SettingRow label={t.exportLabel} help={t.exportHelp}>
    <Button disabled={busy} onclick={exportFile}>{t.exportAction}</Button>
  </SettingRow>
</SettingRows>
