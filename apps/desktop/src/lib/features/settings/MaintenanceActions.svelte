<script lang="ts">
  import Button from '../../ui/Button.svelte';
  import InlineConfirm from '../../ui/InlineConfirm.svelte';
  import SettingRow from '../../ui/SettingRow.svelte';
  import SettingRows from '../../ui/SettingRows.svelte';
  import { processUnsummarized, reindex } from '../../api';
  import { lib, refresh, report } from '../library/libraryState.svelte';
  import { readyCount } from './settingsLogic';
  import { t } from '../../strings';

  // Settings mounts this again on a profile switch: a question or a notice belongs to one profile.
  const profile = lib.profileId;
  let confirming = $state(false);
  let notice = $state<string | null>(null);

  /** Runs a re-queue command on the profile shown, shows how many it moved and reloads the counts. */
  async function run(
    command: (profile: number) => Promise<number>,
    queued: (n: number) => string,
  ): Promise<void> {
    confirming = false;
    try {
      notice = queued(await command(profile));
    } catch (err) {
      report(err);
    }
    await refresh();
  }
</script>

{#if lib.stats}
  <SettingRows>
    <SettingRow
      label={t.reindex}
      help={t.confirmReindex(readyCount(lib.stats))}
      helpRole={confirming ? 'alert' : undefined}
    >
      {#if confirming}
        <InlineConfirm
          label={t.confirmReindex(readyCount(lib.stats))}
          yes={t.confirmReindexYes}
          tone="neutral"
          cancel={t.cancel}
          onyes={() => run(reindex, t.reindexQueued)}
          oncancel={() => (confirming = false)}
        />
      {:else}
        <Button onclick={() => (confirming = true)}>{t.reindexAction}</Button>
      {/if}
    </SettingRow>
    <SettingRow label={t.processLabel} help={t.processPending(lib.stats.unsummarized)}>
      <Button
        disabled={lib.stats.unsummarized === 0}
        onclick={() => run(processUnsummarized, t.processQueued)}
      >
        {t.processAction}
      </Button>
    </SettingRow>
    {#if notice}<SettingRow help={notice} helpRole="status" />{/if}
  </SettingRows>
{/if}
