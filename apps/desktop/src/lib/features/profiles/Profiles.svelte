<!-- The profile rail (DESIGN.md, Layout): one item per profile and "new profile" at the end. -->
<script lang="ts">
  import { lib, toggleSettings } from '../library/libraryState.svelte';
  import { showProfileMenu } from './profileMenu';
  import { parseBookmarkDrag } from './profiles';
  import { BOOKMARK_MIME, moveBookmarkTo, openNewProfile, switchTo } from './profilesState.svelte';
  import { PROFILE_SHORTCUT_COUNT } from '../../shortcuts';
  import { t } from '../../strings';
  import IconButton from '../../ui/IconButton.svelte';
  import ProfileItem from '../../ui/ProfileItem.svelte';
  import ProfileRail from '../../ui/ProfileRail.svelte';

  const MOVE = 'move';

  // The profile a bookmark is dragged over, and how many of its elements the pointer is inside:
  // `dragleave` also fires when moving between children (see `Sidebar.svelte`).
  let dropId = $state<number | null>(null);
  let dropDepth = 0;

  /** A bookmark row is dragged, and `id` is not the profile it is in. */
  function accepts(event: DragEvent, id: number): boolean {
    return id !== lib.profileId && (event.dataTransfer?.types.includes(BOOKMARK_MIME) ?? false);
  }

  function enter(event: DragEvent, id: number): void {
    if (!accepts(event, id)) return;
    dropDepth = dropId === id ? dropDepth + 1 : 1;
    dropId = id;
  }

  function over(event: DragEvent, id: number): void {
    if (!accepts(event, id)) return;
    event.preventDefault();
    if (event.dataTransfer) event.dataTransfer.dropEffect = MOVE;
  }

  function leave(id: number): void {
    if (dropId !== id) return;
    dropDepth -= 1;
    if (dropDepth <= 0) dropId = null;
  }

  function drop(event: DragEvent, id: number): void {
    if (!accepts(event, id)) return;
    event.preventDefault();
    endDrag();
    const dragged = parseBookmarkDrag(event.dataTransfer?.getData(BOOKMARK_MIME) ?? '');
    if (dragged) void moveBookmarkTo(dragged.id, dragged.profile, id);
  }

  // Also bound to the window: a drop elsewhere, or a drag whose row left the list, ends here.
  function endDrag(): void {
    dropId = null;
    dropDepth = 0;
  }
</script>

<svelte:window ondragend={endDrag} ondrop={endDrag} />

<ProfileRail label={t.profilesLabel}>
  {#each lib.profiles as profile, i (profile.id)}
    <ProfileItem
      label={t.profileTitle(profile.name, i < PROFILE_SHORTCUT_COUNT ? t.profileShortcut(i) : null)}
      selected={profile.id === lib.profileId}
      drop={dropId === profile.id}
      onclick={() => switchTo(profile.id)}
      oncontextmenu={(e) => {
        e.preventDefault();
        void showProfileMenu(profile);
      }}
      ondragenter={(e) => enter(e, profile.id)}
      ondragover={(e) => over(e, profile.id)}
      ondragleave={() => leave(profile.id)}
      ondrop={(e) => drop(e, profile.id)}
    >
      {profile.initials}
    </ProfileItem>
  {/each}
  <ProfileItem add label={t.newProfile} onclick={openNewProfile} />
  {#snippet foot()}
    <IconButton
      icon="gear"
      label={t.settingsButton}
      pressed={lib.view === 'settings'}
      onclick={toggleSettings}
    />
  {/snippet}
</ProfileRail>
