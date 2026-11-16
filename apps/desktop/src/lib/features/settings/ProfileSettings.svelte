<script lang="ts">
  import Button from '../../ui/Button.svelte';
  import InlineConfirm from '../../ui/InlineConfirm.svelte';
  import SettingRow from '../../ui/SettingRow.svelte';
  import SettingRows from '../../ui/SettingRows.svelte';
  import TextField from '../../ui/TextField.svelte';
  import { onMount } from 'svelte';
  import { renameProfile } from '../../api';
  import { lib, refresh, report } from '../library/libraryState.svelte';
  import { DEFAULT_PROFILE_ID, validProfile } from '../profiles/profiles';
  import { profileDelete, removeProfile } from '../profiles/profilesState.svelte';
  import { t } from '../../strings';

  const SAVED_NOTICE_MS = 3000;

  // Settings mounts this again on a profile switch: an edit or a result belongs to one profile.
  const profile = lib.profileId;
  // The backend's error on save.
  let message = $state<string | null>(null);
  let saved = $state(false);
  let saving = $state(false);
  let savedTimer: ReturnType<typeof setTimeout> | undefined;
  const uid = $props.id();

  // Strings, not the profile object: every refresh brings new objects, and that must not wipe an edit.
  const activeName = $derived(lib.profiles.find((p) => p.id === profile)?.name ?? '');
  const activeInitials = $derived(lib.profiles.find((p) => p.id === profile)?.initials ?? '');
  // The fields follow the saved values (first load, a rename from the rail) until the user types.
  let name = $derived(activeName);
  let initials = $derived(activeInitials);
  const isDefault = profile === DEFAULT_PROFILE_ID;
  // Without the count the question would be empty: no confirmation until the stats arrive.
  const asking = $derived(profileDelete.asking && !isDefault && lib.stats !== null);
  const question = $derived(
    lib.stats ? `${t.confirmDeleteProfile(lib.stats.total)} ${t.confirmDeleteProfileHint}` : '',
  );

  async function save(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    if (saving || !validProfile(name, initials)) return;
    message = null;
    saving = true;
    try {
      await renameProfile(profile, name, initials);
    } catch (err) {
      message = err instanceof Error ? err.message : String(err);
      report(err, { toast: false });
      return;
    } finally {
      saving = false;
    }
    await refresh();
    saved = true;
    clearTimeout(savedTimer);
    savedTimer = setTimeout(() => (saved = false), SAVED_NOTICE_MS);
  }

  onMount(() => () => clearTimeout(savedTimer));
</script>

<div class="blocks">
  <SettingRows onsubmit={save}>
    <SettingRow label={t.profileNameLabel} for="{uid}-name">
      <TextField id="{uid}-name" field bind:value={name} />
    </SettingRow>
    <SettingRow label={t.profileInitialsLabel} for="{uid}-initials">
      <TextField id="{uid}-initials" mono field bind:value={initials} />
    </SettingRow>
    <SettingRow
      help={message ?? (saved ? t.profileSaved : null)}
      helpRole={message ? 'alert' : 'status'}
      helpError={!!message}
    >
      <Button variant="primary" type="submit" disabled={saving || !validProfile(name, initials)}
        >{t.save}</Button
      >
    </SettingRow>
  </SettingRows>

  <SettingRows>
    <SettingRow
      label={t.deleteProfile}
      help={asking ? question : isDefault ? t.defaultProfileHelp : null}
      helpRole={asking ? 'alert' : undefined}
    >
      {#if asking}
        <InlineConfirm
          label={question}
          yes={t.confirmDeleteProfileYes}
          cancel={t.cancel}
          onyes={() => void removeProfile(profile)}
          oncancel={() => (profileDelete.asking = false)}
        />
      {:else}
        <Button disabled={isDefault || !lib.stats} onclick={() => (profileDelete.asking = true)}
          >{t.deleteProfile}</Button
        >
      {/if}
    </SettingRow>
  </SettingRows>
</div>

<style>
  .blocks {
    display: grid;
    gap: var(--space-lg);
  }
</style>
