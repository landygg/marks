<script lang="ts">
  import { add } from '../../api';
  import { parseTags } from '../../format';
  import { closeAddDialog, lib, refresh, report } from './libraryState.svelte';
  import { modal } from '../../modal';
  import { t } from '../../strings';
  import Button from '../../ui/Button.svelte';
  import Field from '../../ui/Field.svelte';
  import TextArea from '../../ui/TextArea.svelte';
  import TextField from '../../ui/TextField.svelte';

  let url = $state('');
  let notes = $state('');
  let tags = $state('');
  // Error returned by the backend on save (e.g. invalid URL).
  let message = $state<string | null>(null);
  let saving = $state(false);
  const uid = $props.id();

  // Every way out (esc, Cancel, a save) fires `close`: the backend's error goes with it, so the next
  // opening starts clean.
  function onclose(): void {
    message = null;
    closeAddDialog();
  }

  async function save(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    message = null;
    saving = true;
    try {
      await add(url.trim(), notes, parseTags(tags));
    } catch (err) {
      message = String(err);
      report(err, { toast: false });
      return;
    } finally {
      saving = false;
    }
    url = notes = tags = '';
    closeAddDialog();
    await refresh();
  }
</script>

<dialog {@attach modal(lib.addDialogOpen)} aria-labelledby="{uid}-title" {onclose}>
  <!-- novalidate: the backend validates the URL, so its message shows here instead of WebKit's bubble. -->
  <form novalidate onsubmit={save}>
    <h2 id="{uid}-title">{t.addDialogTitle}</h2>

    <Field label={t.urlLabel} for="{uid}-url">
      <!-- Inside a modal dialog autofocus is the expected focus target, not a surprise jump. -->
      <TextField
        id="{uid}-url"
        type="url"
        placeholder={t.urlPlaceholder}
        required
        autofocus
        inset
        bind:value={url}
      />
    </Field>

    <Field label={t.notesLabel} for="{uid}-notes">
      <TextArea id="{uid}-notes" inset bind:value={notes} />
    </Field>

    <Field label={t.tagsLabel} for="{uid}-tags">
      <TextField id="{uid}-tags" placeholder={t.tagsInputPlaceholder} inset bind:value={tags} />
    </Field>

    {#if message}<p class="error-text" role="alert">{message}</p>{/if}

    <div class="actions">
      <Button variant="primary" type="submit" disabled={saving}>{t.save}</Button>
      <Button onclick={closeAddDialog}>{t.cancel}</Button>
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

  p {
    margin: 0;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-sm);
  }
</style>
