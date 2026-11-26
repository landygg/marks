<script lang="ts">
  import { saveBookmark } from './libraryState.svelte';
  import { ENTER_KEY, isComposing } from '../../shortcuts';
  import { t } from '../../strings';
  import TitleField from '../../ui/TitleField.svelte';

  interface Props {
    /** The profile the bookmark was loaded in; fixed for the editor's life (`Detail` keys on it). */
    profile: number;
    id: number;
    saved: string;
  }

  let { profile, id, saved }: Props = $props();

  let title = $state('');
  // Plain (non-reactive) so the sync below reruns only when the saved title changes.
  let dirty = false;

  // Follow the stored title unless the user has unsaved edits. Not a writable `$derived`: a save's
  // response would overwrite whatever was typed while it was in flight.
  $effect.pre(() => {
    if (!dirty) title = saved;
  });

  function save(): void {
    if (!dirty) return;
    dirty = false;
    if (title !== saved) void saveBookmark(profile, id, { title });
  }

  function blurOnEnter(event: KeyboardEvent & { currentTarget: HTMLInputElement }): void {
    if (isComposing(event)) return;
    if (event.key === ENTER_KEY) event.currentTarget.blur();
  }
</script>

<TitleField
  aria-label={t.titleLabel}
  bind:value={title}
  oninput={() => (dirty = true)}
  onblur={save}
  onkeydown={blurOnEnter}
/>
