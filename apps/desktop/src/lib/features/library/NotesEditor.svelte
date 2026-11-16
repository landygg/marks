<script lang="ts">
  import { onDestroy } from 'svelte';
  import { lib, saveBookmark } from './libraryState.svelte';
  import { t } from '../../strings';
  import TextArea from '../../ui/TextArea.svelte';

  const NOTES_DEBOUNCE_MS = 600;

  interface Props {
    /** The profile the bookmark was loaded in; fixed for the editor's life (`Detail` keys on it). */
    profile: number;
    id: number;
    saved: string;
  }

  let { profile, id, saved }: Props = $props();

  let notes = $state('');
  // Plain (non-reactive) so the sync below reruns only when the saved notes change.
  let dirty = false;
  let timer: ReturnType<typeof setTimeout> | undefined;
  const textareaId = $props.id();

  // Follow the stored notes unless the user has unsaved edits.
  $effect.pre(() => {
    if (!dirty) notes = saved;
  });

  function schedule(): void {
    dirty = true;
    clearTimeout(timer);
    timer = setTimeout(flush, NOTES_DEBOUNCE_MS);
  }

  function flush(): void {
    clearTimeout(timer);
    if (!dirty) return;
    dirty = false;
    // Deleted (here or elsewhere): nothing left to save into. After a profile switch the list is
    // another profile's, so it says nothing about this bookmark.
    if (lib.profileId === profile && !lib.bookmarks.some((b) => b.id === id)) return;
    void saveBookmark(profile, id, { notes });
  }

  onDestroy(flush);
</script>

<label class="section-label" for={textareaId}>{t.notesLabel}</label>
<TextArea id={textareaId} inset bind:value={notes} oninput={schedule} onblur={flush} />
