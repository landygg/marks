<!-- "New profile" and "Rename profile": the same dialog (DESIGN.md, Layout). -->
<script lang="ts">
  import { modal } from '../../modal';
  import { suggestInitials, validProfile } from './profiles';
  import { closeProfileDialog, profileDialog, submitProfile } from './profilesState.svelte';
  import { t } from '../../strings';
  import Button from '../../ui/Button.svelte';
  import Field from '../../ui/Field.svelte';
  import TextField from '../../ui/TextField.svelte';

  let saving = $state(false);
  const uid = $props.id();

  function onNameInput(): void {
    if (!profileDialog.initialsEdited) profileDialog.initials = suggestInitials(profileDialog.name);
  }

  async function save(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    const { name, initials } = profileDialog;
    if (saving || !validProfile(name, initials)) return;
    saving = true;
    try {
      await submitProfile(name, initials);
    } finally {
      saving = false;
    }
  }
</script>

<dialog
  {@attach modal(profileDialog.open)}
  aria-labelledby="{uid}-title"
  onclose={closeProfileDialog}
>
  <!-- novalidate: the backend validates, so its message shows here instead of WebKit's bubble. -->
  <form novalidate onsubmit={save}>
    <h2 id="{uid}-title">{profileDialog.id === null ? t.newProfile : t.renameProfileTitle}</h2>

    <Field label={t.profileNameLabel} for="{uid}-name">
      <!-- Inside a modal dialog autofocus is the expected focus target, not a surprise jump. -->
      <TextField
        id="{uid}-name"
        autofocus
        inset
        bind:value={profileDialog.name}
        oninput={onNameInput}
      />
    </Field>

    <Field label={t.profileInitialsLabel} for="{uid}-initials">
      <TextField
        id="{uid}-initials"
        identifier
        inset
        mono
        field
        bind:value={profileDialog.initials}
        oninput={() => (profileDialog.initialsEdited = true)}
      />
    </Field>

    {#if profileDialog.error}<p class="error-text" role="alert">{profileDialog.error}</p>{/if}

    <div class="actions">
      <Button onclick={closeProfileDialog}>{t.cancel}</Button>
      <Button
        variant="primary"
        type="submit"
        disabled={saving || !validProfile(profileDialog.name, profileDialog.initials)}
      >
        {profileDialog.id === null ? t.create : t.save}
      </Button>
    </div>
  </form>
</dialog>

<style>
  dialog {
    padding: var(--space-lg);
  }

  form {
    display: grid;
    gap: var(--space-md);
  }

  h2 {
    margin: 0;
    font: var(--type-headline-lg);
    letter-spacing: var(--tracking-headline-lg);
  }

  /* The backend's error. */
  p {
    margin: 0;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-sm);
  }
</style>
